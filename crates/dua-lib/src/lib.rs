//! Parallel filesystem traversal backed by a work-stealing worker pool.
//!
//! [`walk`] yields the root first, then workers read directories and distribute newly discovered
//! subdirectories among themselves. [`Order::ParentFirst`] publishes each directory's entries
//! before scheduling its children, while [`Order::Completion`] allows descendant batches to arrive
//! first when their reads finish sooner. Sibling order is unspecified in both modes.
//!
//! The `descend` predicate controls which directories are traversed; rejected directories are
//! still yielded (but not traversed).
//! Symbolic links are reported but never followed, and filesystem errors are
//! returned as iterator items. Dropping the iterator stops and joins its workers.
//!
//! # Scheduling
//!
//! The root directory starts in a shared injector queue. On platforms where directory-entry
//! metadata may require another syscall, directory reads enqueue small metadata batches, and
//! metadata batches enqueue accepted child directories. Windows workers instead consume the
//! metadata returned by directory enumeration directly and enqueue child directories immediately.
//! Every worker can run available jobs from its local LIFO queue or steal from a peer. Each
//! successful thief wakes another idle worker, ramping up only while work remains stealable. A
//! worker parks when no queue has work and is unparked when new work arrives or the walk stops. The
//! last completed job emits the finished event; dropping the iterator stops and joins all workers.
#![deny(unsafe_code)]
#![deny(missing_docs)]

use crossbeam::{
    deque::{Injector, Steal, Stealer, Worker},
    sync::{Parker, Unparker},
};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread,
};

#[cfg(any(not(windows), test))]
use std::{ffi::OsString, fs};

#[cfg(not(windows))]
pub use std::fs::{FileType, Metadata};

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows;

#[cfg(windows)]
pub use windows::{Entry, FileType, Metadata};

/// Decides whether to traverse an entry's children for a given root index.
/// Returning `false` prunes descendants but still emits the entry itself.
type Descend = dyn Fn(usize, &Entry) -> bool + Send + Sync;
/// Entries obtained from one directory read.
/// The outer error means `fs::read_dir` could not open the directory; inner errors come from
/// reading or converting individual directory entries.
type Batch = io::Result<Vec<io::Result<Entry>>>;
/// Number of directory entries grouped into each metadata job or result batch.
/// Small chunks expose parallel work and stream wide directories while amortizing queue overhead.
const ENTRY_CHUNK_SIZE: usize = 4;

/// Controls when entries are yielded relative to their descendants.
#[derive(Clone, Copy)]
pub enum Order {
    /// Yield entries as their parent-directory reads complete.
    Completion,
    /// Yield every parent before its descendants.
    ParentFirst,
}

/// A filesystem entry produced by [`walk`].
#[cfg(not(windows))]
pub struct Entry {
    /// Distance from the walk root: `0` for the root, `1` for its children, and so on.
    pub depth: usize,
    /// File name relative to `parent_path`.
    pub file_name: OsString,
    /// Filesystem entry type without following symbolic links.
    pub file_type: FileType,
    /// Entry metadata, or the error encountered while reading it.
    pub metadata: io::Result<Metadata>,
    /// Path containing this entry.
    pub parent_path: Arc<Path>,
}

enum Job {
    /// Read a directory and schedule processing of its entries.
    ReadDir {
        root_idx: usize,
        path: Arc<Path>,
        /// Depth to be assigned to entries read from `path`; always at least `1`.
        /// The directory at `path` is one level shallower.
        entry_depth: usize,
    },
    /// Fetch metadata for a chunk of entries from a completed directory read.
    #[cfg(not(windows))]
    StatCompletion {
        root_idx: usize,
        path: Arc<Path>,
        /// Depth assigned to every entry in this chunk; always at least `1`, i.e. a file in a directory.
        entry_depth: usize,
        entries: Vec<fs::DirEntry>,
    },
}

impl Job {
    /// Return the index of the root path that this job belongs to.
    fn root_idx(&self) -> usize {
        match self {
            Job::ReadDir { root_idx, .. } => *root_idx,
            #[cfg(not(windows))]
            Job::StatCompletion { root_idx, .. } => *root_idx,
        }
    }
}

/// Internal worker-channel events, including batches, per-root completion, and pool completion.
enum Event {
    Batch {
        root_idx: usize,
        batch: Batch,
    },
    /// All work for this root is complete; emitted after all of its batches.
    /// Completion events for different roots may occur in any order.
    RootFinished {
        root_idx: usize,
    },
    /// Emitted once after all roots have emitted `RootFinished`; this is the final event.
    Finished,
}

