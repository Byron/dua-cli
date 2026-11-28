//! Parallel filesystem traversal backed by a work-stealing worker pool.
//!
//! # [`Walk`] or [`MultiWalk`]
//!
//! Use [`walk`] to iterate over a single root. Its [`Walk`] yields one entry or filesystem error
//! at a time, starting with the root. [`Walk::next_cancellable`] checks a cancellation flag between
//! entries and while waiting for worker output. On cancellation it discards buffered entries and
//! joins the workers after their in-flight filesystem calls finish. A deletion tool can combine
//! this with [`Options::skip_metadata`] to collect only paths and file types before removing anything.
//! [`Walk::restart`] can reuse the workers after an exhausted directory walk.
//!
//! Use [`MultiWalk::prepare`] to receive whole batches or track multiple roots. Start with
//! [paths](MultiWalkBuilder::start_paths), [prepared entries](MultiWalkBuilder::start_entries), or
//! [streamed roots](MultiWalkBuilder::stream_roots). [`MultiWalk::recv`] reports a root index
//! and a [`RootEvent`] for each batch, root completion, or terminal worker failure. With an optional
//! [batch visitor](MultiWalkBuilder::visit_batches), workers pass their batches directly to the
//! visitor and `recv` reports completions and worker failures. Fixed root entries are visited on
//! the calling thread during startup. Use a timed `recv` to check for cancellation between
//! waits, and drop the walk to stop its workers.
//!
//! Both APIs use the same worker scheduling. [`Order::ParentFirst`] publishes each entry's batch
//! before scheduling its children, while [`Order::Completion`] allows descendant batches to arrive
//! first when their reads finish sooner. Sibling order is unspecified in both modes.
//!
//! The `descend` predicate prunes a directory's descendants while keeping the directory entry.
//! Both APIs report symbolic links without following them.
//!
//! # Scheduling
//!
//! The root directory starts in a shared injector queue. On platforms where directory-entry
//! metadata may require another syscall, directory reads enqueue small metadata batches, and
//! metadata batches enqueue accepted child directories. Windows and macOS walks consume native
//! metadata returned by directory enumeration instead. Multi-threaded macOS walks probe the
//! initial bulk refills and distribute metadata lookups when those reads spend time waiting.
//! [`MultiWalkBuilder::stream_roots`] accepts additional roots while walking on the same worker pool.
//! Workers check submitted roots before their local queues so existing trees cannot starve new roots.
//! Every worker can run available jobs from its local queue or steal from a peer. Each
//! successful thief wakes another idle worker, ramping up only while work remains stealable. A
//! worker parks when no queue has work and is unparked when new work arrives or the walk stops. The
//! pool emits its final event once all roots finish and root submissions are closed; dropping
//! the walk stops and joins all workers.
//!
//! Linux completion-order metadata walks combine file-only results across directory jobs into
//! batches of at most 512 entries per worker. Partial batches are published before descent
//! callbacks or parking. Directory batches, root entries, errors, and parent-first results are
//! sent directly. Buffered entries keep their root active until they have been published, so
//! completion cannot overtake them.
#![deny(unsafe_code)]
#![deny(missing_docs)]

mod error;
pub use error::Error;

use crossbeam::{
    deque::{Injector, Steal, Stealer, Worker},
    sync::{Parker, Unparker},
};
use std::{
    collections::HashSet,
    fs, io,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
        mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel},
    },
    thread,
    time::{Duration, Instant},
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

#[cfg(all(windows, any(test, feature = "unstable")))]
pub use windows::FileAttributes;

#[cfg(windows)]
pub use windows::{Entry, FileType, Metadata};

#[cfg(windows)]
use windows::{ReadDir as NativeReadDir, read_dir_types};

#[cfg(any(windows, target_os = "macos"))]
enum ReadDir {
    Metadata {
        reader: NativeReadDir,
        path: Arc<Path>,
    },
    FileTypes {
        entries: fs::ReadDir,
        parent_path: Arc<Path>,
        depth: usize,
    },
}

#[cfg(any(windows, target_os = "macos"))]
impl ReadDir {
    fn open(path: Arc<Path>, depth: usize, options: Options) -> Result<Self, Error> {
        if options.skip_metadata {
            Ok(Self::FileTypes {
                entries: read_dir_types(&path)
                    .map_err(|err| Error::at_path(err, Arc::clone(&path)))?,
                parent_path: path,
                depth,
            })
        } else {
            NativeReadDir::open(Arc::clone(&path), depth, options)
                .map(|reader| Self::Metadata {
                    reader,
                    path: Arc::clone(&path),
                })
                .map_err(|err| Error::at_path(err, path))
        }
    }
}

#[cfg(any(windows, target_os = "macos"))]
impl Iterator for ReadDir {
    type Item = Result<Entry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Metadata { reader, path } => reader
                .next()
                .map(|entry| entry.map_err(|err| Error::at_path(err, Arc::clone(path)))),
            Self::FileTypes {
                entries,
                parent_path,
                depth,
            } => entries.next().map(|entry| {
                let entry = entry.map_err(|err| Error::at_path(err, Arc::clone(parent_path)))?;
                Ok(Entry {
                    depth: *depth,
                    file_name: entry.file_name(),
                    file_type: FileType::from_std(
                        entry
                            .file_type()
                            .map_err(|err| Error::at_path(err, entry.path()))?,
                    ),
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
type VisitBatch = dyn Fn(usize, usize, Vec<Result<Entry, Error>>) + Send + Sync;
/// Entries obtained from one directory read.
/// An outer error means the directory could not be opened; inner errors come from reading or
/// converting individual directory entries.
type Batch = Result<Vec<Result<Entry, Error>>, Error>;

/// Number of directory entries grouped into each metadata job or result batch.
/// Native enumeration uses small chunks to stream entries promptly. Linux metadata jobs use
/// larger chunks once workers are busy, retaining smaller jobs while idle workers can share them.
#[cfg(target_os = "linux")]
const ENTRY_CHUNK_SIZE: usize = 64;
#[cfg(not(target_os = "linux"))]
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

/// Filesystem metadata and result buffering requested during traversal.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
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
    /// Result queue capacity per worker, defaulting to `2`.
    ///
    /// Workers share a queue with this capacity multiplied by the worker count and block when
    /// it is full. Increase this to keep workers busy while the consumer pauses to process entries,
    /// at the cost of buffering more batches in memory. Linux completion-order metadata walks
    /// also combine up to 512 entries in each worker before sending a batch. `0` disables both
    /// queue buffering and this worker-local coalescing.
    pub queued_batches_per_worker: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            skip_metadata: false,
            #[cfg(target_os = "macos")]
            apfs_clone_metadata: false,
            queued_batches_per_worker: 2,
        }
    }
}

impl Options {
    /// Collect only entry types, leaving [`Entry::metadata`] as `None`.
    #[must_use]
    pub fn skip_metadata(mut self) -> Self {
        self.skip_metadata = true;
        self
    }

    /// Set the result queue capacity per worker; see [`Self::queued_batches_per_worker`].
    #[must_use]
    pub fn queued_batches_per_worker(mut self, batches: usize) -> Self {
        self.queued_batches_per_worker = batches;
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

struct Root {
    /// Root identifier passed to the descent predicate and attached to emitted events.
    index: usize,
    /// Queued and running jobs plus buffered-output guards; the last completion emits
    /// `RootFinished` after all entries have been sent.
    pending: AtomicUsize,
    /// Shared predicate receiving this root's index and a directory entry.
    /// Returning `false` prunes its children while still yielding the directory itself.
    descend: Arc<Descend>,
}

/// Coalesce small Linux metadata batches without enlarging the shared result queue.
/// Each worker buffers only one root, whose completion count includes the buffered batch.
#[derive(Default)]
struct Output {
    /// Identifies the worker to the direct visitor so it can use worker-local state.
    worker_index: usize,
    /// The root whose entries are buffered, or `None` when the buffer is empty.
    /// The buffer holds one pending count so `RootFinished` cannot precede publication.
    root: Option<Arc<Root>>,
    /// Successful non-directory entries awaiting a channel send, all from `root`.
    entries: Vec<Result<Entry, Error>>,
}

impl Output {
    const CAPACITY: usize = 512;

    fn coalesces(shared: &PoolShared) -> bool {
        cfg!(target_os = "linux")
            && matches!(shared.order, Order::Completion)
            && !shared.options.skip_metadata
            && shared.options.queued_batches_per_worker != 0
            && shared.visitor.is_none()
    }

    fn send(&mut self, root: &Arc<Root>, batch: Batch, shared: &PoolShared) -> bool {
        if shared.stop.load(AtomicOrdering::Relaxed) {
            return false;
        }
        if shared.visitor.is_some() {
            return self.send_direct(root.index, batch, shared);
        }
        match batch {
            Ok(mut entries)
                if Self::coalesces(shared)
                    && entries.iter().all(|entry| {
                        entry.as_ref().is_ok_and(|entry| {
                            !entry.file_type.is_dir()
                                && entry.metadata.as_ref().is_some_and(Result::is_ok)
                        })
                    }) =>
            {
                if entries.is_empty() {
                    return true;
                }
                if (self
                    .root
                    .as_ref()
                    .is_some_and(|buffered| !Arc::ptr_eq(buffered, root))
                    || self.entries.len() + entries.len() > Self::CAPACITY)
                    && !self.flush(shared)
                {
                    return false;
                }
                if self.root.is_none() {
                    root.pending.fetch_add(1, AtomicOrdering::Relaxed);
                    self.root = Some(Arc::clone(root));
                    self.entries = entries;
                } else {
                    self.entries.append(&mut entries);
                }
                self.entries.len() < Self::CAPACITY || self.flush(shared)
            }
            batch => {
                if !self.flush(shared) {
                    return false;
                }
                self.send_direct(root.index, batch, shared)
            }
        }
    }

    /// Deliver a batch without buffering it or flushing any pending entries.
    /// If configured, the visitor runs synchronously on this worker and receives a batch-level
    /// error as a single failed entry. Otherwise, send the batch over the event channel.
    /// Returns `false` if traversal has stopped when the visitor returns or if the channel
    /// receiver has disconnected. A disconnection also stops traversal.
    fn send_direct(&self, root_idx: usize, batch: Batch, shared: &PoolShared) -> bool {
        if let Some(visitor) = &shared.visitor {
            visitor(
                self.worker_index,
                root_idx,
                batch.unwrap_or_else(|err| vec![Err(err)]),
            );
            return !shared.stop.load(AtomicOrdering::Relaxed);
        }
        if shared
            .events
            .send(Event::Batch { root_idx, batch })
            .is_err()
        {
            shared.stop.store(true, AtomicOrdering::Relaxed);
            return false;
        }
        true
    }

    fn flush(&mut self, shared: &PoolShared) -> bool {
        let Some(root) = self.root.take() else {
            return !shared.stop.load(AtomicOrdering::Relaxed);
        };
        let entries = std::mem::take(&mut self.entries);
        // Cancellation discards the buffer. Do not send from Drop or wait for a consumer.
        if shared.stop.load(AtomicOrdering::Relaxed) {
            return false;
        }
        if shared
            .events
            .send(Event::Batch {
                root_idx: root.index,
                batch: Ok(entries),
            })
            .is_err()
        {
            shared.stop.store(true, AtomicOrdering::Relaxed);
            return false;
        }
        // Releasing this guard may finish the root, but only after its batch was enqueued.
        finish_pending(&root, shared);
        true
    }
}

enum Job {
    /// Prepare a newly submitted root on a worker, including its metadata.
    StartRoot { root: Arc<Root>, path: PathBuf },
    /// Release the open-input completion guard on a worker, where sending events can block.
    CloseInput,
    /// Read a directory and schedule processing of its entries.
    ReadDir {
        root: Arc<Root>,
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
        root: Arc<Root>,
        path: Arc<Path>,
        /// Dense identifier of the directory containing these entries.
        directory_id: usize,
        /// Depth assigned to every entry in this chunk; always at least `1`.
        entry_depth: usize,
        entries: Vec<StatEntry>,
    },
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
    /// A panicking worker makes completion accounting unusable; terminate the entire walk.
    WorkerFailed {
        root_idx: usize,
        error: Error,
    },
    /// Emitted once after all roots have emitted `RootFinished`; this is the final event.
    Finished,
}

/// Output from [`MultiWalk::recv`] for a single traversal root.
pub enum RootEvent {
    /// An owned batch of entries and filesystem errors in traversal order.
    Entries(Vec<Result<Entry, Error>>),
    /// All entries for the root have been delivered or visited.
    Finished,
    /// A panic on a worker thread terminated the entire walk in an unwind build.
    ///
    /// This includes panics in a batch visitor or descent predicate running on a worker.
    /// Panics during initial visits propagate on the calling thread.
    /// Ordinary filesystem errors are delivered with entries, not as worker failures.
    WorkerFailed(Error),
}

struct PoolShared {
    /// Shared queue for initial roots, newly submitted roots, and input closure.
    injector: Injector<Job>,
    stealers: Vec<Stealer<Job>>,
    stop: AtomicBool,
    events: SyncSender<Event>,
    visitor: Option<Arc<VisitBatch>>,
    /// Active roots, plus one while a streaming input remains open.
    active_roots: AtomicUsize,
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
    events: Option<Receiver<Event>>,
    handles: Vec<thread::JoinHandle<()>>,
}

/// A parallel walk over one or more roots that delivers batches and per-root completion through
/// [`Self::recv`].
pub struct MultiWalk {
    /// Prepared roots and completions that precede worker output.
    pending: Option<Pending>,
    /// See [`Walk::pool`].
    pool: Option<Pool>,
}

/// Startup configuration for a multi-root walk, created with [`MultiWalk::prepare`].
#[must_use]
pub struct MultiWalkBuilder {
    threads: usize,
    order: Order,
    options: Options,
    visitor: Option<Arc<VisitBatch>>,
}

enum Pending {
    Initial(std::vec::IntoIter<(usize, InitialEvent)>),
    InitialFinished(std::vec::IntoIter<usize>),
}

enum InitialEvent {
    Entry(Result<Entry, Error>),
    Finished,
}

/// Submits roots to a running [`MultiWalk`] without starting additional workers.
/// Drop this handle after discovery finishes to allow the walk to finish.
pub struct RootSender {
    shared: Arc<PoolShared>,
    indices: HashSet<usize>,
}

impl RootSender {
    /// Queue a root and its descent predicate. Metadata is collected by the worker pool.
    ///
    /// * `index` identifies this root in [`MultiWalk`] events and must be unique across all roots
    ///   submitted through this sender, including roots that have already finished.
    /// * `path` is the filesystem entry to walk, yielded at depth `0`. Symbolic links are not
    ///   followed. Filesystem errors are delivered through [`MultiWalk`].
    /// * `descend` decides whether to traverse a directory's children, including the root's.
    ///   Returning `false` still yields the directory itself. The predicate runs on worker
    ///   threads and may be called concurrently for different directories.
    ///
    /// Returns an error for a duplicate index or a dropped walker.
    pub fn add_root(
        &mut self,
        index: usize,
        path: PathBuf,
        descend: impl Fn(&Entry) -> bool + Send + Sync + 'static,
    ) -> io::Result<()> {
        if self.shared.stop.load(AtomicOrdering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "walker stopped"));
        }
        if !self.indices.insert(index) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "duplicate root index",
            ));
        }
        let root = Arc::new(Root {
            index,
            pending: AtomicUsize::new(1),
            descend: Arc::new(move |_, entry| descend(entry)),
        });
        self.shared
            .active_roots
            .fetch_add(1, AtomicOrdering::Relaxed);
        self.shared.injector.push(Job::StartRoot { root, path });
        self.shared.wake_worker();
        Ok(())
    }
}

