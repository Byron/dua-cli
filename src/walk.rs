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
//! The root directory starts in a shared injector queue. Each worker takes jobs from its local
//! FIFO queue, then the injector, then other workers. Reading a directory schedules each accepted
//! child directory on the current worker and requests a worker to wake so it can steal available
//! work. A worker parks when no queue has work and is unparked when new work arrives or the walk
//! stops. The last completed job emits the finished event; dropping the iterator stops all workers,
//! unparks them, and joins their threads.

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use std::{
    ffi::OsString,
    fs::{self, FileType, Metadata},
    io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread,
};

type Descend = dyn Fn(&Entry) -> bool + Send + Sync;
type Batch = io::Result<Vec<io::Result<Entry>>>;

/// Controls when entries are yielded relative to their descendants.
#[derive(Clone, Copy)]
pub enum Order {
    /// Yield entries as their parent-directory reads complete.
    Completion,
    /// Yield every parent before its descendants.
    ParentFirst,
}

/// A filesystem entry produced by [`walk`].
pub struct Entry {
    /// Number of ancestors below the walk root.
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

struct Job {
    path: Arc<Path>,
    depth: usize,
}

enum Event {
    Batch(Batch),
    Finished,
}

struct PoolShared {
    /// Global queue that makes the initial root job available to whichever worker starts first.
    injector: Injector<Job>,
    stealers: Vec<Stealer<Job>>,
    stop: AtomicBool,
    descend: Arc<Descend>,
    events: SyncSender<Event>,
    /// Number of directory jobs that are queued or running.
    pending: AtomicUsize,
    order: Order,
    threads: Mutex<Vec<thread::Thread>>,
    /// A round-robbin counter for the next thread to wake.
    next_wake: AtomicUsize,
}

struct Pool {
    shared: Arc<PoolShared>,
    handles: Vec<thread::JoinHandle<()>>,
}

/// A directory iterator whose directory reads happen in parallel.
pub struct Walk {
    /// Entries buffered for delivery.
    ///
    /// This vector is used as a stack: it starts with the root, and received batches are inserted
    /// in reverse so popping preserves their original order.
    ///
    /// If consumption isn't as fast as its production, threads will block.
    next: Vec<io::Result<Entry>>,
    /// Worker-event receiver while a directory traversal is active.
    ///
    /// It is `None` when the root is not traversed and after the finished event is received.
    events: Option<Receiver<Event>>,
    /// Owns the worker threads for as long as traversal is active.
    ///
    /// Clearing or dropping it requests shutdown, unparks every worker, and joins their threads.
    pool: Option<Pool>,
}

/// Walk `root` without following symlinks.
pub fn walk(
    root: &Path,
    threads: usize,
    order: Order,
    descend: impl Fn(&Entry) -> bool + Send + Sync + 'static,
) -> Walk {
    let threads = threads.max(1);
    let root = Entry::from_path(root);
    let mut pool = None;
    let mut events = None;

    if let Ok(entry) = &root
        && entry.file_type.is_dir()
        && descend(entry)
    {
        let (new_pool, event_rx) = start_pool(
            Job {
                path: Arc::from(entry.path()),
                depth: 1,
            },
            threads,
            order,
            Arc::new(descend),
        );
        pool = Some(new_pool);
        events = Some(event_rx);
    }

    Walk {
        next: vec![root],
        events,
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

            match self.events.as_ref()?.recv() {
                Ok(Event::Batch(Ok(entries))) => {
                    self.next.extend(entries.into_iter().rev());
                }
                Ok(Event::Batch(Err(err))) => return Some(Err(err)),
                Ok(Event::Finished) => {
                    self.events = None;
                    self.pool = None;
                    return None;
                }
                Err(_) => return Some(Err(io::Error::other("directory worker stopped"))),
            }
        }
    }
}

impl PoolShared {
    /// Unpark one worker, selected round-robin.
    ///
    /// Selection does not exclude the caller. Unparking the current thread stores a wake token,
    /// causing its next [`thread::park`] to return immediately rather than waking a peer. This
    /// preserves correctness but may delay additional parallelism until a later call advances to
    /// another worker. Unparking an already-awake worker has the same token semantics.
    fn wake_one_worker(&self) {
        let threads = self.threads.lock().expect("worker list lock");
        if let Some(thread) =
            threads.get(self.next_wake.fetch_add(1, AtomicOrdering::Relaxed) % threads.len().max(1))
        {
            thread.unpark();
        }
    }