/// Per-root events exposed by [`RootWalk`].
/// Unlike [`Event`], batches are flattened into entries and pool-wide completion ends the iterator
/// instead of being yielded; `Finished` therefore means only that the associated root completed.
/// [`RootWalk`] yields `(root_idx, event)`, separating root routing from event meaning. [`Event`]
/// cannot do this uniformly because its `Finished` variant is pool-wide and has no root index.
pub enum RootEvent {
    /// An entry or filesystem error produced while walking the root.
    Entry(io::Result<Entry>),
    /// All entries for the root have been emitted.
    Finished,
}

struct PoolShared {
    /// Global queue that makes the initial root job available to whichever worker starts first.
    injector: Injector<Job>,
    stealers: Vec<Stealer<Job>>,
    stop: AtomicBool,
    descend: Arc<Descend>,
    events: SyncSender<Event>,
    /// Number of roots with queued or running jobs.
    active_roots: AtomicUsize,
    /// Number of queued or running jobs for each root index.
    /// A counter reaching zero emits that root's [`Event::RootFinished`].
    jobs_per_root: HashMap<usize, AtomicUsize>,
    order: Order,
    /// Handles used to wake workers, indexed by worker number.
    unparkers: Vec<Unparker>,
    /// Whether each worker has announced that it is idle, indexed like `unparkers`.
    /// `wake_worker` atomically claims one idle worker before unparking it.
    idle: Vec<AtomicBool>,
    /// A round-robin cursor for the first idle worker to inspect.
    next_wake: AtomicUsize,
}

struct Pool {
    shared: Arc<PoolShared>,
    events: Receiver<Event>,
    handles: Vec<thread::JoinHandle<()>>,
}

/// A multi-root iterator yielding each root index with entry and per-root completion events.
/// Unlike [`Walk`], it preserves root identity and exposes when each root finishes.
pub struct RootWalk {
    /// Entries buffered for delivery, by root index.
    next: Vec<(usize, RootEvent)>,
    /// See [`Walk::pool`].
    pool: Option<Pool>,
}

/// A single-root directory iterator whose directory reads happen in parallel.
/// Unlike `RootWalk`, it yields entries directly and hides root identity and completion events.
pub struct Walk {
    /// Entries buffered for delivery.
    ///
    /// This vector is used as a stack: it starts with the root, and received batches are inserted
    /// in reverse so popping preserves their original order.
    ///
    /// If consumption isn't as fast as its production, threads will block.
    next: Vec<io::Result<Entry>>,
    /// Owns the worker threads for as long as traversal is active.
    ///
    /// Clearing or dropping it requests shutdown, unparks every worker, and joins their threads.
    pool: Option<Pool>,
}

/// Walk `root` without following symlinks.
/// Unlike `walk_roots`, this yields entries directly for a single root and hides
/// completion events.
pub fn walk(
    root: &Path,
    threads: usize,
    order: Order,
    descend: impl Fn(&Entry) -> bool + Send + Sync + 'static,
) -> Walk {
    let root = Entry::from_path(root);
    let pool = match &root {
        Ok(entry) if entry.file_type.is_dir() && descend(entry) => {
            let path = Arc::from(entry.path());
            let pool = start_pool(
                threads.max(1),
                HashMap::from([(0, AtomicUsize::new(0))]),
                order,
                Arc::new(move |_, entry| descend(entry)),
            );
            start_jobs(
                &pool,
                vec![Job::ReadDir {
                    root_idx: 0,
                    path,
                    entry_depth: 1,
                }],
            );
            Some(pool)
        }
        _ => None,
    };
    Walk {
        next: vec![root],
        pool,
    }
}

impl Iterator for Walk {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.next.pop() {
                return Some(entry);
            }

            match self.pool.as_ref()?.events.recv() {
                Ok(Event::Batch {
                    batch: Ok(entries), ..
                }) => {
                    self.next.extend(entries.into_iter().rev());
                }
                Ok(Event::Batch {
                    batch: Err(err), ..
                }) => return Some(Err(err)),
                Ok(Event::RootFinished { .. }) => {}
                Ok(Event::Finished) => {
                    self.pool = None;
                    return None;
                }
                Err(_) => return Some(Err(io::Error::other("directory worker stopped"))),
            }
        }
    }
}