impl Drop for RootSender {
    fn drop(&mut self) {
        // Sending the final event may block on a full output channel; leave it to a worker.
        self.shared.injector.push(Job::CloseInput);
        self.shared.wake_worker();
    }
}

/// A single-root directory iterator whose directory reads happen in parallel.
/// It yields individual entries and filesystem errors.
pub struct Walk {
    /// Entries buffered for delivery.
    ///
    /// This vector is used as a stack: it starts with the root, and received batches are inserted
    /// in reverse so popping preserves their original order.
    ///
    /// If consumption isn't as fast as its production, threads will block.
    next: Vec<Result<Entry, Error>>,
    /// Owns the worker threads for as long as traversal is active.
    ///
    /// Clearing or dropping it requests shutdown, unparks every worker, and joins their threads.
    pool: Option<Pool>,
    root: PathBuf,
    root_state: Arc<Root>,
    options: Options,
    /// Whether the worker pool has finished the current traversal.
    finished: bool,
}

/// Read a directory using native enumeration, collecting metadata unless
/// [`Options::skip_metadata`] is set.
///
/// Entries have depth zero so they can be passed directly to [`MultiWalkBuilder::start_entries`] without
/// querying their paths again. Directory-open errors are returned immediately; later enumeration
/// errors are yielded by the iterator.
#[cfg(any(windows, target_os = "macos"))]
pub fn read_dir(
    path: &Path,
    options: Options,
) -> Result<impl Iterator<Item = Result<Entry, Error>>, Error> {
    ReadDir::open(Arc::from(path), 0, options)
}

