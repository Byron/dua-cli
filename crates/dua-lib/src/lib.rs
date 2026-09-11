//! Parallel filesystem traversal backed by a work-stealing worker pool.
//!
//! [`walk`] yields the root first, then workers read directories and distribute newly discovered
//! subdirectories among themselves. [`Order::ParentFirst`] publishes each entry's batch
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
//! metadata batches enqueue accepted child directories. Windows and macOS walks consume native
//! metadata returned by directory enumeration instead. Multi-threaded macOS walks probe the
//! initial bulk refills and distribute metadata lookups when those reads spend time waiting.
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
    fs, io,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread,
};

#[cfg(any(not(any(windows, target_os = "macos")), test))]
use std::ffi::OsString;

#[cfg(not(any(windows, target_os = "macos")))]
pub use std::fs::{FileType, Metadata};

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos;

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::{Entry, FileType, Metadata};

#[cfg(target_os = "macos")]
use macos::ReadDir as NativeReadDir;

#[cfg(target_os = "macos")]
use std::fs::read_dir as read_dir_types;

#[cfg(windows)]
pub use windows::{Entry, FileType, Metadata};

#[cfg(windows)]
use windows::{ReadDir as NativeReadDir, read_dir_types};

#[cfg(any(windows, target_os = "macos"))]
enum ReadDir {
    Metadata(NativeReadDir),
    FileTypes {
        entries: fs::ReadDir,
        parent_path: Arc<Path>,
        depth: usize,
    },
}

#[cfg(any(windows, target_os = "macos"))]
impl ReadDir {
    fn open(path: Arc<Path>, depth: usize, options: Options) -> io::Result<Self> {
        if options.skip_metadata {
            Ok(Self::FileTypes {
                entries: read_dir_types(&path)?,
                parent_path: path,
                depth,
            })
        } else {
            NativeReadDir::open(path, depth, options).map(Self::Metadata)
        }
    }
}

#[cfg(any(windows, target_os = "macos"))]
impl Iterator for ReadDir {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Metadata(reader) => reader.next(),
            Self::FileTypes {
                entries,
                parent_path,
                depth,
            } => entries.next().map(|entry| {
                let entry = entry?;
                Ok(Entry {
                    depth: *depth,
                    file_name: entry.file_name(),
                    file_type: FileType::from_std(entry.file_type()?),
                    metadata: None,
                    parent_path: Arc::clone(parent_path),
                    directory_id: None,
                    parent_directory_id: None,
                })
            }),
        }
    }
}

/// Decides whether to traverse an entry's children for a given root index.
/// Returning `false` prunes descendants but still emits the entry itself.
type Descend = dyn Fn(usize, &Entry) -> bool + Send + Sync;
/// Entries obtained from one directory read.
/// An outer error means the directory could not be opened; inner errors come from reading or
/// converting individual directory entries.
type Batch = io::Result<Vec<io::Result<Entry>>>;
/// Number of directory entries grouped into each metadata job or result batch.
/// Small chunks expose parallel work and stream wide directories while amortizing queue overhead.
const ENTRY_CHUNK_SIZE: usize = 4;
/// Keep enough metadata jobs available for thieves without retaining an entire wide directory.
#[cfg(not(windows))]
const MAX_QUEUED_STAT_JOBS: usize = 64;
#[cfg(target_os = "macos")]
type StatEntry = Entry;
#[cfg(not(any(windows, target_os = "macos")))]
type StatEntry = fs::DirEntry;

/// Controls when entries are yielded relative to their descendants.
#[derive(Clone, Copy)]
pub enum Order {
    /// Yield entries as their parent-directory reads complete.
    Completion,
    /// Yield every parent before its descendants.
    ParentFirst,
}

/// Filesystem metadata requested during traversal.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// Collect only entry types, leaving [`Entry::metadata`] as `None`.
    ///
    /// Directory enumeration supplies types when available. Explicit roots and filesystems
    /// without directory-entry types may still require a metadata lookup. This also disables
    /// APFS clone metadata collection.
    pub skip_metadata: bool,
    /// Collect APFS clone identity and data-fork allocation metadata.
    #[cfg(target_os = "macos")]
    pub apfs_clone_metadata: bool,
}

impl Options {
    /// Collect only entry types, leaving [`Entry::metadata`] as `None`.
    #[must_use]
    pub fn skip_metadata(mut self) -> Self {
        self.skip_metadata = true;
        self
    }
}

/// Dense identifier of a directory within one filesystem walk.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DirectoryId(NonZeroU32);

impl DirectoryId {
    fn new(index: usize) -> Self {
        let stored = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .and_then(NonZeroU32::new)
            .expect("directory identifier overflow");
        Self(stored)
    }