/// Walk multiple indexed roots without following symlinks.
/// Unlike [`walk`], this preserves each root index and yields its completion as a [`RootEvent`].
///
/// # Panics
///
/// Panics if two roots have the same index.
pub fn walk_roots(
    roots: impl IntoIterator<Item = (usize, PathBuf)>,
    threads: usize,
    order: Order,
    descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
) -> RootWalk {
    let roots = roots.into_iter().collect::<Vec<_>>();
    let jobs_per_root = roots
        .iter()
        .map(|(root_idx, _)| (*root_idx, AtomicUsize::new(0)))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        jobs_per_root.len(),
        roots.len(),
        "root indices must be unique"
    );
    let descend = Arc::new(descend);
    let (next, root_jobs) = begin_walks(roots, descend.as_ref());
    let pool = if root_jobs.is_empty() {
        None
    } else {
        let pool = start_pool(threads.max(1), jobs_per_root, order, descend);
        start_jobs(&pool, root_jobs);
        Some(pool)
    };
    RootWalk { next, pool }
}

impl Iterator for RootWalk {
    type Item = (usize, RootEvent);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.next.pop() {
                return Some(entry);
            }
            match self.pool.as_ref()?.events.recv() {
                Ok(Event::Batch {
                    root_idx,
                    batch: Ok(entries),
                }) => self.next.extend(
                    entries
                        .into_iter()
                        .rev()
                        .map(|entry| (root_idx, RootEvent::Entry(entry))),
                ),
                Ok(Event::Batch {
                    root_idx,
                    batch: Err(err),
                }) => return Some((root_idx, RootEvent::Entry(Err(err)))),
                Ok(Event::RootFinished { root_idx }) => {
                    return Some((root_idx, RootEvent::Finished));
                }
                Ok(Event::Finished) => {
                    self.pool = None;
                    return None;
                }
                Err(_) => {
                    return Some((
                        0,
                        RootEvent::Entry(Err(io::Error::other("directory worker stopped"))),
                    ));
                }
            }
        }
    }
}

impl PoolShared {
    /// Wake one worker that has announced it is idle.
    fn wake_worker(&self) {
        let len = self.idle.len();
        // This cursor only distributes scan starting points, so relaxed races affect fairness, not
        // correctness; the compare-exchange below exclusively claims the worker to wake.
        let start = self.next_wake.fetch_add(1, AtomicOrdering::Relaxed) % len;
        for offset in 0..len {
            let idx = (start + offset) % len;
            if self.idle[idx]
                .compare_exchange(true, false, AtomicOrdering::AcqRel, AtomicOrdering::Relaxed)
                .is_ok()
            {
                self.unparkers[idx].unpark();
                break;
            }
        }
    }

    /// Wake all threads unconditionally.
    fn wake_workers(&self) {
        for unparker in &self.unparkers {
            unparker.unpark();
        }
    }
}

#[cfg(not(windows))]
impl Entry {
    /// Return the full path to this entry.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.parent_path.join(&self.file_name)
    }

    /// Create an entry from a filesystem path.
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        Ok(Self {
            depth: 0,
            file_name: path.file_name().unwrap_or(path.as_os_str()).to_owned(),
            file_type: metadata.file_type(),
            metadata: Ok(metadata),
            parent_path: Arc::from(path.parent().unwrap_or(Path::new(""))),
        })
    }

    fn from_dir_entry(
        depth: usize,
        parent_path: Arc<Path>,
        entry: fs::DirEntry,
    ) -> io::Result<Self> {
        Ok(Self {
            depth,
            file_name: entry.file_name(),
            file_type: entry.file_type()?,
            metadata: entry.metadata(),
            parent_path,
        })
    }
}