/// Walk a single root without following symlinks, yielding entries directly.
///
/// Use [`MultiWalk`] to walk multiple roots and receive batches and per-root completion events.
pub fn walk(
    root: &Path,
    threads: usize,
    order: Order,
    options: Options,
    descend: impl Fn(&Entry) -> bool + Send + Sync + 'static,
) -> Walk {
    let root_path = root.to_owned();
    let root_state = Arc::new(Root {
        index: 0,
        pending: AtomicUsize::new(0),
        descend: Arc::new(move |_, entry| descend(entry)),
    });
    let mut root = Entry::from_path(root, options);
    if let Ok(entry) = &mut root
        && entry.file_type.is_dir()
    {
        entry.directory_id = Some(DirectoryId::new(0));
    }
    let pool = match &root {
        Ok(entry) if entry.file_type.is_dir() && (root_state.descend)(0, entry) => {
            let path = Arc::from(entry.path());
            let pool = start_pool(threads.max(1), order, options, 1);
            start_jobs(
                &pool,
                vec![Job::ReadDir {
                    root: Arc::clone(&root_state),
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
        root_state,
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
            Ok(entry) if entry.file_type.is_dir() && (self.root_state.descend)(0, entry) => {
                Some(Job::ReadDir {
                    root: Arc::clone(&self.root_state),
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

    /// Yield the next entry, polling `cancelled` while waiting for directory reads.
    ///
    /// Once cancelled, discard buffered entries and stop and join the worker pool before
    /// returning `None`. In-progress filesystem calls are allowed to finish, and the cancelled
    /// walk cannot be restarted.
    pub fn next_cancellable(&mut self, cancelled: &AtomicBool) -> Option<Result<Entry, Error>> {
        self.next_with_cancellation(Some(cancelled))
    }

    fn next_with_cancellation(
        &mut self,
        cancelled: Option<&AtomicBool>,
    ) -> Option<Result<Entry, Error>> {
        const CANCELLATION_POLL_INTERVAL: Duration = Duration::from_millis(100);

        loop {
            if cancelled.is_some_and(|cancelled| cancelled.load(AtomicOrdering::Relaxed)) {
                self.next.clear();
                self.finished = true;
                self.pool = None;
                return None;
            }
            if let Some(entry) = self.next.pop() {
                return Some(entry);
            }
            if self.finished {
                return None;
            }

            let events = self.pool.as_ref()?.events.as_ref()?;
            let event = if cancelled.is_some() {
                match events.recv_timeout(CANCELLATION_POLL_INTERVAL) {
                    Ok(event) => event,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        return Some(Err(io::Error::other("directory worker stopped").into()));
                    }
                }
            } else {
                match events.recv() {
                    Ok(event) => event,
                    Err(_) => {
                        return Some(Err(io::Error::other("directory worker stopped").into()));
                    }
                }
            };
            match event {
                Event::Batch {
                    batch: Ok(entries), ..
                } => {
                    self.next.extend(entries.into_iter().rev());
                }
                Event::Batch {
                    batch: Err(err), ..
                } => return Some(Err(err)),
                Event::RootFinished { .. } => {}
                Event::WorkerFailed { error, .. } => {
                    self.next.clear();
                    self.finished = true;
                    self.pool = None;
                    return Some(Err(error));
                }
                Event::Finished => {
                    self.finished = true;
                    return None;
                }
            }
        }
    }
}

impl Iterator for Walk {
    type Item = Result<Entry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_with_cancellation(None)
    }
}

/// Lifecycle
impl MultiWalk {
    /// Configure a multi-root walk before starting workers or visiting any roots.
    pub fn prepare(threads: usize, order: Order, options: Options) -> MultiWalkBuilder {
        MultiWalkBuilder {
            threads,
            order,
            options,
            visitor: None,
        }
    }
}

/// Builder
impl MultiWalkBuilder {
    /// Consume entry batches using a callback. The resulting walk reports root completions and
    /// worker failures through [`MultiWalk::recv`].
    ///
    /// `visit` receives a worker index in `0..threads.max(1)`, the root index, and the owned batch
    /// of entries and filesystem errors. Fixed roots are visited during startup on the calling
    /// thread, with worker index zero. Streamed roots are visited on workers. Worker visits may
    /// run concurrently, and a root completes after all its visits return. Dropping the walk
    /// waits for active visits to finish.
    pub fn visit_batches(
        mut self,
        visit: impl Fn(usize, usize, Vec<Result<Entry, Error>>) + Send + Sync + 'static,
    ) -> Self {
        self.visitor = Some(Arc::new(visit));
        self
    }
}

/// Lifecycle
impl MultiWalkBuilder {
    /// Start a walk over indexed paths without following symlinks.
    ///
    /// Root metadata is read during startup. The root index identifies its batches and
    /// completion, and is passed to `descend`.
    ///
    /// # Panics
    ///
    /// Panics if root indices are duplicated or if an initial callback panics.
    pub fn start_paths(
        self,
        roots: impl IntoIterator<Item = (usize, PathBuf)>,
        descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
    ) -> MultiWalk {
        let options = self.options;
        self.start(roots, descend, |path| Entry::from_path(&path, options))
    }

    /// Start a walk over indexed, prepared entries and reuse their metadata.
    ///
    /// Supplied entries are re-rooted at depth zero before `descend` runs. Their descendants
    /// start at depth one and use the configured metadata options. Supplied errors are reported
    /// under their root index.
    ///
    /// # Panics
    ///
    /// Panics if root indices are duplicated or if an initial callback panics.
    pub fn start_entries(
        self,
        roots: impl IntoIterator<Item = (usize, Result<Entry, Error>)>,
        descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
    ) -> MultiWalk {
        self.start(roots, descend, std::convert::identity)
    }

    /// Start a pool that accepts new roots through [`RootSender::add_root`] during traversal.
    ///
    /// Workers can begin processing each root as soon as it is submitted, using that root's own
    /// descent predicate. Batches and per-root completion events can be delivered while the sender
    /// is still open.
    ///
    /// If no event is ready, [`MultiWalk::recv`] with `None` waits for more work while the sender
    /// is open, even after all submitted roots have finished. On normal completion, `recv`
    /// returns `Ok(None)` once the sender is dropped and all submitted roots have finished.
    #[must_use]
    pub fn stream_roots(self) -> (RootSender, MultiWalk) {
        let pool = start_pool_with_visitor(
            self.threads.max(1),
            self.order,
            self.options,
            0,
            self.visitor,
        );
        pool.shared.active_roots.store(1, AtomicOrdering::Relaxed);
        let sender = RootSender {
            shared: Arc::clone(&pool.shared),
            indices: HashSet::new(),
        };
        (
            sender,
            MultiWalk {
                pending: None,
                pool: Some(pool),
            },
        )
    }

    fn start<Root>(
        self,
        roots: impl IntoIterator<Item = (usize, Root)>,
        descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
        prepare_root: impl Fn(Root) -> Result<Entry, Error>,
    ) -> MultiWalk {
        let Self {
            threads,
            order,
            options,
            visitor,
        } = self;
        let roots: Vec<_> = roots.into_iter().collect();
        assert_eq!(
            roots
                .iter()
                .map(|(index, _)| *index)
                .collect::<HashSet<_>>()
                .len(),
            roots.len(),
            "root indices must be unique"
        );
        let descend: Arc<Descend> = Arc::new(descend);
        let mut initial = Vec::new();
        let mut initial_finished = Vec::new();
        let (root_jobs, next_directory_id) = begin_walks(
            roots
                .into_iter()
                .map(|(root_idx, root)| (root_idx, prepare_root(root))),
            &descend,
            |root_idx, entry, finished| {
                if let Some(visitor) = &visitor {
                    visitor(0, root_idx, vec![entry]);
                    if finished {
                        initial_finished.push(root_idx);
                    }
                } else {
                    initial.push((root_idx, InitialEvent::Entry(entry)));
                    if finished {
                        initial.push((root_idx, InitialEvent::Finished));
                    }
                }
            },
        );
        let pending = if visitor.is_some() {
            Pending::InitialFinished(initial_finished.into_iter())
        } else {
            Pending::Initial(initial.into_iter())
        };
        let pool = if root_jobs.is_empty() {
            None
        } else {
            let pool =
                start_pool_with_visitor(threads.max(1), order, options, next_directory_id, visitor);
            start_jobs(&pool, root_jobs);
            Some(pool)
        };
        MultiWalk {
            pending: Some(pending),
            pool,
        }
    }
}

impl MultiWalk {
    /// Receive the next whole batch, completion, or worker failure for a root.
    ///
    /// Worker entry vectors are transferred directly and empty batches are skipped.
    /// With a [batch visitor](MultiWalkBuilder::visit_batches), this reports only root completions
    /// and terminal worker failures.
    ///
    /// Pass `None` to wait indefinitely or `Some(duration)` to wait up to that duration.
    /// A zero duration polls the channel. If skipping an empty batch exhausts the timeout,
    /// later events stay queued. `Ok(None)` means the walk has finished.
    pub fn recv(
        &mut self,
        timeout: Option<Duration>,
    ) -> Result<Option<(usize, RootEvent)>, RecvTimeoutError> {
        if let Some(event) = self.next_initial() {
            return Ok(Some(event));
        }
        let wait = timeout.map(|timeout| (Instant::now(), timeout));
        loop {
            let Some(events) = self.pool.as_ref().and_then(|pool| pool.events.as_ref()) else {
                return Ok(None);
            };
            let event = if let Some((started, timeout)) = wait {
                events.recv_timeout(timeout.saturating_sub(started.elapsed()))?
            } else {
                events.recv().map_err(|_| RecvTimeoutError::Disconnected)?
            };
            if let Some(batch) = self.accept_event(event) {
                return Ok(Some(batch));
            }
            if self.pool.is_some()
                && wait.is_some_and(|(started, timeout)| started.elapsed() >= timeout)
            {
                return Err(RecvTimeoutError::Timeout);
            }
        }
    }

    fn next_initial(&mut self) -> Option<(usize, RootEvent)> {
        let event = match self.pending.as_mut()? {
            Pending::Initial(events) => events.next().map(|(root_idx, event)| {
                let batch = match event {
                    InitialEvent::Entry(entry) => RootEvent::Entries(vec![entry]),
                    InitialEvent::Finished => RootEvent::Finished,
                };
                (root_idx, batch)
            }),
            Pending::InitialFinished(indices) => indices
                .next()
                .map(|root_idx| (root_idx, RootEvent::Finished)),
        };
        if event.is_none() {
            self.pending = None;
        }
        event
    }

    fn accept_event(&mut self, event: Event) -> Option<(usize, RootEvent)> {
        match event {
            Event::Batch {
                root_idx,
                batch: Ok(entries),
            } => (!entries.is_empty()).then_some((root_idx, RootEvent::Entries(entries))),
            Event::Batch {
                root_idx,
                batch: Err(err),
            } => Some((root_idx, RootEvent::Entries(vec![Err(err)]))),
            Event::RootFinished { root_idx } => Some((root_idx, RootEvent::Finished)),
            Event::WorkerFailed { root_idx, error } => {
                self.pending = None;
                if let Some(pool) = &mut self.pool {
                    pool.disconnect();
                }
                Some((root_idx, RootEvent::WorkerFailed(error)))
            }
            Event::Finished => {
                self.pool = None;
                None
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
    pub fn from_path(path: &Path, options: Options) -> Result<Self, Error> {
        let metadata = fs::symlink_metadata(path).map_err(|err| Error::at_path(err, path))?;
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
    ) -> Result<Self, Error> {
        Ok(Self {
            depth,
            file_name: entry.file_name(),
            file_type: entry
                .file_type()
                .map_err(|err| Error::at_path(err, entry.path()))?,
            metadata: (!options.skip_metadata).then(|| entry.metadata()),
            parent_path,
            directory_id: None,
            parent_directory_id: None,
        })
    }
}

fn start_pool(threads: usize, order: Order, options: Options, next_directory_id: usize) -> Pool {
    start_pool_with_visitor(threads, order, options, next_directory_id, None)
}

fn start_pool_with_visitor(
    threads: usize,
    order: Order,
    options: Options,
    next_directory_id: usize,
    visitor: Option<Arc<VisitBatch>>,
) -> Pool {
    let workers: Vec<_> = (0..threads).map(|_| Worker::new_lifo()).collect();
    let parkers: Vec<_> = (0..threads).map(|_| Parker::new()).collect();
    let (event_tx, event_rx) = sync_channel(
        threads
            .checked_mul(options.queued_batches_per_worker)
            .expect("result queue capacity overflow"),
    );
    let shared = Arc::new(PoolShared {
        injector: Injector::new(),
        stealers: workers.iter().map(Worker::stealer).collect(),
        stop: AtomicBool::new(false),
        events: event_tx,
        visitor,
        active_roots: AtomicUsize::new(0),
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
                .spawn(move || {
                    let mut root_idx = 0;
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        worker_loop(idx, worker, parker, Arc::clone(&shared), &mut root_idx);
                    }));
                    if let Err(payload) = result
                        && !shared.stop.swap(true, AtomicOrdering::AcqRel)
                    {
                        shared.wake_workers();
                        let message = payload
                            .downcast_ref::<&str>()
                            .copied()
                            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                            .unwrap_or("non-string panic payload");
                        let error =
                            io::Error::other(format!("filesystem worker panicked: {message}"));
                        shared
                            .events
                            .send(Event::WorkerFailed {
                                root_idx,
                                error: error.into(),
                            })
                            .ok();
                    }
                })
                .expect("filesystem worker thread can be spawned")
        })
        .collect();

    Pool {
        shared,
        events: Some(event_rx),
        handles,
    }
}

/// Prepare initial roots and report each entry with whether it needs no further work.
fn begin_walks(
    roots: impl IntoIterator<Item = (usize, Result<Entry, Error>)>,
    descend: &Arc<Descend>,
    mut initial: impl FnMut(usize, Result<Entry, Error>, bool),
) -> (Vec<Job>, usize) {
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
                root: Arc::new(Root {
                    index: root_idx,
                    pending: AtomicUsize::new(0),
                    descend: Arc::clone(descend),
                }),
                path: Arc::from(entry.path()),
                directory_id: directory_id.index(),
                entry_depth: 1,
            });
            true
        } else {
            false
        };
        initial(root_idx, entry, !has_job);
    }
    (jobs, next_directory_id)
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
    pool.shared
        .active_roots
        .store(root_jobs.len(), AtomicOrdering::Relaxed);
    for job in &root_jobs {
        let Job::ReadDir { root, .. } = job else {
            unreachable!("initial directory jobs")
        };
        root.pending.store(1, AtomicOrdering::Relaxed);
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

fn worker_loop(
    idx: usize,
    worker: Worker<Job>,
    parker: Parker,
    shared: Arc<PoolShared>,
    root_idx: &mut usize,
) {
    let mut output = Output {
        worker_index: idx,
        ..Output::default()
    };
    while !shared.stop.load(AtomicOrdering::Relaxed) {
        let found = if let Some(found) = find_job(&worker, &shared) {
            found
        } else {
            // A partial batch must reach the consumer before this worker can become idle.
            if !output.flush(&shared) {
                break;
            }
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
        match &job {
            Job::StartRoot { root, .. } | Job::ReadDir { root, .. } => *root_idx = root.index,
            #[cfg(not(windows))]
            Job::Stat { root, .. } => *root_idx = root.index,
            Job::CloseInput => {}
        }
        if stolen {
            // A successful steal proves peer work is available; wake one more worker so
            // concurrency ramps up only while work remains stealable.
            shared.wake_worker();
        }
        run_job(job, &worker, &shared, &mut output);
    }
}

impl Pool {
    fn disconnect(&mut self) {
        self.shared.stop.store(true, AtomicOrdering::Relaxed);
        self.shared.wake_workers();
        // Workers blocked on the bounded output channel must be released before joining them.
        drop(self.events.take());
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.disconnect();
        for handle in self.handles.drain(..) {
            handle.join().ok();
        }
    }
}

/// Find newly submitted roots, then local work, then work from peers.
///
/// Check submitted roots before local directory jobs so a large existing tree cannot starve
/// newly discovered roots. Local LIFO work still takes priority over stealing from peers.
///
/// Returns the selected job and whether it was stolen from another worker; the caller uses a
/// successful steal to wake another idle worker. Returns `None` when a full scan finds no work.
fn find_job(worker: &Worker<Job>, shared: &PoolShared) -> Option<(Job, bool)> {
    loop {
        match shared.injector.steal() {
            Steal::Success(job) => return Some((job, true)),
            Steal::Retry => continue,
            Steal::Empty => {}
        }
        if let Some(job) = worker.pop() {
            return Some((job, false));
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

fn run_job(job: Job, worker: &Worker<Job>, shared: &PoolShared, output: &mut Output) {
    let flush = match &job {
        Job::StartRoot { .. } | Job::CloseInput => true,
        Job::ReadDir { root, .. } => output
            .root
            .as_ref()
            .is_some_and(|buffered| !Arc::ptr_eq(buffered, root)),
        #[cfg(not(windows))]
        Job::Stat { root, .. } => output
            .root
            .as_ref()
            .is_some_and(|buffered| !Arc::ptr_eq(buffered, root)),
    };
    if flush && !output.flush(shared) {
        return;
    }
    match job {
        Job::StartRoot { root, path } => {
            let mut entry = Entry::from_path(&path, shared.options);
            let mut jobs = Vec::new();
            if let Ok(entry) = &mut entry
                && entry.file_type.is_dir()
            {
                entry.directory_id = Some(allocate_directory_id(shared));
                if entry.metadata.as_ref().is_none_or(Result::is_ok)
                    && (root.descend)(root.index, entry)
                {
                    jobs.push(Job::ReadDir {
                        root: Arc::clone(&root),
                        path: Arc::from(path),
                        directory_id: entry.directory_id.unwrap().index(),
                        entry_depth: 1,
                    });
                }
            }
            // Root entries precede descendants in either traversal order.
            root.pending.fetch_add(jobs.len(), AtomicOrdering::Relaxed);
            if !output.send_direct(root.index, Ok(vec![entry]), shared) {
                return;
            }
            schedule_jobs(jobs, worker, shared);
            finish_pending(&root, shared);
        }
        Job::CloseInput => finish_root(shared),
        Job::ReadDir {
            root,
            path,
            directory_id,
            entry_depth,
        } => {
            #[cfg(any(windows, target_os = "macos"))]
            read_dir_native(
                &root,
                path,
                directory_id,
                entry_depth,
                worker,
                shared,
                output,
            );
            #[cfg(not(any(windows, target_os = "macos")))]
            if matches!(shared.order, Order::Completion) {
                read_dir_parallel(
                    &root,
                    path,
                    directory_id,
                    entry_depth,
                    worker,
                    shared,
                    output,
                );
            } else {
                read_dir_parent_first(
                    &root,
                    path,
                    directory_id,
                    entry_depth,
                    worker,
                    shared,
                    output,
                );
            }
        }
        #[cfg(not(windows))]
        Job::Stat {
            root,
            path,
            directory_id,
            entry_depth,
            entries,
        } => stat_entries(
            &root,
            path,
            directory_id,
            entry_depth,
            entries,
            worker,
            shared,
            output,
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
    root: &Arc<Root>,
    path: Arc<Path>,
    directory_id: usize,
    entry_depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) {
    if shared.options.skip_metadata {
        read_dir_inline(
            root,
            path,
            directory_id,
            entry_depth,
            worker,
            shared,
            output,
        );
        return;
    }
    let dir_entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            output.send(root, Err(Error::at_path(err, Arc::clone(&path))), shared);
            finish_pending(root, shared);
            return;
        }
    };
    let mut chunk_size = metadata_job_size(shared);
    let mut chunk = Vec::with_capacity(ENTRY_CHUNK_SIZE);
    let mut errors = Vec::new();
    for entry in dir_entries {
        if shared.stop.load(AtomicOrdering::Relaxed) {
            finish_pending(root, shared);
            return;
        }
        match entry {
            Ok(entry) => {
                chunk.push(entry);
                if chunk.len() == chunk_size {
                    schedule_stat_entries(
                        root,
                        &path,
                        directory_id,
                        entry_depth,
                        std::mem::replace(&mut chunk, Vec::with_capacity(ENTRY_CHUNK_SIZE)),
                        worker,
                        shared,
                        output,
                    );
                    chunk_size = metadata_job_size(shared);
                }
            }
            Err(err) => errors.push(Err(Error::at_path(err, Arc::clone(&path)))),
        }
    }
    if !chunk.is_empty() {
        schedule_stat_entries(
            root,
            &path,
            directory_id,
            entry_depth,
            chunk,
            worker,
            shared,
            output,
        );
    }
    if !errors.is_empty() {
        output.send(root, Ok(errors), shared);
    }
    finish_pending(root, shared);
}

/// Share metadata latency with idle Linux workers without shrinking batches when they are busy.
#[cfg(not(any(windows, target_os = "macos")))]
fn metadata_job_size(shared: &PoolShared) -> usize {
    if cfg!(target_os = "linux")
        && shared
            .idle
            .iter()
            .any(|idle| idle.load(AtomicOrdering::Relaxed))
    {
        4
    } else {
        ENTRY_CHUNK_SIZE
    }
}

/// Read a directory in either traversal order.
///
/// Native metadata is published directly in chunks. An initial macOS bulk probe can choose
/// ordinary directory enumeration with bounded parallel metadata jobs. Each parent-first batch
/// is sent before its child jobs become stealable.
#[cfg(any(windows, target_os = "macos"))]
fn read_dir_native(
    root: &Arc<Root>,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) {
    let dir_entries = match ReadDir::open(Arc::clone(&path), depth, shared.options) {
        Ok(entries) => entries,
        Err(err) => {
            publish_directory(root, Err(err), Vec::new(), worker, shared, output);
            finish_pending(root, shared);
            return;
        }
    };
    #[cfg(target_os = "macos")]
    let dir_entries = {
        let mut dir_entries = dir_entries;
        let mut prefix = Vec::new();
        if shared.stealers.len() > 1
            && let ReadDir::Metadata { reader, .. } = &mut dir_entries
        {
            prefix = reader
                .probe_metadata()
                .into_iter()
                .map(|entry| entry.map_err(|err| Error::at_path(err, Arc::clone(&path))))
                .collect();
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
            finish_pending(root, shared);
            return;
        }
        #[cfg(target_os = "macos")]
        if !shared.options.skip_metadata
            && entry.as_ref().is_ok_and(|entry| entry.metadata.is_none())
        {
            deferred.push(entry.expect("metadata-free entry was checked"));
            if deferred.len() == ENTRY_CHUNK_SIZE {
                schedule_stat_entries(
                    root,
                    &path,
                    directory_id,
                    depth,
                    std::mem::replace(&mut deferred, Vec::with_capacity(ENTRY_CHUNK_SIZE)),
                    worker,
                    shared,
                    output,
                );
            }
            continue;
        }
        if let Ok(entry) = &mut entry {
            assign_directory_ids(entry, directory_id, shared);
        }
        if let Ok(entry) = &entry
            && entry.file_type.is_dir()
            && output.flush(shared)
            && (root.descend)(root.index, entry)
        {
            jobs.push(Job::ReadDir {
                root: Arc::clone(root),
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
                root,
                Ok(std::mem::replace(
                    &mut entries,
                    Vec::with_capacity(ENTRY_CHUNK_SIZE),
                )),
                std::mem::take(&mut jobs),
                worker,
                shared,
                output,
            )
        {
            finish_pending(root, shared);
            return;
        }
    }
    if !entries.is_empty() {
        publish_directory(root, Ok(entries), jobs, worker, shared, output);
    }
    #[cfg(target_os = "macos")]
    if !deferred.is_empty() {
        schedule_stat_entries(
            root,
            &path,
            directory_id,
            depth,
            deferred,
            worker,
            shared,
            output,
        );
    }
    finish_pending(root, shared);
}

#[cfg(not(windows))]
#[allow(clippy::too_many_arguments)]
fn schedule_stat_entries(
    root: &Arc<Root>,
    path: &Arc<Path>,
    directory_id: usize,
    entry_depth: usize,
    entries: Vec<StatEntry>,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) {
    let job = Job::Stat {
        root: Arc::clone(root),
        path: Arc::clone(path),
        directory_id,
        entry_depth,
        entries,
    };
    root.pending.fetch_add(1, AtomicOrdering::Relaxed);
    if worker.len() >= MAX_QUEUED_STAT_JOBS {
        run_job(job, worker, shared, output);
    } else {
        worker.push(job);
        shared.wake_worker();
    }
}

#[cfg(not(windows))]
#[allow(clippy::too_many_arguments)]
fn stat_entries(
    root: &Arc<Root>,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    entries: Vec<StatEntry>,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) {
    #[cfg(target_os = "macos")]
    let _ = (path, depth);
    let mut jobs = Vec::new();
    let mut batch = Vec::with_capacity(entries.len());
    let entries = entries
        .into_iter()
        .take_while(|_| !shared.stop.load(AtomicOrdering::Relaxed))
        .map(|entry| {
            #[cfg(target_os = "macos")]
            let entry = Ok(entry.read_metadata(shared.options));
            #[cfg(not(target_os = "macos"))]
            let entry = Entry::from_dir_entry(depth, Arc::clone(&path), entry, shared.options);
            entry.map(|mut entry| {
                assign_directory_ids(&mut entry, directory_id, shared);
                if entry.file_type.is_dir()
                    && output.flush(shared)
                    && (root.descend)(root.index, &entry)
                {
                    jobs.push(Job::ReadDir {
                        root: Arc::clone(root),
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
        });
    batch.extend(entries);
    if shared.stop.load(AtomicOrdering::Relaxed) {
        return;
    }
    publish_directory(root, Ok(batch), jobs, worker, shared, output);
    finish_pending(root, shared);
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
    root: &Arc<Root>,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) {
    read_dir_inline(root, path, directory_id, depth, worker, shared, output);
}

/// Convert a directory's entries on the worker that enumerates it, then schedule its children.
///
/// Parent-first traversal converts each entry inline to preserve ordering.
#[cfg(not(any(windows, target_os = "macos")))]
fn read_dir_inline(
    root: &Arc<Root>,
    path: Arc<Path>,
    directory_id: usize,
    depth: usize,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) {
    let dir_entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(err) => {
            publish_directory(
                root,
                Err(Error::at_path(err, Arc::clone(&path))),
                Vec::new(),
                worker,
                shared,
                output,
            );
            finish_pending(root, shared);
            return;
        }
    };
    let mut jobs = Vec::new();
    let entries = dir_entries
        .map(|entry| {
            entry
                .map_err(|err| Error::at_path(err, Arc::clone(&path)))
                .and_then(|entry| {
                    Entry::from_dir_entry(depth, Arc::clone(&path), entry, shared.options)
                })
                .map(|mut entry| {
                    assign_directory_ids(&mut entry, directory_id, shared);
                    if entry.file_type.is_dir()
                        && output.flush(shared)
                        && (root.descend)(root.index, &entry)
                    {
                        jobs.push(Job::ReadDir {
                            root: Arc::clone(root),
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
    publish_directory(root, Ok(entries), jobs, worker, shared, output);
    finish_pending(root, shared);
}

fn assign_directory_ids(entry: &mut Entry, parent_directory_id: usize, shared: &PoolShared) {
    entry.parent_directory_id = Some(DirectoryId::new(parent_directory_id));
    entry.directory_id = entry
        .file_type
        .is_dir()
        .then(|| allocate_directory_id(shared));
}

#[allow(deprecated)] // `try_update` requires Rust 1.95; keep the Rust 1.88-compatible alias.
fn allocate_directory_id(shared: &PoolShared) -> DirectoryId {
    DirectoryId::new(
        shared
            .next_directory_id
            .fetch_update(AtomicOrdering::Relaxed, AtomicOrdering::Relaxed, |id| {
                id.checked_add(1)
            })
            .expect("directory identifier overflow"),
    )
}

/// Publish a directory batch and schedule its accepted child-directory jobs.
/// `ParentFirst` and coalesced walks send directory batches before exposing their child jobs.
/// Child jobs are counted before either action; the caller completes the current job after EOF.
///
/// Returns `true` if the batch was sent or buffered and child jobs were scheduled, or `false`
/// if traversal stopped or the event receiver disconnected.
fn publish_directory(
    root: &Arc<Root>,
    batch: Batch,
    jobs: Vec<Job>,
    worker: &Worker<Job>,
    shared: &PoolShared,
    output: &mut Output,
) -> bool {
    root.pending.fetch_add(jobs.len(), AtomicOrdering::Relaxed);

    if matches!(shared.order, Order::ParentFirst) || !jobs.is_empty() && Output::coalesces(shared) {
        if !output.send(root, batch, shared) {
            return false;
        }
        schedule_jobs(jobs, worker, shared);
        true
    } else {
        schedule_jobs(jobs, worker, shared);
        output.send(root, batch, shared)
    }
}

/// Complete a job, publishing root completion before releasing its pool-wide count.
fn finish_pending(root: &Root, shared: &PoolShared) {
    // An interrupted job may have discarded entries. Leave it pending so other
    // workers cannot report this root as complete ahead of the terminal error.
    if shared.stop.load(AtomicOrdering::Relaxed) {
        return;
    }
    if root.pending.fetch_sub(1, AtomicOrdering::AcqRel) == 1 {
        shared
            .events
            .send(Event::RootFinished {
                root_idx: root.index,
            })
            .ok();
        finish_root(shared);
    }
}

fn finish_root(shared: &PoolShared) {
    if shared.active_roots.fetch_sub(1, AtomicOrdering::AcqRel) == 1 {
        shared.events.send(Event::Finished).ok();
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
    use std::collections::HashMap;

    #[cfg(target_os = "linux")]
    #[test]
    fn discarded_directory_batch_does_not_finish_its_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("child")).unwrap();
        let pool = start_pool(1, Order::ParentFirst, Options::default(), 1);
        pool.shared.active_roots.store(2, AtomicOrdering::Relaxed);
        let shared = Arc::clone(&pool.shared);
        let root = Arc::new(Root {
            index: 7,
            pending: AtomicUsize::new(1),
            descend: Arc::new(move |_, _| {
                // Model the interval between a peer stopping the pool and sending WorkerFailed.
                shared.stop.store(true, AtomicOrdering::Relaxed);
                false
            }),
        });
        run_job(
            Job::ReadDir {
                root,
                path: Arc::from(dir.path()),
                directory_id: 0,
                entry_depth: 1,
            },
            &Worker::new_lifo(),
            &pool.shared,
            &mut Output::default(),
        );
        assert!(matches!(
            pool.events.as_ref().unwrap().try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
    }

    #[cfg(panic = "unwind")]
    fn assert_visitor_panic_terminates_walk(timed: bool) {
        use std::time::Duration;

        for threads in [1, 4] {
            for capacity in [0, 2] {
                let dir = tempfile::tempdir().unwrap();
                for idx in 0..24 {
                    fs::write(dir.path().join(idx.to_string()), b"data").unwrap();
                }
                let mut walk = visit_roots(
                    [(7, dir.path().to_owned())],
                    threads,
                    Options::default().queued_batches_per_worker(capacity),
                    |_, _| true,
                    |_, _, entries| {
                        assert!(
                            !entries
                                .iter()
                                .any(|entry| entry.as_ref().is_ok_and(|entry| entry.depth > 0)),
                            "visitor failed"
                        );
                    },
                );
                let shared = Arc::clone(&walk.pool.as_ref().unwrap().shared);
                let (done_tx, done_rx) = crossbeam::channel::bounded(1);
                let reader = thread::spawn(move || {
                    let result = walk.recv(timed.then_some(Duration::from_millis(250)));
                    let exhausted = walk.recv(None).unwrap().is_none();
                    done_tx.send((result, exhausted)).unwrap();
                });
                let result = done_rx.recv_timeout(Duration::from_secs(2));
                if result.is_err() {
                    // Unblock the original implementation so the failing regression leaves no hung reader.
                    shared.events.send(Event::Finished).unwrap();
                }
                reader.join().unwrap();
                let (result, exhausted) =
                    result.expect("worker panic must wake a blocked receiver");
                let Some((root, RootEvent::WorkerFailed(error))) =
                    result.expect("worker panic must not be a timeout")
                else {
                    panic!("worker panic must produce a terminal error");
                };
                assert_eq!(root, 7);
                assert_eq!(error.io_error().kind(), io::ErrorKind::Other);
                assert!(error.to_string().contains("visitor failed"));
                assert!(exhausted, "the worker error must terminate receiving");
            }
        }
    }

    fn visit_roots(
        roots: impl IntoIterator<Item = (usize, PathBuf)>,
        threads: usize,
        options: Options,
        descend: impl Fn(usize, &Entry) -> bool + Send + Sync + 'static,
        visit: impl Fn(usize, usize, Vec<Result<Entry, Error>>) + Send + Sync + 'static,
    ) -> MultiWalk {
        MultiWalk::prepare(threads, Order::Completion, options)
            .visit_batches(visit)
            .start_entries(
                roots
                    .into_iter()
                    .map(|(idx, path)| (idx, Entry::from_path(&path, options))),
                descend,
            )
    }

    #[cfg(panic = "unwind")]
    #[test]
    fn visitor_panic_terminates_blocking_walk() {
        assert_visitor_panic_terminates_walk(false);
    }

    #[cfg(panic = "unwind")]
    #[test]
    fn visitor_panic_terminates_timed_walk() {
        assert_visitor_panic_terminates_walk(true);
    }

    #[cfg(panic = "unwind")]
    #[test]
    fn visitor_panic_is_reported_while_another_visitor_is_blocked() {
        use std::time::Duration;

        let dir = tempfile::tempdir().unwrap();
        for root in ["blocked", "panicking"] {
            let path = dir.path().join(root);
            fs::create_dir(&path).unwrap();
            fs::write(path.join("file"), b"data").unwrap();
        }
        let (blocked_tx, blocked_rx) = crossbeam::channel::bounded(1);
        let (release_tx, release_rx) = crossbeam::channel::bounded::<()>(0);
        let (panic_tx, panic_rx) = crossbeam::channel::bounded(1);
        let mut walk = visit_roots(
            [
                (0, dir.path().join("blocked")),
                (1, dir.path().join("panicking")),
            ],
            2,
            Options::default().queued_batches_per_worker(0),
            |_, _| true,
            move |_, root, entries| {
                if entries
                    .iter()
                    .any(|entry| entry.as_ref().is_ok_and(|entry| entry.depth > 0))
                {
                    if root == 0 {
                        blocked_tx.send(()).unwrap();
                        release_rx.recv().ok();
                    } else if panic_rx.recv().is_ok() {
                        panic!("visitor failed");
                    }
                }
            },
        );
        blocked_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        panic_tx.send(()).unwrap();
        let (done_tx, done_rx) = crossbeam::channel::bounded(1);
        let reader = thread::spawn(move || {
            let event = walk.recv(Some(Duration::from_secs(2)));
            let exhausted = matches!(walk.recv(Some(Duration::ZERO)), Ok(None));
            done_tx.send((event, exhausted)).unwrap();
        });
        let result = done_rx.recv_timeout(Duration::from_secs(3));
        drop(release_tx);
        reader.join().unwrap();
        let (event, exhausted) =
            result.expect("failure must be reported before releasing the blocked visitor");
        let Ok(Some((1, RootEvent::WorkerFailed(error)))) = event else {
            panic!("the worker panic must be reported as a terminal error");
        };
        assert!(error.to_string().contains("visitor failed"));
        assert!(exhausted);
    }

    #[test]
    fn visitors_account_roots_and_errors_before_completion() {
        let dir = tempfile::tempdir().unwrap();
        for idx in 0..24 {
            fs::write(dir.path().join(idx.to_string()), b"metadata").unwrap();
        }
        let doomed = dir.path().join("doomed");
        fs::create_dir(&doomed).unwrap();
        let file = tempfile::NamedTempFile::new().unwrap();
        let missing = dir.path().join("missing");
        let initial = Arc::new(AtomicUsize::new(0));
        let counts = Arc::new(std::array::from_fn::<_, 3, _>(|_| AtomicUsize::new(0)));
        let errors = Arc::new(std::array::from_fn::<_, 3, _>(|_| AtomicUsize::new(0)));
        let seen_initial = Arc::clone(&initial);
        let seen_counts = Arc::clone(&counts);
        let seen_errors = Arc::clone(&errors);
        let caller = thread::current().id();
        let removed = doomed.clone();
        let mut walk = visit_roots(
            [
                (0, dir.path().to_owned()),
                (1, file.path().to_owned()),
                (2, missing),
            ],
            4,
            Options::default(),
            move |_, entry| {
                if entry.depth == 1 && entry.path() == removed {
                    fs::remove_dir(&removed).unwrap();
                }
                true
            },
            move |worker, root, entries| {
                assert!(worker < 4);
                for entry in entries {
                    let is_initial = entry.as_ref().map_or(root == 2, |entry| entry.depth == 0);
                    if is_initial {
                        assert_eq!(worker, 0);
                        assert_eq!(thread::current().id(), caller);
                        seen_initial.fetch_add(1, AtomicOrdering::SeqCst);
                    } else {
                        assert_eq!(seen_initial.load(AtomicOrdering::SeqCst), 3);
                    }
                    match entry {
                        Ok(entry) => {
                            entry.metadata.unwrap().unwrap();
                        }
                        Err(_) => {
                            seen_errors[root].fetch_add(1, AtomicOrdering::SeqCst);
                        }
                    }
                    seen_counts[root].fetch_add(1, AtomicOrdering::SeqCst);
                }
            },
        );
        let expected_counts = [27, 1, 1];
        let expected_errors = [1, 0, 1];
        let mut completed = HashSet::new();
        while let Some((root, event)) = walk.recv(Some(std::time::Duration::from_secs(5))).unwrap()
        {
            assert!(
                matches!(event, RootEvent::Finished),
                "entries must be consumed on workers"
            );
            assert!(completed.insert(root));
            assert_eq!(
                counts[root].load(AtomicOrdering::SeqCst),
                expected_counts[root]
            );
            assert_eq!(
                errors[root].load(AtomicOrdering::SeqCst),
                expected_errors[root]
            );
        }
        assert_eq!(completed.len(), 3);
    }

    #[test]
    fn visitor_walk_accounts_initial_entries_during_preparation() {
        let dir = tempfile::tempdir().unwrap();
        let file = tempfile::NamedTempFile::new().unwrap();
        let removed = dir.path().join("removed");
        fs::write(&removed, b"data").unwrap();
        let seen = Arc::new(AtomicUsize::new(0));
        let visits = Arc::clone(&seen);
        let roots = [
            (7, file.path().to_owned()),
            (2, dir.path().to_owned()),
            (9, removed.clone()),
        ];
        let mut walk = MultiWalk::prepare(4, Order::Completion, Options::default())
            .visit_batches(move |worker, root, entries| {
                assert_eq!(worker, 0);
                assert_eq!(entries.len(), 1);
                match root {
                    7 => fs::remove_file(&removed).unwrap(),
                    9 => assert!(
                        entries[0].is_err(),
                        "the last root is queried after the first visit"
                    ),
                    _ => {}
                }
                visits.fetch_add(1, AtomicOrdering::Relaxed);
            })
            .start_paths(roots, move |root, _| {
                assert_eq!(root, 2);
                assert_eq!(seen.load(AtomicOrdering::Relaxed), 1);
                false
            });
        assert!(walk.pool.is_none());
        assert!(matches!(
            &walk.pending,
            Some(Pending::InitialFinished(indices)) if indices.as_slice() == [7, 2, 9]
        ));
        assert!(matches!(
            walk.recv(Some(std::time::Duration::ZERO)),
            Ok(Some((7, RootEvent::Finished)))
        ));
        for root in [2, 9] {
            assert!(
                matches!(walk.recv(None).unwrap(), Some((idx, RootEvent::Finished)) if idx == root)
            );
        }
        assert!(walk.recv(None).unwrap().is_none());
    }

    #[test]
    fn timed_walk_keeps_queued_events_when_empty_batches_exhaust_the_deadline() {
        use std::time::Duration;

        let empty_batches = 1_024;
        let pool = start_pool(
            1,
            Order::ParentFirst,
            Options::default().queued_batches_per_worker(empty_batches + 3),
            0,
        );
        pool.shared
            .events
            .try_send(Event::RootFinished { root_idx: 3 })
            .unwrap();
        for _ in 0..empty_batches {
            pool.shared
                .events
                .try_send(Event::Batch {
                    root_idx: 7,
                    batch: Ok(Vec::new()),
                })
                .unwrap();
        }
        pool.shared
            .events
            .try_send(Event::RootFinished { root_idx: 7 })
            .unwrap();
        pool.shared.events.try_send(Event::Finished).unwrap();
        let mut walk = MultiWalk {
            pending: None,
            pool: Some(pool),
        };

        // A zero timeout still polls for an immediately available real event.
        assert!(matches!(
            walk.recv(Some(Duration::ZERO)),
            Ok(Some((3, RootEvent::Finished)))
        ));
        // Once that poll yields only an empty batch, an expired deadline must stop draining.
        assert!(matches!(
            walk.recv(Some(Duration::ZERO)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        assert!(matches!(
            walk.recv(Some(Duration::from_secs(1))),
            Ok(Some((7, RootEvent::Finished)))
        ));
        // Consuming final completion must finish the walk even at a zero deadline.
        assert!(matches!(walk.recv(Some(Duration::ZERO)), Ok(None)));
        assert!(matches!(walk.recv(Some(Duration::ZERO)), Ok(None)));
    }

    #[test]
    fn completion_waits_for_its_visitor_without_blocking_another_root() {
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        for root in ["blocked", "ready"] {
            fs::create_dir(dir.path().join(root)).unwrap();
            fs::write(dir.path().join(root).join("file"), b"data").unwrap();
        }
        let (started_tx, started_rx) = crossbeam::channel::bounded(1);
        let (release_tx, release_rx) = crossbeam::channel::bounded(1);
        let mut walk = visit_roots(
            [
                (7, dir.path().join("blocked")),
                (9, dir.path().join("ready")),
            ],
            2,
            Options::default(),
            |_, _| true,
            move |_, root, entries| {
                if root == 7
                    && entries
                        .iter()
                        .any(|entry| entry.as_ref().is_ok_and(|entry| entry.depth > 0))
                {
                    started_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                }
            },
        );
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let Some((9, RootEvent::Finished)) = walk.recv(Some(Duration::from_secs(5))).unwrap()
        else {
            panic!("a blocked visitor must not finish or prevent a peer root from finishing");
        };
        assert!(matches!(
            walk.recv(Some(Duration::from_millis(20))),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        release_tx.send(()).unwrap();
        assert!(matches!(
            walk.recv(Some(Duration::from_secs(5))),
            Ok(Some((7, RootEvent::Finished)))
        ));
        assert!(matches!(walk.recv(Some(Duration::from_secs(5))), Ok(None)));
    }

    #[test]
    fn dropping_a_visit_walk_waits_for_active_visitors() {
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file"), b"data").unwrap();
        let (started_tx, started_rx) = crossbeam::channel::bounded(1);
        let (release_tx, release_rx) = crossbeam::channel::bounded(1);
        let walk = visit_roots(
            [(0, dir.path().to_owned())],
            1,
            Options::default(),
            |_, _| true,
            move |_, _, entries| {
                if entries
                    .iter()
                    .any(|entry| entry.as_ref().is_ok_and(|entry| entry.depth > 0))
                {
                    started_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                }
            },
        );
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let (done_tx, done_rx) = crossbeam::channel::bounded(1);
        let dropping = thread::spawn(move || {
            drop(walk);
            done_tx.send(()).unwrap();
        });
        assert!(done_rx.recv_timeout(Duration::from_millis(20)).is_err());
        release_tx.send(()).unwrap();
        done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        dropping.join().unwrap();
    }

    #[test]
    fn multi_walk_forwards_worker_batches_in_order() {
        let (roots, mut walk) =
            MultiWalk::prepare(1, Order::ParentFirst, Options::default()).stream_roots();
        let events = &roots.shared.events;
        let send_batch = |entries| {
            events
                .try_send(Event::Batch {
                    root_idx: 7,
                    batch: Ok(entries),
                })
                .unwrap();
        };
        let errors = || {
            vec![
                Err(io::Error::from(io::ErrorKind::NotFound).into()),
                Err(io::Error::from(io::ErrorKind::PermissionDenied).into()),
            ]
        };

        send_batch(Vec::new());
        let entries = errors();
        let original = entries.as_ptr();
        send_batch(entries);
        let Some((7, RootEvent::Entries(entries))) = walk.recv(None).unwrap() else {
            panic!("the walk must skip the empty batch and return the worker batch");
        };
        assert_eq!(entries.as_ptr(), original);
        assert_eq!(entries.len(), 2);

        assert!(matches!(
            &entries[..],
            [Err(first), Err(second)] if first.io_error().kind() == io::ErrorKind::NotFound
                && second.io_error().kind() == io::ErrorKind::PermissionDenied
        ));

        events
            .try_send(Event::Batch {
                root_idx: 7,
                batch: Err(io::Error::from(io::ErrorKind::NotFound).into()),
            })
            .unwrap();
        events
            .try_send(Event::RootFinished { root_idx: 7 })
            .unwrap();
        assert!(matches!(
            walk.recv(None).unwrap(),
            Some((7, RootEvent::Entries(entries)))
                if matches!(&entries[..], [Err(error)] if error.io_error().kind() == io::ErrorKind::NotFound)
        ));
        assert!(matches!(
            walk.recv(None).unwrap(),
            Some((7, RootEvent::Finished))
        ));
        events.try_send(Event::Finished).unwrap();
        assert!(walk.recv(None).unwrap().is_none());
    }

    #[test]
    fn result_queue_capacity_is_configurable_per_worker() {
        for threads in [1, 2] {
            for (options, batches) in [
                (Options::default(), 2),
                (Options::default().queued_batches_per_worker(0), 0),
                (Options::default().queued_batches_per_worker(1), 1),
                (Options::default().queued_batches_per_worker(128), 128),
            ] {
                let pool = start_pool(threads, Order::ParentFirst, options, 0);
                for _ in 0..threads * batches {
                    pool.shared.events.try_send(Event::Finished).unwrap();
                }
                assert!(matches!(
                    pool.shared.events.try_send(Event::Finished),
                    Err(std::sync::mpsc::TrySendError::Full(_))
                ));
            }
        }
    }

    #[test]
    fn roots_added_during_a_blocked_walk_finish_independently() {
        use std::time::Duration;

        let directory = tempfile::tempdir().unwrap();
        let slow = directory.path().join("slow");
        let fast = directory.path().join("fast");
        fs::create_dir(&slow).unwrap();
        fs::create_dir(&fast).unwrap();
        let (started_tx, started_rx) = crossbeam::channel::bounded(1);
        let (release_tx, release_rx) = crossbeam::channel::bounded(1);
        let (mut roots, walk) =
            MultiWalk::prepare(2, Order::ParentFirst, Options::default()).stream_roots();
        assert_eq!(walk.pool.as_ref().unwrap().handles.len(), 2);
        let (finished_tx, finished_rx) = crossbeam::channel::unbounded();
        let reader = thread::spawn(move || {
            let mut walk = walk;
            while let Some((index, batch)) = walk.recv(None).unwrap() {
                match batch {
                    RootEvent::Entries(entries) => {
                        for entry in entries {
                            entry.unwrap();
                        }
                    }
                    RootEvent::Finished => finished_tx.send(index).unwrap(),
                    RootEvent::WorkerFailed(error) => panic!("worker failed: {error}"),
                }
            }
        });
        roots
            .add_root(0, slow, move |entry| {
                if entry.depth == 0 {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                }
                true
            })
            .unwrap();
        let started = started_rx.recv_timeout(Duration::from_secs(5));
        roots.add_root(1, fast.clone(), |_| true).unwrap();
        let first_finished = finished_rx.recv_timeout(Duration::from_secs(5));
        release_tx.send(()).unwrap();
        let second_finished = finished_rx.recv_timeout(Duration::from_secs(5));

        // Finishing every current root must leave the pool available for later submissions.
        roots.add_root(2, fast, |_| true).unwrap();
        drop(roots);
        let last_finished = finished_rx.recv_timeout(Duration::from_secs(5));
        reader.join().unwrap();
        assert!(started.is_ok());
        assert_eq!(first_finished.unwrap(), 1);
        assert_eq!(second_finished.unwrap(), 0);
        assert_eq!(last_finished.unwrap(), 2);
        assert!(finished_rx.recv().is_err());
    }

    #[test]
    fn submitted_roots_are_not_starved_by_existing_subdirectories() {
        use std::time::Duration;

        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("old");
        let new = directory.path().join("new");
        fs::create_dir_all(old.join("child/grandchild")).unwrap();
        fs::create_dir(&new).unwrap();
        let (started_tx, started_rx) = crossbeam::channel::bounded(1);
        let (release_tx, release_rx) = crossbeam::channel::bounded(1);
        let (order_tx, order_rx) = crossbeam::channel::unbounded();
        let old_order = order_tx.clone();
        let (mut roots, walk) =
            MultiWalk::prepare(1, Order::ParentFirst, Options::default()).stream_roots();
        let reader = thread::spawn(move || {
            let mut walk = walk;
            while walk.recv(None).unwrap().is_some() {}
        });
        roots
            .add_root(0, old, move |entry| {
                if entry.depth == 1 {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                } else if entry.depth == 2 {
                    old_order.send("old descendants").unwrap();
                }
                true
            })
            .unwrap();
        let started = started_rx.recv_timeout(Duration::from_secs(5));
        roots
            .add_root(1, new, move |_| {
                order_tx.send("new root").unwrap();
                true
            })
            .unwrap();
        drop(roots);
        release_tx.send(()).unwrap();
        let next = order_rx.recv_timeout(Duration::from_secs(5));
        reader.join().unwrap();
        assert!(started.is_ok());
        assert_eq!(next.unwrap(), "new root");
    }

    #[test]
    fn streaming_input_closes_and_cancels_without_draining_output() {
        use std::time::Duration;

        let directory = tempfile::tempdir().unwrap();
        let (mut roots, walk) =
            MultiWalk::prepare(1, Order::ParentFirst, Options::default()).stream_roots();
        let (started_tx, started_rx) = crossbeam::channel::unbounded();
        let (release_tx, release_rx) = crossbeam::channel::bounded(1);
        for index in 0..8 {
            let started_tx = started_tx.clone();
            let release_rx = release_rx.clone();
            roots
                .add_root(index, directory.path().to_owned(), move |_| {
                    if index == 0 {
                        release_rx.recv().unwrap();
                    }
                    started_tx.send(()).unwrap();
                    true
                })
                .unwrap();
        }
        // Queue every root before allowing the worker to choose between roots and local work.
        release_tx.send(()).unwrap();
        // Two root batches fill the output; the third reaches its predicate before blocking.
        for _ in 0..3 {
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        let (dropped_tx, dropped_rx) = crossbeam::channel::bounded(1);
        let closer = thread::spawn(move || {
            drop(roots);
            drop(walk);
            dropped_tx.send(()).unwrap();
        });
        dropped_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("closing input and dropping the walker cannot block on output");
        closer.join().unwrap();

        let (roots, mut walk) =
            MultiWalk::prepare(1, Order::ParentFirst, Options::default()).stream_roots();
        drop(roots);
        assert!(walk.recv(None).unwrap().is_none());
        let (mut roots, walk) =
            MultiWalk::prepare(1, Order::ParentFirst, Options::default()).stream_roots();
        roots
            .add_root(usize::MAX, directory.path().to_owned(), |_| true)
            .unwrap();
        assert_eq!(
            roots
                .add_root(usize::MAX, directory.path().to_owned(), |_| true)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        drop(walk);
        assert_eq!(
            roots
                .add_root(0, directory.path().to_owned(), |_| true)
                .unwrap_err()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }

    #[test]
    fn dropping_a_walk_disconnects_workers_blocked_on_full_output() {
        use std::{sync::mpsc::channel, time::Duration};

        let dir = tempfile::tempdir().unwrap();
        let roots = [dir.path().join("first"), dir.path().join("second")];
        for root in &roots {
            fs::create_dir_all(root.join("child")).unwrap();
        }
        for order in [Order::ParentFirst, Order::Completion] {
            let (started_tx, started_rx) = channel();
            let walk = MultiWalk::prepare(1, order, Options::default()).start_paths(
                roots.clone().into_iter().enumerate(),
                move |_, entry| {
                    if entry.depth == 0 {
                        true
                    } else {
                        started_tx.send(()).unwrap();
                        false
                    }
                },
            );
            // One worker has two output slots: the first root's batch and completion fill them.
            // Reaching the second root guarantees its worker must send to the full queue next.
            for _ in &roots {
                started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            }
            let (dropped_tx, dropped_rx) = channel();
            thread::spawn(move || {
                drop(walk);
                dropped_tx.send(()).unwrap();
            });
            dropped_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("dropping a walk must unblock workers before joining them");
        }
    }

    #[test]
    fn cancelling_a_walk_waits_for_active_reads_and_discards_buffered_entries() {
        use std::time::{Duration, Instant};

        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("child")).unwrap();
        let (started_tx, started_rx) = crossbeam::channel::bounded(1);
        let (release_tx, release_rx) = crossbeam::channel::bounded(1);
        let mut walk = walk(
            dir.path(),
            1,
            Order::ParentFirst,
            Options::default().skip_metadata(),
            move |entry| {
                if entry.depth == 1 {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                }
                true
            },
        );
        let cancelled = Arc::new(AtomicBool::new(false));
        assert!(walk.next_cancellable(&cancelled).unwrap().is_ok());
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let shared = Arc::clone(&walk.pool.as_ref().unwrap().shared);
        let (done_tx, done_rx) = crossbeam::channel::bounded(1);
        let reader_cancelled = Arc::clone(&cancelled);
        let reader = thread::spawn(move || {
            assert!(walk.next_cancellable(&reader_cancelled).is_none());
            assert!(walk.next.is_empty());
            assert!(walk.pool.is_none());
            assert!(!walk.restart());
            assert!(walk.next().is_none());
            done_tx.send(()).unwrap();
        });

        assert!(done_rx.recv_timeout(Duration::from_millis(20)).is_err());
        cancelled.store(true, AtomicOrdering::Relaxed);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !shared.stop.load(AtomicOrdering::Relaxed) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        let stopped_without_output = shared.stop.load(AtomicOrdering::Relaxed);
        let still_joining = done_rx.try_recv().is_err();
        release_tx.send(()).unwrap();
        done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        reader.join().unwrap();

        assert!(
            stopped_without_output,
            "cancellation must wake a waiting reader"
        );
        assert!(
            still_joining,
            "cancellation must wait for active worker calls"
        );
    }

    #[test]
    fn cancelling_a_walk_disconnects_workers_blocked_on_full_output() {
        use std::time::Duration;

        let dir = tempfile::tempdir().unwrap();
        for name in ["first", "second", "third"] {
            fs::create_dir_all(dir.path().join(name).join("child")).unwrap();
        }
        let (started_tx, started_rx) = crossbeam::channel::unbounded();
        let mut walk = walk(
            dir.path(),
            1,
            Order::ParentFirst,
            Options::default().skip_metadata(),
            move |entry| {
                if entry.depth == 2 {
                    started_tx.send(()).unwrap();
                    return false;
                }
                true
            },
        );
        // The root-directory batch and first child-directory batch fill the two output slots.
        // The second child directory then reaches its predicate before sending to the full queue.
        for _ in 0..2 {
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        let (done_tx, done_rx) = crossbeam::channel::bounded(1);
        let reader = thread::spawn(move || {
            assert!(walk.next_cancellable(&AtomicBool::new(true)).is_none());
            assert!(walk.next.is_empty(), "the buffered root is also discarded");
            done_tx.send(()).unwrap();
        });
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("cancellation disconnects full output before joining workers");
        reader.join().unwrap();
    }

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

        let mut walk = MultiWalk::prepare(2, Order::Completion, Options::default())
            .start_paths(roots.iter().cloned().enumerate(), |_, _| true);
        let events = std::iter::from_fn(|| walk.recv(None).unwrap());
        let mut paths = Vec::new();
        let mut last_entry = [0; 2];
        let mut finished = [None; 2];
        for (position, (root_idx, event)) in events.enumerate() {
            match event {
                RootEvent::Entries(entries) => {
                    last_entry[root_idx] = position;
                    paths.extend(entries.into_iter().map(|entry| {
                        (
                            root_idx,
                            entry
                                .unwrap()
                                .path()
                                .strip_prefix(&roots[root_idx])
                                .unwrap()
                                .to_owned(),
                        )
                    }));
                }
                RootEvent::Finished => finished[root_idx] = Some(position),
                RootEvent::WorkerFailed(error) => panic!("worker failed: {error}"),
            }
        }
        paths[roots.len()..].sort();
        assert_eq!(
            paths,
            [
                (0, PathBuf::new()),
                (1, PathBuf::new()),
                (0, PathBuf::from("child")),
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

        let mut events = MultiWalk::prepare(2, Order::ParentFirst, Options::default())
            .start_entries([(7, Ok(descendant_entry))], |root_idx, entry| {
                assert_eq!(root_idx, 7);
                assert_eq!(entry.depth, 0, "the predicate should see a re-rooted entry");
                true
            });

        let Some((7, RootEvent::Entries(entries))) = events.recv(None).unwrap() else {
            panic!("the prepared descendant should be emitted as the new root");
        };
        let Ok([Ok(mut root)]) = <[_; 1]>::try_from(entries) else {
            panic!("expected a batch with one entry");
        };
        assert_eq!(root.path(), descendant);
        assert_eq!(
            root.depth, 0,
            "the entry originally found at depth 1 must become the new traversal root"
        );

        let Some((7, RootEvent::Entries(entries))) = events.recv(None).unwrap() else {
            panic!("the re-rooted directory should emit its child");
        };
        let Ok([Ok(entry)]) = <[_; 1]>::try_from(entries) else {
            panic!("expected a batch with one entry");
        };
        assert_eq!(entry.path(), child);
        assert_eq!(
            entry.depth, 1,
            "the child depth must be relative to the prepared entry used as the new root"
        );
        assert!(matches!(
            events.recv(None).unwrap(),
            Some((7, RootEvent::Finished))
        ));
        assert_eq!(
            events.recv(None).unwrap().map(|(root_idx, _)| root_idx),
            None,
            "nothing left after the Finished event"
        );

        root.metadata = Some(Err(io::Error::from(io::ErrorKind::PermissionDenied)));
        let mut events = MultiWalk::prepare(2, Order::ParentFirst, Options::default())
            .start_entries([(7, Ok(root))], |_, _| {
                panic!("a directory with inaccessible metadata must not be descended")
            });
        let Some((7, RootEvent::Entries(entries))) = events.recv(None).unwrap() else {
            panic!("the prepared directory must retain its metadata error");
        };
        let Ok([Ok(root)]) = <[_; 1]>::try_from(entries) else {
            panic!("expected a batch with one entry");
        };
        let error = root
            .metadata
            .unwrap()
            .err()
            .expect("the inaccessible root must retain its metadata error");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(matches!(
            events.recv(None).unwrap(),
            Some((7, RootEvent::Finished))
        ));
        assert_eq!(
            events.recv(None).unwrap().map(|(root_idx, _)| root_idx),
            None
        );
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

        let mut events = MultiWalk::prepare(1, Order::Completion, Options::default())
            .start_entries([(7, Ok(entry))], |_, _| true);
        let Some((7, RootEvent::Entries(entries))) = events.recv(None).unwrap() else {
            panic!(
                "prepared root must be yielded without querying its removed path which would fail"
            );
        };
        let Ok([Ok(entry)]) = <[_; 1]>::try_from(entries) else {
            panic!("expected a batch with one entry");
        };
        assert_eq!(entry.path(), path);
        assert_eq!(entry.metadata.unwrap().unwrap().len(), expected_len);
        assert!(matches!(
            events.recv(None).unwrap(),
            Some((7, RootEvent::Finished))
        ));
        assert_eq!(
            events.recv(None).unwrap().map(|(root_idx, _)| root_idx),
            None
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn small_directory_shares_metadata_before_a_slow_callback_returns() {
        use std::time::{Duration, Instant};

        let dir = tempfile::tempdir().unwrap();
        for idx in 0..32 {
            fs::create_dir(dir.path().join(idx.to_string())).unwrap();
        }
        let (started_tx, started_rx) = crossbeam::channel::unbounded();
        let (release_tx, release_rx) = crossbeam::channel::unbounded();
        let descend: Arc<Descend> = Arc::new(move |_, entry| {
            if entry.depth == 1 {
                started_tx.send(thread::current().id()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            true
        });
        let options = Options::default();
        let mut next = Vec::new();
        let (jobs, next_directory_id) = begin_walks(
            [(0, Entry::from_path(dir.path(), options))],
            &descend,
            |root_idx, entry, finished| {
                assert!(!finished);
                next.push((root_idx, InitialEvent::Entry(entry)));
            },
        );
        let pool = start_pool(8, Order::Completion, options, next_directory_id);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !pool
            .shared
            .idle
            .iter()
            .all(|idle| idle.load(AtomicOrdering::Acquire))
        {
            assert!(Instant::now() < deadline, "workers must become idle");
            thread::sleep(Duration::from_millis(1));
        }
        start_jobs(&pool, jobs);
        let walk = MultiWalk {
            pending: Some(Pending::Initial(next.into_iter())),
            pool: Some(pool),
        };
        let mut workers = HashSet::new();
        for _ in 0..4 {
            match started_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(worker) => {
                    workers.insert(worker);
                }
                Err(_) => break,
            }
        }
        // Release every callback before asserting, including on the regressing implementation.
        for _ in 0..32 {
            release_tx.send(()).unwrap();
        }
        let mut walk = walk;
        let mut entries = 0;
        while let Some((_, batch)) = walk.recv(None).unwrap() {
            match batch {
                RootEvent::Entries(batch) => {
                    entries += batch.into_iter().filter(Result::is_ok).count()
                }
                RootEvent::Finished => {}
                RootEvent::WorkerFailed(error) => panic!("worker failed: {error}"),
            }
        }
        assert_eq!(entries, 33);
        assert!(
            workers.len() >= 4,
            "idle workers must share a 32-entry metadata job before its blocked callback returns"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn wide_metadata_walk_amortizes_result_transfers() {
        let dir = tempfile::tempdir().unwrap();
        let count = 1_024;
        for idx in 0..count {
            fs::write(dir.path().join(idx.to_string()), b"metadata").unwrap();
        }

        let mut walk = walk(dir.path(), 4, Order::Completion, Options::default(), |_| {
            true
        });
        let root = walk.next().unwrap().unwrap();
        assert_eq!(root.depth, 0);
        assert_eq!(root.directory_id, Some(DirectoryId::new(0)));
        assert_eq!(root.parent_directory_id, None);

        let pool = walk.pool.as_ref().unwrap();
        let mut names = HashSet::new();
        let mut batches = 0;
        let mut root_finished = false;
        loop {
            match pool
                .events
                .as_ref()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("the metadata walk must finish")
            {
                Event::Batch { root_idx, batch } => {
                    assert_eq!(root_idx, 0);
                    assert!(!root_finished, "entries must precede root completion");
                    batches += 1;
                    for entry in batch.unwrap() {
                        let entry = entry.unwrap();
                        assert_eq!(entry.depth, 1);
                        assert_eq!(entry.parent_path.as_ref(), dir.path());
                        assert_eq!(entry.directory_id, None);
                        assert_eq!(entry.parent_directory_id, Some(DirectoryId::new(0)));
                        assert!(entry.file_type.is_file());
                        assert_eq!(entry.metadata.unwrap().unwrap().len(), 8);
                        assert!(names.insert(entry.file_name), "entries must be unique");
                    }
                }
                Event::RootFinished { root_idx } => {
                    assert_eq!(root_idx, 0);
                    assert!(!root_finished, "root completion must be emitted once");
                    assert_eq!(names.len(), count);
                    root_finished = true;
                }
                Event::WorkerFailed { error, .. } => panic!("metadata worker failed: {error}"),
                Event::Finished => break,
            }
        }

        assert!(
            root_finished,
            "root completion must precede pool completion"
        );
        for idx in 0..count {
            assert!(names.contains(&OsString::from(idx.to_string())));
        }
        assert!(
            batches <= 32,
            "a wide local directory must amortize result-channel transfers: \
             {count} entries required {batches} batches"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn completion_walk_publishes_a_directory_before_its_child_read() {
        let dir = tempfile::tempdir().unwrap();
        let options = Options::default();
        let entry = Entry::from_path(dir.path(), options).unwrap();
        let pool = start_pool(1, Order::Completion, options, 1);
        let worker = Worker::new_lifo();
        let root = Arc::new(Root {
            index: 7,
            pending: AtomicUsize::new(1),
            descend: Arc::new(|_, _| true),
        });
        let child_read = Job::ReadDir {
            root: Arc::clone(&root),
            path: Arc::from(dir.path()),
            directory_id: 0,
            entry_depth: 1,
        };
        assert!(publish_directory(
            &root,
            Ok(vec![Ok(entry)]),
            vec![child_read],
            &worker,
            &pool.shared,
            &mut Output::default(),
        ));
        let event = pool.events.as_ref().unwrap().try_recv();
        let Ok(Event::Batch { root_idx: 7, batch }) = event else {
            panic!("the directory must be available before the worker runs the queued child read");
        };
        let entries = batch.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].as_ref().unwrap().path(), dir.path());
        assert!(matches!(worker.pop(), Some(Job::ReadDir { .. })));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn completion_walk_coalesces_small_directories_and_preserves_errors() {
        let dir = tempfile::tempdir().unwrap();
        let mut expected = HashSet::new();
        for idx in 0..128 {
            let relative = PathBuf::from(format!("d{idx:03}"));
            fs::create_dir(dir.path().join(&relative)).unwrap();
            expected.insert(relative.clone());
            for file in 0..4 {
                let relative = relative.join(file.to_string());
                fs::write(dir.path().join(&relative), b"file").unwrap();
                expected.insert(relative);
            }
        }
        let vanishing = dir.path().join("vanishing");
        fs::create_dir(&vanishing).unwrap();
        expected.insert(PathBuf::from("vanishing"));
        let remove_before_read = vanishing.clone();
        let mut walk = walk(
            dir.path(),
            1,
            Order::Completion,
            Options::default(),
            move |entry| {
                // Metadata is already collected, but the queued child read must fail.
                if entry.depth == 1 && entry.path() == remove_before_read {
                    fs::remove_dir(&remove_before_read).unwrap();
                }
                true
            },
        );
        assert_eq!(walk.next().unwrap().unwrap().depth, 0);
        let pool = walk.pool.as_ref().unwrap();
        let mut paths = HashSet::new();
        let mut errors = 0;
        let mut batches = 0;
        let mut root_finished = false;
        loop {
            match pool
                .events
                .as_ref()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("small-directory results and completion must be flushed")
            {
                Event::Batch { root_idx, batch } => {
                    assert_eq!(root_idx, 0);
                    assert!(!root_finished, "entries and errors must precede completion");
                    batches += 1;
                    let entries = match batch {
                        Ok(entries) => entries,
                        Err(err) => vec![Err(err)],
                    };
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                let path = entry.path();
                                let relative = path.strip_prefix(dir.path()).unwrap().to_owned();
                                assert!(expected.contains(&relative));
                                assert!(paths.insert(relative), "entries must be unique");
                                let metadata = entry.metadata.unwrap().unwrap();
                                if entry.file_type.is_file() {
                                    assert_eq!(entry.depth, 2);
                                    assert_eq!(metadata.len(), 4);
                                } else {
                                    assert!(entry.file_type.is_dir());
                                    assert_eq!(entry.depth, 1);
                                }
                            }
                            Err(err) => {
                                assert_eq!(err.path(), Some(vanishing.as_path()));
                                assert_eq!(err.io_error().kind(), io::ErrorKind::NotFound);
                                errors += 1;
                            }
                        }
                    }
                }
                Event::RootFinished { root_idx } => {
                    assert_eq!(root_idx, 0);
                    assert!(!root_finished, "root completion must be emitted once");
                    assert_eq!(paths, expected);
                    assert_eq!(errors, 1);
                    root_finished = true;
                }
                Event::WorkerFailed { error, .. } => panic!("metadata worker failed: {error}"),
                Event::Finished => {
                    assert!(
                        root_finished,
                        "root completion must precede pool completion"
                    );
                    break;
                }
            }
        }
        assert!(
            batches < 32,
            "small directories must share result transfers: 128 directories required {batches} batches"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn completion_walk_keeps_ancestors_visible_to_descend_callbacks() {
        use std::time::Duration;

        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("child/grandchild")).unwrap();
        let (consumed_tx, consumed_rx) = crossbeam::channel::bounded(1);
        let released = Arc::new(AtomicBool::new(false));
        let callback_released = Arc::clone(&released);
        let cancelled = Arc::new(AtomicBool::new(false));
        let timeout_cancelled = Arc::clone(&cancelled);
        let (done_tx, done_rx) = crossbeam::channel::bounded(1);
        let timeout = thread::spawn(move || {
            if done_rx.recv_timeout(Duration::from_secs(5)).is_err() {
                timeout_cancelled.store(true, AtomicOrdering::Relaxed);
            }
        });
        let mut walk = walk(
            dir.path(),
            1,
            Order::Completion,
            Options::default(),
            move |entry| {
                if entry.depth == 2 {
                    callback_released.store(
                        consumed_rx.recv_timeout(Duration::from_secs(5)).is_ok(),
                        AtomicOrdering::Relaxed,
                    );
                }
                true
            },
        );
        let mut paths = Vec::new();
        while let Some(entry) = walk.next_cancellable(&cancelled) {
            let entry = entry.unwrap();
            if entry.depth == 1 {
                consumed_tx.try_send(()).unwrap();
            }
            paths.push(entry.path().strip_prefix(dir.path()).unwrap().to_owned());
        }
        done_tx.send(()).ok();
        timeout.join().unwrap();
        assert!(
            !cancelled.load(AtomicOrdering::Relaxed),
            "completion output must remain available while the next predicate waits"
        );
        assert!(
            released.load(AtomicOrdering::Relaxed),
            "the child must reach the consumer before its descendant's predicate times out"
        );
        paths.sort();
        assert_eq!(paths, ["", "child", "child/grandchild"].map(PathBuf::from));
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

        let pool = start_pool(1, Order::ParentFirst, Options::default(), 1);
        let root = Arc::new(Root {
            index: 7,
            pending: AtomicUsize::new(0),
            descend: Arc::new(|_, _| true),
        });
        // Leave this queue unconsumed to model workers stalled on metadata lookups.
        let worker = Worker::new_lifo();
        let path = Arc::from(dir.path());
        #[cfg(target_os = "macos")]
        let entries = read_dir(dir.path(), Options::default().skip_metadata()).unwrap();
        #[cfg(not(target_os = "macos"))]
        let entries = fs::read_dir(dir.path()).unwrap();
        let mut entries = entries.map(Result::unwrap);
        let mut output = Output::default();
        for _ in 0..=MAX_QUEUED_STAT_JOBS {
            schedule_stat_entries(
                &root,
                &path,
                0,
                1,
                entries.by_ref().take(ENTRY_CHUNK_SIZE).collect(),
                &worker,
                &pool.shared,
                &mut output,
            );
        }

        assert_eq!(worker.len(), MAX_QUEUED_STAT_JOBS);
        let Event::Batch { root_idx, batch } = pool.events.as_ref().unwrap().try_recv().unwrap()
        else {
            panic!("a full queue must process the next metadata batch inline");
        };
        assert_eq!(root_idx, 7);
        let entries = batch.unwrap();
        assert_eq!(entries.len(), ENTRY_CHUNK_SIZE);
        for entry in entries {
            let entry = entry.unwrap();
            assert_eq!(entry.metadata.unwrap().unwrap().len(), 8);
            assert_eq!(entry.parent_directory_id, Some(DirectoryId::new(0)));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn stopping_during_a_metadata_batch_skips_remaining_entries() {
        let dir = tempfile::tempdir().unwrap();
        for child in ["first", "second"] {
            fs::create_dir(dir.path().join(child)).unwrap();
        }
        let (shared_tx, shared_rx) = crossbeam::channel::bounded::<Arc<PoolShared>>(1);
        let mut walk = walk(
            dir.path(),
            1,
            Order::Completion,
            Options::default().queued_batches_per_worker(3),
            move |entry| {
                let id = entry.directory_id.unwrap().index();
                if id == 1 {
                    shared_rx
                        .recv()
                        .unwrap()
                        .stop
                        .store(true, AtomicOrdering::Relaxed);
                }
                id == 0
            },
        );
        let pool = walk.pool.as_mut().unwrap();
        shared_tx.send(Arc::clone(&pool.shared)).unwrap();
        pool.handles.pop().unwrap().join().unwrap();
        assert_eq!(
            pool.shared.next_directory_id.load(AtomicOrdering::Relaxed),
            2,
            "only the first directory should have been converted"
        );
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
                    .as_ref()
                    .unwrap()
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