    fn wake_workers(&self) {
        for thread in self.threads.lock().expect("worker list lock").iter() {
            thread.unpark();
        }
    }
}

impl Entry {
    /// Return the full path to this entry.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.parent_path.join(&self.file_name)
    }

    fn from_path(path: &Path) -> io::Result<Self> {
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
    first_job: Job,
    threads: usize,
    order: Order,
    descend: Arc<Descend>,
) -> (Pool, Receiver<Event>) {
    let workers: Vec<_> = (0..threads).map(|_| Worker::new_fifo()).collect();
    let (event_tx, event_rx) = sync_channel(threads * 2);
    let shared = Arc::new(PoolShared {
        injector: Injector::new(),
        stealers: workers.iter().map(Worker::stealer).collect(),
        stop: AtomicBool::new(false),
        descend,
        events: event_tx,
        pending: AtomicUsize::new(1),
        order,
        threads: Mutex::new(Vec::with_capacity(threads)),
        next_wake: AtomicUsize::new(0),
    });
    shared.injector.push(first_job);

    let handles: Vec<_> = workers
        .into_iter()
        .enumerate()
        .map(|(idx, worker)| {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name(format!("dua-fs-walk-{idx}"))
                .spawn(move || worker_loop(worker, shared))
                .expect("filesystem worker thread can be spawned")
        })
        .collect();
    *shared.threads.lock().expect("worker list lock") = handles
        .iter()
        .map(|handle| handle.thread().clone())
        .collect();
    shared.wake_workers();

    (Pool { shared, handles }, event_rx)
}

fn worker_loop(worker: Worker<Job>, shared: Arc<PoolShared>) {
    while !shared.stop.load(AtomicOrdering::Relaxed) {
        if let Some(job) = find_job(&worker, &shared) {
            run_job(job, &worker, &shared);
        } else {
            thread::park();
        }
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
/// The worker checks its own FIFO queue first, preserving local scheduling order and avoiding
/// shared-queue contention. It next takes a batch from the injector, keeping one job and moving
/// the rest into its local queue. Only then does it inspect other workers, because stealing from a
/// peer is the most contentious path. Consequently, a worker with local jobs keeps processing
/// them before helping elsewhere, and injector jobs take priority over peer jobs.
fn find_job(worker: &Worker<Job>, shared: &PoolShared) -> Option<Job> {
    loop {
        if let Some(job) = worker.pop() {
            return Some(job);
        }

        match shared.injector.steal_batch_and_pop(worker) {
            Steal::Success(job) => return Some(job),
            Steal::Retry => continue,
            Steal::Empty => {}
        }

        let mut retry = false;
        for stealer in &shared.stealers {
            match stealer.steal() {
                Steal::Success(job) => return Some(job),
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
    let (batch, jobs) = read_dir(&job.path, job.depth, shared);
    shared
        .pending
        .fetch_add(jobs.len(), AtomicOrdering::Relaxed);

    match shared.order {
        Order::ParentFirst => {
            if shared.events.send(Event::Batch(batch)).is_err() {
                shared.stop.store(true, AtomicOrdering::Relaxed);
                return;
            }
            schedule_jobs(jobs, worker, shared);
        }
        Order::Completion => {
            schedule_jobs(jobs, worker, shared);
            if shared.events.send(Event::Batch(batch)).is_err() {
                shared.stop.store(true, AtomicOrdering::Relaxed);
                return;
            }
        }
    }

    let only_this_thread_left = shared.pending.fetch_sub(1, AtomicOrdering::Relaxed) == 1;
    if only_this_thread_left {
        shared.events.send(Event::Finished).ok();
    }
}

fn schedule_jobs(jobs: Vec<Job>, worker: &Worker<Job>, shared: &PoolShared) {
    let has_jobs = !jobs.is_empty();
    for job in jobs {
        worker.push(job);
    }
    if has_jobs {
        shared.wake_one_worker();
    }
}

fn read_dir(path: &Arc<Path>, depth: usize, shared: &PoolShared) -> (Batch, Vec<Job>) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => return (Err(err), Vec::new()),
    };
    let mut jobs = Vec::new();
    let entries = entries
        .map(|entry| {
            entry
                .and_then(|entry| Entry::from_dir_entry(depth, Arc::clone(path), entry))
                .inspect(|entry| {
                    if entry.file_type.is_dir() && (shared.descend)(entry) {
                        jobs.push(Job {
                            path: Arc::from(entry.path()),
                            depth: depth + 1,
                        });
                    }
                })
        })
        .collect();
    (Ok(entries), jobs)
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
}