fn start_pool(
    threads: usize,
    jobs_per_root: HashMap<usize, AtomicUsize>,
    order: Order,
    descend: Arc<Descend>,
) -> Pool {
    let workers: Vec<_> = (0..threads).map(|_| Worker::new_lifo()).collect();
    let parkers: Vec<_> = (0..threads).map(|_| Parker::new()).collect();
    let (event_tx, event_rx) = sync_channel(threads * 2);
    let shared = Arc::new(PoolShared {
        injector: Injector::new(),
        stealers: workers.iter().map(Worker::stealer).collect(),
        stop: AtomicBool::new(false),
        descend,
        events: event_tx,
        active_roots: AtomicUsize::new(0),
        jobs_per_root,
        order,
        unparkers: parkers
            .iter()
            .map(|parker| parker.unparker().clone())
            .collect(),
        idle: (0..threads).map(|_| AtomicBool::new(false)).collect(),
        next_wake: AtomicUsize::new(0),
    });
    let handles: Vec<_> = workers
        .into_iter()
        .zip(parkers)
        .enumerate()
        .map(|(idx, (worker, parker))| {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name(format!("dua-fs-walk-{idx}"))
                .spawn(move || worker_loop(idx, worker, parker, shared))
                .expect("filesystem worker thread can be spawned")
        })
        .collect();

    Pool {
        shared,
        events: event_rx,
        handles,
    }
}

/// Prepare initial root events and directory jobs.
/// Returns events in stack order for [`RootWalk::next`] to pop, plus jobs requiring a worker pool.
fn begin_walks(
    roots: impl IntoIterator<Item = (usize, PathBuf)>,
    descend: &Descend,
) -> (Vec<(usize, RootEvent)>, Vec<Job>) {
    let mut next = Vec::new();
    let mut jobs = Vec::new();
    for (root_idx, path) in roots {
        let entry = Entry::from_path(&path);
        let has_job = if let Ok(entry) = &entry
            && entry.file_type.is_dir()
            && descend(root_idx, entry)
        {
            jobs.push(Job::ReadDir {
                root_idx,
                path: Arc::from(entry.path()),
                entry_depth: 1,
            });
            true
        } else {
            false
        };
        next.push((root_idx, RootEvent::Entry(entry)));
        if !has_job {
            next.push((root_idx, RootEvent::Finished));
        }
    }
    next.reverse();
    (next, jobs)
}

/// Seed an idle pool with one initial job per active root.
/// Initializes per-root completion accounting, queues the jobs, and wakes workers to process them.
fn start_jobs(pool: &Pool, root_jobs: Vec<Job>) {
    let wake_all = root_jobs.len() > 1;
    debug_assert_eq!(
        pool.shared.active_roots.load(AtomicOrdering::Relaxed),
        0,
        "initial jobs must be started on an idle pool"
    );
    debug_assert!(
        root_jobs.iter().all(|j| match j {
            Job::ReadDir { entry_depth, .. } => *entry_depth,
            #[cfg(not(windows))]
            Job::StatCompletion { entry_depth, .. } => *entry_depth,
        } == 1),
        "the first jobs should be root jobs, so active_root counts match"
    );
    pool.shared
        .active_roots
        .store(root_jobs.len(), AtomicOrdering::Relaxed);
    for job in &root_jobs {
        add_pending(job.root_idx(), 1, &pool.shared);
    }
    for job in root_jobs {
        pool.shared.injector.push(job);
    }
    if wake_all {
        pool.shared.wake_workers();
    } else {
        pool.shared.wake_worker();
    }
}

fn worker_loop(idx: usize, worker: Worker<Job>, parker: Parker, shared: Arc<PoolShared>) {
    while !shared.stop.load(AtomicOrdering::Relaxed) {
        let found = if let Some(found) = find_job(&worker, &shared) {
            found
        } else {
            shared.idle[idx].store(true, AtomicOrdering::Release);
            let Some(found) = find_job(&worker, &shared) else {
                parker.park();
                shared.idle[idx].store(false, AtomicOrdering::Release);
                continue;
            };
            shared.idle[idx].store(false, AtomicOrdering::Release);
            found
        };
        let (job, stolen) = found;
        if stolen {
            // A successful steal proves peer work is available; wake one more worker so
            // concurrency ramps up only while work remains stealable.
            shared.wake_worker();
        }
        run_job(job, &worker, &shared);
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.shared.stop.store(true, AtomicOrdering::Relaxed);
        self.shared.wake_workers();
        for handle in self.handles.drain(..) {
            handle.join().ok();
        }
    }
}