    /// Return the zero-based identifier for indexing compact side tables.
    #[must_use]
    pub fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

/// A filesystem entry produced by [`walk`].
#[cfg(not(any(windows, target_os = "macos")))]
pub struct Entry {
    /// Distance from the walk root: `0` for the root, `1` for its children, and so on.
    pub depth: usize,
    /// File name relative to `parent_path`.
    pub file_name: OsString,
    /// Filesystem entry type without following symbolic links.
    pub file_type: FileType,
    /// Requested metadata or its read error; `None` when [`Options::skip_metadata`] is set.
    pub metadata: Option<io::Result<Metadata>>,
    /// Path containing this entry.
    pub parent_path: Arc<Path>,
    /// Dense identifier of this directory within the current walk, or `None` for non-directories.
    pub directory_id: Option<DirectoryId>,
    /// Dense identifier of the directory containing this entry, or `None` for a walk root.
    pub parent_directory_id: Option<DirectoryId>,
}

enum Job {
    /// Read a directory and schedule processing of its entries.
    ReadDir {
        root_idx: usize,
        path: Arc<Path>,
        /// Dense identifier of the directory being read.
        directory_id: usize,
        /// Depth to be assigned to entries read from `path`; always at least `1`.
        /// The directory at `path` is one level shallower.
        entry_depth: usize,
    },
    /// Fetch metadata for a chunk of entries from a completed directory read.
    #[cfg(not(windows))]
    Stat {
        root_idx: usize,
        path: Arc<Path>,
        /// Dense identifier of the directory containing these entries.
        directory_id: usize,
        /// Depth assigned to every entry in this chunk; always at least `1`, i.e. a file in a directory.
        entry_depth: usize,
        entries: Vec<StatEntry>,
    },
}

impl Job {
    /// Return the index of the root path that this job belongs to.
    fn root_idx(&self) -> usize {
        match self {
            Job::ReadDir { root_idx, .. } => *root_idx,
            #[cfg(not(windows))]
            Job::Stat { root_idx, .. } => *root_idx,
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
    options: Options,
    /// Handles used to wake workers, indexed by worker number.
    unparkers: Vec<Unparker>,
    /// Whether each worker has announced that it is idle, indexed like `unparkers`.
    /// `wake_worker` atomically claims one idle worker before unparking it.
    idle: Vec<AtomicBool>,
    /// A round-robin cursor for the first idle worker to inspect.
    next_wake: AtomicUsize,
    /// Allocates dense identifiers to directories as they are discovered.
    next_directory_id: AtomicUsize,
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
    root: PathBuf,
    options: Options,
    /// Whether the worker pool has finished the current traversal.
    finished: bool,
}

/// Read a directory using native enumeration, collecting metadata unless
/// [`Options::skip_metadata`] is set.
///
/// Entries have depth zero so they can be passed directly to [`walk_root_entries`] without
/// querying their paths again. Directory-open errors are returned immediately; later enumeration
/// errors are yielded by the iterator.
#[cfg(any(windows, target_os = "macos"))]
pub fn read_dir(
    path: &Path,
    options: Options,
) -> io::Result<impl Iterator<Item = io::Result<Entry>>> {
    ReadDir::open(Arc::from(path), 0, options)
}

/// Walk `root` without following symlinks.
/// Unlike `walk_roots`, this yields entries directly for a single root and hides
/// completion events.
pub fn walk(
    root: &Path,
    threads: usize,
    order: Order,
    options: Options,
    descend: impl Fn(&Entry) -> bool + Send + Sync + 'static,
) -> Walk {
    let root_path = root.to_owned();
    let mut root = Entry::from_path(root, options);
    if let Ok(entry) = &mut root
        && entry.file_type.is_dir()
    {
        entry.directory_id = Some(DirectoryId::new(0));
    }
    let pool = match &root {
        Ok(entry) if entry.file_type.is_dir() && descend(entry) => {
            let path = Arc::from(entry.path());
            let pool = start_pool(
                threads.max(1),
                HashMap::from([(0, AtomicUsize::new(0))]),
                order,
                Arc::new(move |_, entry| descend(entry)),
                options,
                1,
            );
            start_jobs(
                &pool,
                vec![Job::ReadDir {
                    root_idx: 0,
                    path,
                    directory_id: 0,
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
        root: root_path,
        options,
        finished: false,
    }
}

impl Walk {
    /// Restart an exhausted directory walk while retaining its worker threads.
    ///
    /// Returns `false` if the walk is still active or did not start a worker pool.
    #[must_use]
    pub fn restart(&mut self) -> bool {
        if !self.finished || !self.next.is_empty() {
            return false;
        }
        let Some(pool) = self.pool.as_ref() else {
            return false;
        };
        let mut root = Entry::from_path(&self.root, self.options);
        if let Ok(entry) = &mut root
            && entry.file_type.is_dir()
        {
            entry.directory_id = Some(DirectoryId::new(0));
        }
        let job = match &root {
            Ok(entry) if entry.file_type.is_dir() && (pool.shared.descend)(0, entry) => {
                Some(Job::ReadDir {
                    root_idx: 0,
                    path: Arc::from(entry.path()),
                    directory_id: 0,
                    entry_depth: 1,
                })
            }
            _ => None,
        };
        self.next.push(root);
        if let Some(job) = job {
            pool.shared
                .next_directory_id
                .store(1, AtomicOrdering::Relaxed);
            self.finished = false;
            start_jobs(pool, vec![job]);
        }
        true
    }
}

impl Iterator for Walk {
    type Item = io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.next.pop() {
                return Some(entry);
            }
            if self.finished {
                return None;
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
                    self.finished = true;
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
/// Each item in `roots` is `(root_index, path)`. `root_index` is a caller-chosen identifier passed
/// to `descend` and returned with every [`RootEvent`] for that root, unique per root path.
///
/// # Panics
///
/// Panics if two roots have the same index.
pub fn walk_roots(
    roots: impl IntoIterator<Item = (usize, PathBuf)>,
    threads: usize,
    order: Order,
    options: Options,
    descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
) -> RootWalk {
    start_root_walk(
        roots.into_iter().collect(),
        threads,
        order,
        descend,
        |path: PathBuf| Entry::from_path(&path, options),
        options,
    )
}

/// Walk multiple indexed roots whose entries and metadata have already been collected.
///
/// Unlike [`walk_roots`], this reuses each supplied entry without querying its path again. Entry
/// errors are yielded for their corresponding root, and each root retains its index and completion
/// event just as it does with [`walk_roots`]. Supplied entries are re-rooted at depth zero before
/// the predicate runs, and their descendants start at depth one.
/// The supplied entries retain their metadata; `options` controls metadata for descendants.
///
/// # Panics
///
/// Panics if two roots have the same index.
pub fn walk_root_entries(
    roots: impl IntoIterator<Item = (usize, io::Result<Entry>)>,
    threads: usize,
    order: Order,
    options: Options,
    descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
) -> RootWalk {
    start_root_walk(
        roots.into_iter().collect(),
        threads,
        order,
        descend,
        std::convert::identity,
        options,
    )
}

fn start_root_walk<Root>(
    roots: Vec<(usize, Root)>,
    threads: usize,
    order: Order,
    descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
    prepare: impl Fn(Root) -> io::Result<Entry>,
    options: Options,
) -> RootWalk {
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
    let (next, root_jobs, next_directory_id) = begin_walks(
        roots
            .into_iter()
            .map(|(root_idx, root)| (root_idx, prepare(root))),
        descend.as_ref(),
    );
    let pool = if root_jobs.is_empty() {
        None
    } else {
        let pool = start_pool(
            threads.max(1),
            jobs_per_root,
            order,
            descend,
            options,
            next_directory_id,
        );
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

#[cfg(not(any(windows, target_os = "macos")))]
impl Entry {
    /// Return the full path to this entry.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.parent_path.join(&self.file_name)
    }

    /// Create an entry from a filesystem path.
    pub fn from_path(path: &Path, options: Options) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        Ok(Self {
            depth: 0,
            file_name: path.file_name().unwrap_or(path.as_os_str()).to_owned(),
            file_type: metadata.file_type(),
            metadata: (!options.skip_metadata).then_some(Ok(metadata)),
            parent_path: Arc::from(path.parent().unwrap_or(Path::new(""))),
            directory_id: None,
            parent_directory_id: None,
        })
    }

    fn from_dir_entry(
        depth: usize,
        parent_path: Arc<Path>,
        entry: fs::DirEntry,
        options: Options,
    ) -> io::Result<Self> {
        Ok(Self {
            depth,
            file_name: entry.file_name(),
            file_type: entry.file_type()?,
            metadata: (!options.skip_metadata).then(|| entry.metadata()),
            parent_path,
            directory_id: None,
            parent_directory_id: None,
        })
    }
}

fn start_pool(
    threads: usize,
    jobs_per_root: HashMap<usize, AtomicUsize>,
    order: Order,
    descend: Arc<Descend>,
    options: Options,
    next_directory_id: usize,
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
        options,
        unparkers: parkers
            .iter()
            .map(|parker| parker.unparker().clone())
            .collect(),
        idle: (0..threads).map(|_| AtomicBool::new(false)).collect(),
        next_wake: AtomicUsize::new(0),
        next_directory_id: AtomicUsize::new(next_directory_id),
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
    roots: impl IntoIterator<Item = (usize, io::Result<Entry>)>,
    descend: &Descend,
) -> (Vec<(usize, RootEvent)>, Vec<Job>, usize) {
    let mut next = Vec::new();
    let mut jobs = Vec::new();
    let mut next_directory_id = 0;
    for (root_idx, mut entry) in roots {
        if let Ok(entry) = &mut entry {
            entry.depth = 0;
            entry.parent_directory_id = None;
            if entry.file_type.is_dir() {
                entry.directory_id = Some(DirectoryId::new(next_directory_id));
                next_directory_id += 1;
            } else {
                entry.directory_id = None;
            }
        }
        let has_job = if let Ok(entry) = &entry
            && entry.metadata.as_ref().is_none_or(Result::is_ok)
            && entry.file_type.is_dir()
            && descend(root_idx, entry)
        {
            let directory_id = entry
                .directory_id
                .expect("directory roots receive an identifier");
            jobs.push(Job::ReadDir {
                root_idx,
                path: Arc::from(entry.path()),
                directory_id: directory_id.index(),
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
    (next, jobs, next_directory_id)
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
            Job::Stat { entry_depth, .. } => *entry_depth,
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
            directory_id,
            entry_depth,
        } => {
            #[cfg(any(windows, target_os = "macos"))]
            read_dir_native(root, path, directory_id, entry_depth, worker, shared);
            #[cfg(not(any(windows, target_os = "macos")))]
            if matches!(shared.order, Order::Completion) {
                read_dir_parallel(root, path, directory_id, entry_depth, worker, shared);
            } else {
                read_dir_parent_first(root, path, directory_id, entry_depth, worker, shared);
            }
        }
        #[cfg(not(windows))]
        Job::Stat {
            root_idx: root,
            path,
            directory_id,
            entry_depth,
            entries,
        } => stat_entries(
            root,
            path,
            directory_id,
            entry_depth,
            entries,
            worker,
            shared,
        ),
    }
}

/// Read a directory and distribute its metadata lookups among workers.
/// Successful entries are split into stealable metadata jobs; new chunks are processed inline
/// once the local queue is full. Enumeration errors are emitted directly.
/// This adds parallelism within wide directories when metadata calls dominate.
/// Type-only walks convert entries inline instead, avoiding metadata-job overhead.
#[cfg(not(any(windows, target_os = "macos")))]
fn read_dir_parallel(
    root_idx: usize,
    path: Arc<Path>,
    directory_id: usize,
    entry_depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    if shared.options.skip_metadata {
        read_dir_inline(root_idx, path, directory_id, entry_depth, worker, shared);
        return;
    }
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
    for entry in dir_entries {
        if shared.stop.load(AtomicOrdering::Relaxed) {
            finish_pending(root_idx, shared);
            return;
        }
        match entry {
            Ok(entry) => {
                chunk.push(entry);
                if chunk.len() == ENTRY_CHUNK_SIZE {
                    schedule_stat_entries(
                        root_idx,
                        &path,
                        directory_id,
                        entry_depth,
                        std::mem::replace(&mut chunk, Vec::with_capacity(ENTRY_CHUNK_SIZE)),
                        worker,
                        shared,
                    );
                }
            }
            Err(err) => errors.push(Err(err)),
        }
    }
    if !chunk.is_empty() {
        schedule_stat_entries(
            root_idx,
            &path,
            directory_id,
            entry_depth,
            chunk,
            worker,
            shared,
        );
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

/// Read a directory in either traversal order.
///
/// Native metadata is published directly in chunks. An initial macOS bulk probe can choose
/// ordinary directory enumeration with bounded parallel metadata jobs. Each parent-first batch
/// is sent before its child jobs become stealable.
#[cfg(any(windows, target_os = "macos"))]
fn read_dir_native(
    root_idx: usize,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let dir_entries = match ReadDir::open(Arc::clone(&path), depth, shared.options) {
        Ok(entries) => entries,
        Err(err) => {
            publish_directory(root_idx, Err(err), Vec::new(), worker, shared);
            finish_pending(root_idx, shared);
            return;
        }
    };
    #[cfg(target_os = "macos")]
    let dir_entries = {
        let mut dir_entries = dir_entries;
        let mut prefix = Vec::new();
        if shared.stealers.len() > 1
            && let ReadDir::Metadata(reader) = &mut dir_entries
        {
            prefix = reader.probe_metadata();
            if reader.metadata_is_io_bound()
                && let Ok(reopened) =
                    ReadDir::open(Arc::clone(&path), depth, shared.options.skip_metadata())
            {
                // No entries or child jobs have been published, so restarting cannot duplicate them.
                prefix.clear();
                dir_entries = reopened;
            }
        }
        prefix.into_iter().chain(dir_entries)
    };
    #[cfg(target_os = "macos")]
    let mut deferred = Vec::with_capacity(ENTRY_CHUNK_SIZE);
    let mut entries = Vec::with_capacity(ENTRY_CHUNK_SIZE);
    let mut jobs = Vec::new();
    for mut entry in dir_entries {
        if shared.stop.load(AtomicOrdering::Relaxed) {
            finish_pending(root_idx, shared);
            return;
        }
        #[cfg(target_os = "macos")]
        if !shared.options.skip_metadata
            && entry.as_ref().is_ok_and(|entry| entry.metadata.is_none())
        {
            deferred.push(entry.expect("metadata-free entry was checked"));
            if deferred.len() == ENTRY_CHUNK_SIZE {
                schedule_stat_entries(
                    root_idx,
                    &path,
                    directory_id,
                    depth,
                    std::mem::replace(&mut deferred, Vec::with_capacity(ENTRY_CHUNK_SIZE)),
                    worker,
                    shared,
                );
            }
            continue;
        }
        if let Ok(entry) = &mut entry {
            assign_directory_ids(entry, directory_id, shared);
        }
        if let Ok(entry) = &entry
            && entry.file_type.is_dir()
            && (shared.descend)(root_idx, entry)
        {
            jobs.push(Job::ReadDir {
                root_idx,
                path: Arc::from(entry.path()),
                directory_id: entry
                    .directory_id
                    .expect("directories receive an identifier")
                    .index(),
                entry_depth: depth + 1,
            });
        }
        entries.push(entry);
        if entries.len() == ENTRY_CHUNK_SIZE
            && !publish_directory(
                root_idx,
                Ok(std::mem::replace(
                    &mut entries,
                    Vec::with_capacity(ENTRY_CHUNK_SIZE),
                )),
                std::mem::take(&mut jobs),
                worker,
                shared,
            )
        {
            finish_pending(root_idx, shared);
            return;
        }
    }
    if !entries.is_empty() {
        publish_directory(root_idx, Ok(entries), jobs, worker, shared);
    }
    #[cfg(target_os = "macos")]
    if !deferred.is_empty() {
        schedule_stat_entries(
            root_idx,
            &path,
            directory_id,
            depth,
            deferred,
            worker,
            shared,
        );
    }
    finish_pending(root_idx, shared);
}

#[cfg(not(windows))]
fn schedule_stat_entries(
    root_idx: usize,
    path: &Arc<Path>,
    directory_id: usize,
    entry_depth: usize,
    entries: Vec<StatEntry>,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let job = Job::Stat {
        root_idx,
        path: Arc::clone(path),
        directory_id,
        entry_depth,
        entries,
    };
    add_pending(root_idx, 1, shared);
    if worker.len() >= MAX_QUEUED_STAT_JOBS {
        run_job(job, worker, shared);
    } else {
        worker.push(job);
        shared.wake_worker();
    }
}

#[cfg(not(windows))]
fn stat_entries(
    root_idx: usize,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    entries: Vec<StatEntry>,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    #[cfg(target_os = "macos")]
    let _ = (path, depth);
    let mut jobs = Vec::new();
    let entries = entries
        .into_iter()
        .map(|entry| {
            #[cfg(target_os = "macos")]
            let entry = Ok(entry.read_metadata(shared.options));
            #[cfg(not(target_os = "macos"))]
            let entry = Entry::from_dir_entry(depth, Arc::clone(&path), entry, shared.options);
            entry.map(|mut entry| {
                assign_directory_ids(&mut entry, directory_id, shared);
                if entry.file_type.is_dir() && (shared.descend)(root_idx, &entry) {
                    jobs.push(Job::ReadDir {
                        root_idx,
                        path: Arc::from(entry.path()),
                        directory_id: entry
                            .directory_id
                            .expect("directories receive an identifier")
                            .index(),
                        entry_depth: entry.depth + 1,
                    });
                }
                entry
            })
        })
        .collect();
    publish_directory(root_idx, Ok(entries), jobs, worker, shared);
    finish_pending(root_idx, shared);
}

/// Read a directory for parent-first traversal.
/// Entries are converted inline rather than scheduled as `Stat` jobs, producing the
/// complete parent batch and its child-directory jobs together. This lets `publish_directory` send
/// the parent batch before making any child job available, preserving parent-before-descendant
/// order. Metadata within one directory is serial, although separate directories still run in
/// parallel; this often matches completion-order performance unless wide-directory metadata is the
/// bottleneck.
#[cfg(not(any(windows, target_os = "macos")))]
fn read_dir_parent_first(
    root_idx: usize,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    read_dir_inline(root_idx, path, directory_id, depth, worker, shared);
}

/// Convert a directory's entries on the worker that enumerates it, then schedule its children.
///
/// Parent-first traversal converts each entry inline to preserve ordering.
#[cfg(not(any(windows, target_os = "macos")))]
fn read_dir_inline(
    root_idx: usize,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
) {
    let dir_entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            publish_directory(root_idx, Err(err), Vec::new(), worker, shared);
            finish_pending(root_idx, shared);
            return;
        }
    };
    let mut jobs = Vec::new();
    let entries = dir_entries
        .map(|entry| {
            entry
                .and_then(|entry| {
                    Entry::from_dir_entry(depth, Arc::clone(&path), entry, shared.options)
                })
                .map(|mut entry| {
                    assign_directory_ids(&mut entry, directory_id, shared);
                    if entry.file_type.is_dir() && (shared.descend)(root_idx, &entry) {
                        jobs.push(Job::ReadDir {
                            root_idx,
                            path: Arc::from(entry.path()),
                            directory_id: entry
                                .directory_id
                                .expect("directories receive an identifier")
                                .index(),
                            entry_depth: depth + 1,
                        });
                    }
                    entry
                })
        })
        .collect();
    publish_directory(root_idx, Ok(entries), jobs, worker, shared);
    finish_pending(root_idx, shared);
}

fn assign_directory_ids(entry: &mut Entry, parent_directory_id: usize, shared: &PoolShared) {
    entry.parent_directory_id = Some(DirectoryId::new(parent_directory_id));
    entry.directory_id = entry.file_type.is_dir().then(|| {
        DirectoryId::new(
            shared
                .next_directory_id
                .fetch_update(AtomicOrdering::Relaxed, AtomicOrdering::Relaxed, |id| {
                    id.checked_add(1)
                })
                .expect("directory identifier overflow"),
        )
    });
}

/// Publish a directory batch and schedule its accepted child-directory jobs.
/// `ParentFirst` sends the batch before exposing child jobs; `Completion` exposes child jobs first.
/// Child jobs are counted before either action; the caller completes the current job after EOF.
///
/// Returns `true` if the batch was sent and child jobs were scheduled, or `false` if the event
/// receiver disconnected, in which case traversal is marked to stop.
fn publish_directory(
    root_idx: usize,
    batch: Batch,
    jobs: Vec<Job>,
    worker: &Worker<Job>,
    shared: &PoolShared,
) -> bool {
    add_pending(root_idx, jobs.len(), shared);

    match shared.order {
        Order::ParentFirst => {
            if shared
                .events
                .send(Event::Batch { root_idx, batch })
                .is_err()
            {
                shared.stop.store(true, AtomicOrdering::Relaxed);
                return false;
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
                return false;
            }
        }
    }

    true
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
    fn directory_ids_are_compact_dense_and_match_parents() {
        assert_eq!(size_of::<Option<DirectoryId>>(), 4);

        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/child")).unwrap();
        fs::create_dir(dir.path().join("b")).unwrap();
        fs::write(dir.path().join("a/file"), b"x").unwrap();
        fs::write(dir.path().join("file"), b"x").unwrap();

        let entries = walk(
            dir.path(),
            4,
            Order::ParentFirst,
            Options::default(),
            |_| true,
        )
        .map(Result::unwrap)
        .collect::<Vec<_>>();
        let directories = entries
            .iter()
            .filter_map(|entry| entry.directory_id.map(|id| (entry.path(), id)))
            .collect::<HashMap<_, _>>();
        let mut ids = directories
            .values()
            .map(|id| id.index())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, (0..directories.len()).collect::<Vec<_>>());

        for entry in entries {
            assert_eq!(entry.directory_id.is_some(), entry.file_type.is_dir());
            if entry.depth == 0 {
                assert_eq!(entry.parent_directory_id, None);
            } else {
                assert_eq!(
                    entry.parent_directory_id,
                    directories.get(entry.path().parent().unwrap()).copied()
                );
            }
        }
    }

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
            let paths = walk(
                dir.path(),
                threads,
                Order::ParentFirst,
                Options::default(),
                |_| true,
            )
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
    fn a_completed_walk_can_reuse_its_workers() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("first/child")).unwrap();

        let mut walk = walk(dir.path(), 2, Order::Completion, Options::default(), |_| {
            true
        });
        assert!(!walk.restart(), "an active walk cannot be restarted");
        let worker_ids = walk
            .pool
            .as_ref()
            .unwrap()
            .handles
            .iter()
            .map(|handle| handle.thread().id())
            .collect::<Vec<_>>();
        walk.by_ref().for_each(drop);
        assert!(walk.next().is_none(), "a completed walk stays exhausted");

        fs::create_dir_all(dir.path().join("second/child")).unwrap();
        assert!(walk.restart());
        let paths = walk
            .by_ref()
            .map(|entry| {
                entry
                    .unwrap()
                    .path()
                    .strip_prefix(dir.path())
                    .unwrap()
                    .into()
            })
            .collect::<Vec<PathBuf>>();

        assert!(paths.contains(&PathBuf::from("second/child")));
        assert_eq!(
            walk.pool
                .as_ref()
                .unwrap()
                .handles
                .iter()
                .map(|handle| handle.thread().id())
                .collect::<Vec<_>>(),
            worker_ids,
            "a restarted walk keeps its original worker threads"
        );
    }

    #[test]
    fn pruning_keeps_the_directory_and_missing_roots_are_errors() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("skip/child")).unwrap();

        let paths = walk(
            dir.path(),
            2,
            Order::Completion,
            Options::default(),
            |entry| entry.file_name != "skip",
        )
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
            walk(
                &dir.path().join("missing"),
                2,
                Order::Completion,
                Options::default(),
                |_| true,
            )
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
            Options::default(),
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
    fn prepared_roots_rebase_existing_descendants() {
        let directory = tempfile::tempdir().unwrap();
        let descendant = directory.path().join("descendant");
        fs::create_dir(&descendant).unwrap();
        let child = descendant.join("child");
        fs::write(&child, b"nested file").unwrap();

        let descendant_entry = walk(
            directory.path(),
            2,
            Order::ParentFirst,
            Options::default(),
            |_| true,
        )
        .find_map(|entry| {
            let entry = entry.unwrap();
            (entry.path() == descendant).then_some(entry)
        })
        .expect("the initial walk should yield the descendant directory");
        assert_eq!(descendant_entry.depth, 1);

        let mut events = walk_root_entries(
            [(7, Ok(descendant_entry))],
            2,
            Order::ParentFirst,
            Options::default(),
            |root_idx, entry| {
                assert_eq!(root_idx, 7);
                assert_eq!(entry.depth, 0, "the predicate should see a re-rooted entry");
                true
            },
        );

        let Some((7, RootEvent::Entry(Ok(mut root)))) = events.next() else {
            panic!("the prepared descendant should be emitted as the new root");
        };
        assert_eq!(root.path(), descendant);
        assert_eq!(
            root.depth, 0,
            "the entry originally found at depth 1 must become the new traversal root"
        );

        let Some((7, RootEvent::Entry(Ok(entry)))) = events.next() else {
            panic!("the re-rooted directory should emit its child");
        };
        assert_eq!(entry.path(), child);
        assert_eq!(
            entry.depth, 1,
            "the child depth must be relative to the prepared entry used as the new root"
        );
        assert_eq!(
            events
                .next()
                .map(|(root_idx, event)| (root_idx, matches!(event, RootEvent::Finished))),
            Some((7, true))
        );
        assert_eq!(
            events.next().map(|(root_idx, _)| root_idx),
            None,
            "nothing left after the Finished event"
        );

        root.metadata = Some(Err(io::Error::from(io::ErrorKind::PermissionDenied)));
        let mut events = walk_root_entries(
            [(7, Ok(root))],
            2,
            Order::ParentFirst,
            Options::default(),
            |_, _| panic!("a directory with inaccessible metadata must not be descended"),
        );
        let Some((7, RootEvent::Entry(Ok(root)))) = events.next() else {
            panic!("the prepared directory must retain its metadata error");
        };
        let error = root
            .metadata
            .unwrap()
            .err()
            .expect("the inaccessible root must retain its metadata error");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(
            events
                .next()
                .map(|(root_idx, event)| (root_idx, matches!(event, RootEvent::Finished))),
            Some((7, true))
        );
        assert_eq!(events.next().map(|(root_idx, _)| root_idx), None);
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn prepared_roots_reuse_native_directory_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("file");
        fs::write(&path, b"cached metadata").unwrap();
        let expected_len = fs::metadata(&path).unwrap().len();

        let entry = read_dir(directory.path(), Options::default())
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(entry.depth, 0);
        assert_eq!(entry.path(), path);
        fs::remove_file(&path).unwrap();

        let mut events = walk_root_entries(
            [(7, Ok(entry))],
            1,
            Order::Completion,
            Options::default(),
            |_, _| true,
        );
        let Some((7, RootEvent::Entry(Ok(entry)))) = events.next() else {
            panic!(
                "prepared root must be yielded without querying its removed path which would fail"
            );
        };
        assert_eq!(entry.path(), path);
        assert_eq!(entry.metadata.unwrap().unwrap().len(), expected_len);
        assert_eq!(
            events
                .next()
                .map(|(root_idx, event)| (root_idx, matches!(event, RootEvent::Finished))),
            Some((7, true))
        );
        assert_eq!(events.next().map(|(root_idx, _)| root_idx), None);
    }

    #[test]
    fn wide_walk_wakes_multiple_idle_workers() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..32 {
            fs::create_dir_all(dir.path().join(format!("{idx}/child"))).unwrap();
        }

        let worker_threads = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let seen_threads = Arc::clone(&worker_threads);
        walk(
            dir.path(),
            8,
            Order::Completion,
            Options::default(),
            move |entry| {
                if entry.depth == 1 {
                    thread::sleep(std::time::Duration::from_millis(1));
                } else if entry.depth == 2 {
                    seen_threads.lock().unwrap().insert(thread::current().id());
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                true
            },
        )
        .for_each(drop);

        assert!(
            worker_threads.lock().unwrap().len() >= 4,
            "a wide directory should engage more than the producer and one thief"
        );
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn native_metadata_is_collected_by_the_directory_worker() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..32 {
            fs::create_dir(dir.path().join(idx.to_string())).unwrap();
        }

        let worker_threads = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let seen_threads = Arc::clone(&worker_threads);
        walk(
            dir.path(),
            8,
            Order::Completion,
            Options::default(),
            move |entry| {
                if entry.depth == 1 {
                    seen_threads.lock().unwrap().insert(thread::current().id());
                    thread::sleep(std::time::Duration::from_millis(2));
                }
                true
            },
        )
        .for_each(drop);

        assert_eq!(
            worker_threads.lock().unwrap().len(),
            1,
            "native directory-entry metadata should stay on the enumerating worker"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn metadata_backlog_is_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let count = ENTRY_CHUNK_SIZE * (MAX_QUEUED_STAT_JOBS + 1);
        for idx in 0..count {
            fs::write(dir.path().join(idx.to_string()), b"metadata").unwrap();
        }

        let pool = start_pool(
            1,
            HashMap::from([(0, AtomicUsize::new(0))]),
            Order::ParentFirst,
            Arc::new(|_, _| true),
            Options::default(),
            1,
        );
        // Leave this queue unconsumed to model workers stalled on metadata lookups.
        let worker = Worker::new_lifo();
        let path = Arc::from(dir.path());
        #[cfg(target_os = "macos")]
        let entries = read_dir(dir.path(), Options::default().skip_metadata()).unwrap();
        #[cfg(not(target_os = "macos"))]
        let entries = fs::read_dir(dir.path()).unwrap();
        let mut entries = entries.map(Result::unwrap);
        for _ in 0..=MAX_QUEUED_STAT_JOBS {
            schedule_stat_entries(
                0,
                &path,
                0,
                1,
                entries.by_ref().take(ENTRY_CHUNK_SIZE).collect(),
                &worker,
                &pool.shared,
            );
        }

        assert_eq!(worker.len(), MAX_QUEUED_STAT_JOBS);
        let Event::Batch { batch, .. } = pool.events.try_recv().unwrap() else {
            panic!("a full queue must process the next metadata batch inline");
        };
        let entries = batch.unwrap();
        assert_eq!(entries.len(), ENTRY_CHUNK_SIZE);
        for entry in entries {
            let entry = entry.unwrap();
            assert_eq!(entry.metadata.unwrap().unwrap().len(), 8);
            assert_eq!(entry.parent_directory_id, Some(DirectoryId::new(0)));
        }
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn native_walks_stream_entries_and_child_jobs() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..=ENTRY_CHUNK_SIZE {
            fs::create_dir_all(dir.path().join(format!("{idx}/child"))).unwrap();
        }

        for order in [Order::Completion, Order::ParentFirst] {
            for options in [Options::default(), Options::default().skip_metadata()] {
                let (continue_tx, continue_rx) = std::sync::mpsc::channel();
                let continue_rx = std::sync::Mutex::new(continue_rx);
                let seen = AtomicUsize::new(0);
                let mut entries = walk(dir.path(), 2, order, options, move |entry| {
                    if entry.depth == 1
                        && seen.fetch_add(1, AtomicOrdering::Relaxed) == ENTRY_CHUNK_SIZE
                    {
                        continue_rx.lock().unwrap().recv().ok();
                    }
                    true
                });
                let root = entries.next().unwrap();
                let mut batches = Vec::new();
                let mut child_started = false;
                while let Ok(Event::Batch { batch, .. }) = entries
                    .pool
                    .as_ref()
                    .unwrap()
                    .events
                    .recv_timeout(std::time::Duration::from_secs(2))
                {
                    child_started = batch.as_ref().is_ok_and(|entries| {
                        entries
                            .iter()
                            .any(|entry| entry.as_ref().is_ok_and(|entry| entry.depth == 2))
                    });
                    batches.push(batch);
                    if child_started {
                        break;
                    }
                }
                // Release the producer and drain its events before asserting, including on failure.
                continue_tx.send(()).unwrap();
                let remaining = entries.collect::<Vec<_>>();
                assert!(
                    child_started,
                    "entries and child jobs must be published before all entries are processed \
                     (parent first: {}, options: {options:?})",
                    matches!(order, Order::ParentFirst)
                );

                let entries = std::iter::once(root)
                    .chain(batches.into_iter().flat_map(Result::unwrap))
                    .chain(remaining)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                assert_eq!(entries.len(), 1 + 2 * (ENTRY_CHUNK_SIZE + 1));
                assert_eq!(entries[0].depth, 0, "the root must be yielded first");
                if matches!(order, Order::ParentFirst) {
                    let positions = entries
                        .iter()
                        .enumerate()
                        .map(|(idx, entry)| (entry.directory_id.unwrap(), idx))
                        .collect::<HashMap<_, _>>();
                    for (idx, entry) in entries.iter().enumerate().skip(1) {
                        assert!(
                            positions[&entry.parent_directory_id.unwrap()] < idx,
                            "a parent entry must precede every descendant"
                        );
                    }
                }
            }
        }
    }
}