/// Find work in order of increasing synchronization cost.
///
/// The worker checks its own LIFO queue first, favoring locality and avoiding
/// shared-queue contention. It next takes a batch from the injector, keeping one job and moving
/// the rest into its local queue. Only then does it inspect other workers, because stealing from a
/// peer is the most contentious path. Consequently, a worker with local jobs keeps processing
/// them before helping elsewhere, and injector jobs take priority over peer jobs.
///
/// Returns the selected job and whether it was stolen from another worker; the caller uses a
/// successful steal to wake another idle worker. Returns `None` when a full scan finds no work.
fn find_job(worker: &Worker<Job>, shared: &PoolShared) -> Option<(Job, bool)> {
    loop {
        if let Some(job) = worker.pop() {
            return Some((job, false));
        }

        match shared.injector.steal_batch_and_pop(worker) {
            Steal::Success(job) => return Some((job, false)),
            Steal::Retry => continue,
            Steal::Empty => {}
        }

        let mut retry = false;
        for stealer in &shared.stealers {
            match stealer.steal() {
                Steal::Success(job) => return Some((job, true)),
                Steal::Retry => retry = true,
                Steal::Empty => {}
            }
        }
        if !retry {
            return None;
        }
    }
}

fn run_job(job: Job, worker: &Worker<Job>, shared: &PoolShared) {
    match job {
        Job::ReadDir {
            root_idx: root,
            path,
            entry_depth,
        } => {
            if matches!(shared.order, Order::Completion) {
                read_dir_completion(root, path, entry_depth, worker, shared);
            } else {
                read_dir_parent_first(root, path, entry_depth, worker, shared);
            }
        }
        #[cfg(not(windows))]
        Job::StatCompletion {
            root_idx: root,
            path,
            entry_depth,
            entries,
        } => stat_entries_completion(root, path, entry_depth, entries, worker, shared),
    }
}

/// Read a directory for completion-order traversal.
/// Successful directory entries are split into stealable metadata jobs, while enumeration errors
/// are emitted directly; the directory-read job completes after all chunks are queued.
/// This adds parallelism within wide directories when metadata calls dominate. Both traversal
/// orders already process separate directories concurrently, so typical trees may see no speedup.
#[cfg(not(windows))]
fn read_dir_completion(
    root_idx: usize,
    path: Arc<Path>,
    entry_depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let dir_entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            if shared
                .events
                .send(Event::Batch {
                    root_idx,
                    batch: Err(err),
                })
                .is_err()
            {
                shared.stop.store(true, AtomicOrdering::Relaxed);
            }
            finish_pending(root_idx, shared);
            return;
        }
    };
    let mut chunk = Vec::with_capacity(ENTRY_CHUNK_SIZE);
    let mut errors = Vec::new();
    let mut has_jobs = false;
    for entry in dir_entries {
        match entry {
            Ok(entry) => {
                chunk.push(entry);
                if chunk.len() == ENTRY_CHUNK_SIZE {
                    add_pending(root_idx, 1, shared);
                    worker.push(Job::StatCompletion {
                        root_idx,
                        path: Arc::clone(&path),
                        entry_depth,
                        entries: std::mem::replace(
                            &mut chunk,
                            Vec::with_capacity(ENTRY_CHUNK_SIZE),
                        ),
                    });
                    has_jobs = true;
                }
            }
            Err(err) => errors.push(Err(err)),
        }
    }
    if !chunk.is_empty() {
        add_pending(root_idx, 1, shared);
        worker.push(Job::StatCompletion {
            root_idx,
            path,
            entry_depth,
            entries: chunk,
        });
        has_jobs = true;
    }
    if has_jobs {
        shared.wake_worker();
    }
    if !errors.is_empty()
        && shared
            .events
            .send(Event::Batch {
                root_idx,
                batch: Ok(errors),
            })
            .is_err()
    {
        shared.stop.store(true, AtomicOrdering::Relaxed);
    }
    finish_pending(root_idx, shared);
}

/// Read a directory for completion-order traversal.
///
/// Unlike the non-Windows implementation, `windows::ReadDir` collects metadata while enumerating,
/// so complete entries are published directly in chunks instead of being split into stealable
/// metadata jobs. This streams wide directories but keeps their metadata work on one worker.
#[cfg(windows)]
fn read_dir_completion(
    root_idx: usize,
    path: Arc<Path>,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let dir_entries = match windows::ReadDir::open(path, depth) {
        Ok(entries) => entries,
        Err(err) => {
            if shared
                .events
                .send(Event::Batch {
                    root_idx,
                    batch: Err(err),
                })
                .is_err()
            {
                shared.stop.store(true, AtomicOrdering::Relaxed);
            }
            finish_pending(root_idx, shared);
            return;
        }
    };
    let mut entries = Vec::with_capacity(ENTRY_CHUNK_SIZE);
    let mut jobs = Vec::new();
    for entry in dir_entries {
        if let Ok(entry) = &entry
            && entry.file_type.is_dir()
            && (shared.descend)(root_idx, entry)
        {
            jobs.push(Job::ReadDir {
                root_idx,
                path: Arc::from(entry.path()),
                entry_depth: depth + 1,
            });
        }
        entries.push(entry);
        if entries.len() == ENTRY_CHUNK_SIZE
            && !publish_completion_batch(root_idx, &mut entries, &mut jobs, worker, shared)
        {
            finish_pending(root_idx, shared);
            return;
        }
    }
    if !entries.is_empty() {
        publish_completion_batch(root_idx, &mut entries, &mut jobs, worker, shared);
    }
    finish_pending(root_idx, shared);
}

#[cfg(windows)]
fn publish_completion_batch(
    root_idx: usize,
    entries: &mut Vec<io::Result<Entry>>,
    jobs: &mut Vec<Job>,
    worker: &Worker<Job>,
    shared: &PoolShared,
) -> bool {
    add_pending(root_idx, jobs.len(), shared);
    schedule_jobs(std::mem::take(jobs), worker, shared);
    if shared
        .events
        .send(Event::Batch {
            root_idx,
            batch: Ok(std::mem::replace(
                entries,
                Vec::with_capacity(ENTRY_CHUNK_SIZE),
            )),
        })
        .is_err()
    {
        shared.stop.store(true, AtomicOrdering::Relaxed);
        false
    } else {
        true
    }
}

#[cfg(windows)]
fn read_dir_parent_first(
    root_idx: usize,
    path: Arc<Path>,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let dir_entries = match windows::ReadDir::open(path, depth) {
        Ok(entries) => entries,
        Err(err) => {
            finish_directory(root_idx, Err(err), Vec::new(), worker, shared);
            return;
        }
    };
    let mut jobs = Vec::new();
    let entries = dir_entries
        .map(|entry| {
            entry.inspect(|entry| {
                if entry.file_type.is_dir() && (shared.descend)(root_idx, entry) {
                    jobs.push(Job::ReadDir {
                        root_idx,
                        path: Arc::from(entry.path()),
                        entry_depth: depth + 1,
                    });
                }
            })
        })
        .collect();
    finish_directory(root_idx, Ok(entries), jobs, worker, shared);
}

#[cfg(not(windows))]
fn stat_entries_completion(
    root_idx: usize,
    path: Arc<Path>,
    depth: usize,
    entries: Vec<fs::DirEntry>,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let mut jobs = Vec::new();
    let entries = entries
        .into_iter()
        .map(|entry| {
            Entry::from_dir_entry(depth, Arc::clone(&path), entry).inspect(|entry| {
                if entry.file_type.is_dir() && (shared.descend)(root_idx, entry) {
                    jobs.push(Job::ReadDir {
                        root_idx,
                        path: Arc::from(entry.path()),
                        entry_depth: entry.depth + 1,
                    });
                }
            })
        })
        .collect();
    add_pending(root_idx, jobs.len(), shared);
    schedule_jobs(jobs, worker, shared);
    if shared
        .events
        .send(Event::Batch {
            root_idx,
            batch: Ok(entries),
        })
        .is_err()
    {
        shared.stop.store(true, AtomicOrdering::Relaxed);
    }
    finish_pending(root_idx, shared);
}

/// Read a directory for parent-first traversal.
/// Entries are converted inline rather than scheduled as `StatCompletion` jobs, producing the
/// complete parent batch and its child-directory jobs together. This lets `finish_directory` send
/// the parent batch before making any child job available, preserving parent-before-descendant
/// order. Metadata within one directory is serial, although separate directories still run in
/// parallel; this often matches completion-order performance unless wide-directory metadata is the
/// bottleneck.
#[cfg(not(windows))]
fn read_dir_parent_first(
    root_idx: usize,
    path: Arc<Path>,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    read_dir_inline(root_idx, path, depth, worker, shared);
}

/// Convert a directory's entries on the worker that enumerates it, then schedule its children.
///
/// Parent-first traversal converts each entry inline to preserve ordering.
#[cfg(not(windows))]
fn read_dir_inline(
    root_idx: usize,
    path: Arc<Path>,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let dir_entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            finish_directory(root_idx, Err(err), Vec::new(), worker, shared);
            return;
        }
    };
    let mut jobs = Vec::new();
    let entries = dir_entries
        .map(|entry| {
            entry
                .and_then(|entry| Entry::from_dir_entry(depth, Arc::clone(&path), entry))
                .inspect(|entry| {
                    if entry.file_type.is_dir() && (shared.descend)(root_idx, entry) {
                        jobs.push(Job::ReadDir {
                            root_idx,
                            path: Arc::from(entry.path()),
                            entry_depth: depth + 1,
                        });
                    }
                })
        })
        .collect();
    finish_directory(root_idx, Ok(entries), jobs, worker, shared);
}

/// Publish a completed directory read and schedule its accepted child-directory jobs.
/// `ParentFirst` sends the batch before exposing child jobs; `Completion` exposes child jobs first.
/// Child jobs are counted before either action, and the current job is marked complete afterward.
fn finish_directory(
    root_idx: usize,
    batch: Batch,
    jobs: Vec<Job>,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    add_pending(root_idx, jobs.len(), shared);

    match shared.order {
        Order::ParentFirst => {
            if shared
                .events
                .send(Event::Batch { root_idx, batch })
                .is_err()
            {
                shared.stop.store(true, AtomicOrdering::Relaxed);
                return;
            }
            schedule_jobs(jobs, worker, shared);
        }
        Order::Completion => {
            schedule_jobs(jobs, worker, shared);
            if shared
                .events
                .send(Event::Batch { root_idx, batch })
                .is_err()
            {
                shared.stop.store(true, AtomicOrdering::Relaxed);
                return;
            }
        }
    }

    finish_pending(root_idx, shared);
}

fn add_pending(root: usize, count: usize, shared: &PoolShared) {
    shared.jobs_per_root[&root].fetch_add(count, AtomicOrdering::Relaxed);
}

/// Mark one job complete for `root`.
/// The last job emits `RootFinished`; if this was also the last active root, `Finished` follows.
fn finish_pending(root_idx: usize, shared: &PoolShared) {
    if shared.jobs_per_root[&root_idx].fetch_sub(1, AtomicOrdering::Relaxed) == 1 {
        shared.events.send(Event::RootFinished { root_idx }).ok();
        if shared.active_roots.fetch_sub(1, AtomicOrdering::Relaxed) == 1 {
            shared.events.send(Event::Finished).ok();
        }
    }
}

fn schedule_jobs(jobs: Vec<Job>, worker: &Worker<Job>, shared: &PoolShared) {
    let has_jobs = !jobs.is_empty();
    for job in jobs {
        worker.push(job);
    }
    if has_jobs {
        shared.wake_worker();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_walk_is_parent_first_and_does_not_follow_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("b/child")).unwrap();
        fs::create_dir(dir.path().join("a")).unwrap();
        fs::write(dir.path().join("b/child/file"), b"x").unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.path().join("b"), dir.path().join("link")).unwrap();

        #[cfg(unix)]
        let expected = ["", "a", "b", "b/child", "b/child/file", "link"];
        #[cfg(not(unix))]
        let expected = ["", "a", "b", "b/child", "b/child/file"];
        let expected = expected.into_iter().map(PathBuf::from).collect::<Vec<_>>();

        for threads in [1, 4] {
            let paths = walk(dir.path(), threads, Order::ParentFirst, |_| true)
                .map(|entry| {
                    entry
                        .unwrap()
                        .path()
                        .strip_prefix(dir.path())
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<_>>();
            let mut sorted_paths = paths.clone();
            sorted_paths.sort();
            assert_eq!(
                sorted_paths, expected,
                "walk with {threads} threads should visit every expected path exactly once"
            );

            for path in paths.iter().filter(|path| path.components().count() > 1) {
                let parent = path.parent().unwrap();
                assert!(
                    paths.iter().position(|path| path == parent)
                        < paths.iter().position(|candidate| candidate == path),
                    "parent {parent:?} should precede child {path:?} with {threads} threads; \
                     traversal order: {paths:?}"
                );
            }
        }
    }

    #[test]
    fn pruning_keeps_the_directory_and_missing_roots_are_errors() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("skip/child")).unwrap();

        let paths = walk(dir.path(), 2, Order::Completion, |entry| {
            entry.file_name != "skip"
        })
        .map(|entry| entry.unwrap().file_name)
        .collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec![
                dir.path().file_name().unwrap().to_owned(),
                OsString::from("skip")
            ],
            "a pruned directory should be yielded without traversing its children"
        );

        assert!(
            walk(&dir.path().join("missing"), 2, Order::Completion, |_| true)
                .next()
                .unwrap()
                .is_err(),
            "a missing root should be yielded as an I/O error"
        );
    }

    #[test]
    fn concurrent_roots_keep_their_identity() {
        let dir = tempfile::tempdir().unwrap();
        let roots = [dir.path().join("a"), dir.path().join("b")];
        for root in &roots {
            fs::create_dir_all(root.join("child")).unwrap();
        }

        let events = walk_roots(
            roots.iter().cloned().enumerate(),
            2,
            Order::Completion,
            |_, _| true,
        )
        .collect::<Vec<_>>();
        let mut paths = Vec::new();
        let mut last_entry = [0; 2];
        let mut finished = [None; 2];
        for (position, (root_idx, event)) in events.into_iter().enumerate() {
            match event {
                RootEvent::Entry(entry) => {
                    last_entry[root_idx] = position;
                    paths.push((
                        root_idx,
                        entry
                            .unwrap()
                            .path()
                            .strip_prefix(&roots[root_idx])
                            .unwrap()
                            .to_owned(),
                    ));
                }
                RootEvent::Finished => finished[root_idx] = Some(position),
            }
        }
        paths.sort();
        assert_eq!(
            paths,
            [
                (0, PathBuf::new()),
                (0, PathBuf::from("child")),
                (1, PathBuf::new()),
                (1, PathBuf::from("child")),
            ]
        );
        for root_idx in 0..roots.len() {
            assert!(
                last_entry[root_idx] < finished[root_idx].unwrap(),
                "root {root_idx} must finish after its last entry",
            );
        }
    }

    #[test]
    fn wide_walk_wakes_multiple_idle_workers() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..32 {
            fs::create_dir_all(dir.path().join(format!("{idx}/child"))).unwrap();
        }

        let worker_threads = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let seen_threads = Arc::clone(&worker_threads);
        walk(dir.path(), 8, Order::Completion, move |entry| {
            if entry.depth == 1 {
                thread::sleep(std::time::Duration::from_millis(1));
            } else if entry.depth == 2 {
                seen_threads.lock().unwrap().insert(thread::current().id());
                thread::sleep(std::time::Duration::from_millis(10));
            }
            true
        })
        .for_each(drop);

        assert!(
            worker_threads.lock().unwrap().len() >= 4,
            "a wide directory should engage more than the producer and one thief"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_metadata_is_collected_by_the_directory_worker() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..32 {
            fs::create_dir(dir.path().join(idx.to_string())).unwrap();
        }

        let worker_threads = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let seen_threads = Arc::clone(&worker_threads);
        walk(dir.path(), 8, Order::Completion, move |entry| {
            if entry.depth == 1 {
                seen_threads.lock().unwrap().insert(thread::current().id());
                thread::sleep(std::time::Duration::from_millis(2));
            }
            true
        })
        .for_each(drop);

        assert_eq!(
            worker_threads.lock().unwrap().len(),
            1,
            "Windows directory-entry metadata should stay on the enumerating worker"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_completion_streams_metadata_before_enumeration_finishes() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..=ENTRY_CHUNK_SIZE {
            fs::create_dir(dir.path().join(idx.to_string())).unwrap();
        }

        let (continue_tx, continue_rx) = std::sync::mpsc::sync_channel(0);
        let continue_rx = Arc::new(std::sync::Mutex::new(continue_rx));
        let seen = Arc::new(AtomicUsize::new(0));
        let seen_in_worker = Arc::clone(&seen);
        let mut entries = walk(dir.path(), 2, Order::Completion, move |entry| {
            if entry.depth == 1
                && seen_in_worker.fetch_add(1, AtomicOrdering::Relaxed) == ENTRY_CHUNK_SIZE
            {
                continue_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(std::time::Duration::from_secs(2))
                    .expect("the first metadata batch should arrive before enumeration finishes");
            }
            true
        });

        assert_eq!(
            entries.next().unwrap().unwrap().depth,
            0,
            "the root entry should be yielded first"
        );
        assert_eq!(
            entries.next().unwrap().unwrap().depth,
            1,
            "the first metadata batch should be yielded before enumeration resumes"
        );
        continue_tx.send(()).unwrap();
        entries.for_each(drop);
        assert_eq!(
            seen.load(AtomicOrdering::Relaxed),
            ENTRY_CHUNK_SIZE + 1,
            "all directory entries should be inspected"
        );
    }
}
