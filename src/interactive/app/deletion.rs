use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use crossbeam::channel::{Receiver, Sender, bounded, never};

use crate::interactive::widgets::MarkMode;

#[derive(Debug)]
pub(super) enum DeletionEvent {
    /// The path was removed, or disappeared after the directory walk found it.
    Removed { target: usize, path: PathBuf },
    /// Work on this target stopped, possibly because cancellation was requested.
    TargetFinished { target: usize, errors: usize },
    Finished {
        entries: usize,
        errors: usize,
        cancelled: bool,
    },
}

pub(super) struct DeletionTask {
    pub events: Receiver<DeletionEvent>,
    pub cancel: Arc<AtomicBool>,
    pub started: Instant,
    pub mode: MarkMode,
    handle: Option<JoinHandle<()>>,
}

impl DeletionTask {
    #[cfg(test)]
    pub fn from_events(events: Receiver<DeletionEvent>) -> Self {
        Self {
            events,
            cancel: Arc::new(AtomicBool::new(false)),
            started: Instant::now(),
            mode: MarkMode::Delete,
            handle: None,
        }
    }

    pub fn start(paths: Vec<PathBuf>, threads: usize, mode: MarkMode) -> io::Result<Self> {
        let started = Instant::now();
        let (send, events) = bounded(1024);
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let handle = thread::Builder::new()
            .name("dua-delete".into())
            .spawn(move || delete_paths(paths, threads, mode, &send, &worker_cancel))?;
        Ok(Self {
            events,
            cancel,
            started,
            mode,
            handle: Some(handle),
        })
    }

    /// Join after consuming `Finished`, so a full event channel cannot block the worker.
    pub fn join(&mut self) -> thread::Result<()> {
        self.handle.take().map_or(Ok(()), JoinHandle::join)
    }
}

impl Drop for DeletionTask {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        // Unblock publishers before waiting for in-flight filesystem calls to finish.
        drop(std::mem::replace(&mut self.events, never()));
        let _ = self.join();
    }
}

#[derive(Default)]
struct RemovalStats {
    entries: usize,
    errors: usize,
}

fn delete_paths(
    paths: Vec<PathBuf>,
    threads: usize,
    mode: MarkMode,
    events: &Sender<DeletionEvent>,
    cancel: &AtomicBool,
) {
    let mut total = RemovalStats::default();
    for (target, path) in paths.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let stats = match mode {
            MarkMode::Delete => {
                delete_directory_recursively(&path, threads, target, events, cancel)
            }
            #[cfg(feature = "trash-move")]
            MarkMode::Trash => {
                let mut stats = RemovalStats::default();
                if trash::delete(&path).is_ok() {
                    stats.entries = 1;
                    publish_removed(&path, target, events, cancel);
                } else {
                    stats.errors = 1;
                }
                stats
            }
        };
        total.entries += stats.entries;
        total.errors += stats.errors;
        if events
            .send(DeletionEvent::TargetFinished {
                target,
                errors: stats.errors,
            })
            .is_err()
        {
            cancel.store(true, Ordering::Relaxed);
            break;
        }
    }
    let _ = events.send(DeletionEvent::Finished {
        entries: total.entries,
        errors: total.errors,
        cancelled: cancel.load(Ordering::Relaxed),
    });
}

/// Walk without metadata or following symlinks, unlink files in parallel, then remove
/// directories deepest-first. The walker finishes before the unlink worker pool starts.
fn delete_directory_recursively(
    path: &Path,
    threads: usize,
    target: usize,
    events: &Sender<DeletionEvent>,
    cancel: &AtomicBool,
) -> RemovalStats {
    let mut stats = RemovalStats::default();
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let mut walk = dua_core::walk(
        path,
        threads,
        dua_core::Order::Completion,
        dua_core::Options::default().skip_metadata(),
        |_| true,
    );
    while let Some(entry) = walk.next_cancellable(cancel) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if entry.file_type.is_dir() {
                    dirs.push((path, entry.depth));
                } else {
                    files.push(path);
                }
            }
            Err(_) => stats.errors += 1,
        }
    }
    drop(walk);
    if cancel.load(Ordering::Relaxed) {
        return stats;
    }

    let next_file = AtomicUsize::new(0);
    let file_stats = thread::scope(|scope| {
        let handles = (0..threads.max(1).min(files.len()))
            .map(|_| {
                scope.spawn(|| {
                    let mut total = RemovalStats::default();
                    while let Some(path) = files.get(next_file.fetch_add(1, Ordering::Relaxed)) {
                        if cancel.load(Ordering::Relaxed) {
                            break;
                        }
                        let result = fs::remove_file(path);
                        // Windows directory symlinks and junctions require RemoveDirectory.
                        #[cfg(windows)]
                        let result = result.or_else(|err| {
                            if cancel.load(Ordering::Relaxed) {
                                Err(err)
                            } else {
                                fs::remove_dir(path)
                            }
                        });
                        record_removal(result, path, target, events, cancel, &mut total);
                    }
                    total
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("deletion worker does not panic"))
            .fold(RemovalStats::default(), |mut total, stats| {
                total.entries += stats.entries;
                total.errors += stats.errors;
                total
            })
    });
    stats.entries += file_stats.entries;
    stats.errors += file_stats.errors;

    dirs.sort_by_key(|(_, depth)| std::cmp::Reverse(*depth));
    for (dir, _) in dirs {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let result = fs::remove_dir(&dir).or_else(|err| {
            if cancel.load(Ordering::Relaxed) {
                Err(err)
            } else {
                fs::remove_file(&dir)
            }
        });
        record_removal(result, &dir, target, events, cancel, &mut stats);
    }
    stats
}

fn record_removal(
    result: io::Result<()>,
    path: &Path,
    target: usize,
    events: &Sender<DeletionEvent>,
    cancel: &AtomicBool,
    stats: &mut RemovalStats,
) {
    match result {
        Ok(()) => {
            stats.entries += 1;
            publish_removed(path, target, events, cancel);
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            publish_removed(path, target, events, cancel);
        }
        Err(_) => stats.errors += 1,
    }
}

fn publish_removed(
    path: &Path,
    target: usize,
    events: &Sender<DeletionEvent>,
    cancel: &AtomicBool,
) {
    if events
        .send(DeletionEvent::Removed {
            target,
            path: path.to_owned(),
        })
        .is_err()
    {
        cancel.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn finish(task: &mut DeletionTask) -> Vec<DeletionEvent> {
        let events = task.events.iter().collect();
        task.join().expect("deletion worker completes");
        events
    }

    fn assert_finished(events: &[DeletionEvent], entries: usize, errors: usize, cancelled: bool) {
        assert!(matches!(
            events.last(),
            Some(DeletionEvent::Finished {
                entries: actual_entries,
                errors: actual_errors,
                cancelled: actual_cancelled,
            }) if (*actual_entries, *actual_errors, *actual_cancelled) == (entries, errors, cancelled)
        ));
    }

    #[test]
    fn removes_a_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();

        let mut task = DeletionTask::start(vec![file.clone()], 1, MarkMode::Delete).unwrap();
        let events = finish(&mut task);

        assert_finished(&events, 1, 0, false);
        assert!(matches!(&events[0], DeletionEvent::Removed { target: 0, path } if *path == file));
        assert!(!file.exists());
    }

    #[test]
    fn removes_a_nested_tree() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("top.txt"), b"12345").unwrap();
        fs::write(nested.join("deep.txt"), b"abc").unwrap();

        let mut task = DeletionTask::start(vec![root.clone()], 2, MarkMode::Delete).unwrap();
        let events = finish(&mut task);

        assert_finished(&events, 4, 0, false);
        let removed = events
            .iter()
            .filter_map(|event| match event {
                DeletionEvent::Removed { target: 0, path } => Some(path),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(&removed[2..], [&nested, &root]);
        assert!(!root.exists());
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn removes_symlink_without_following_it() {
        for delete_parent in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("target");
            fs::create_dir(&target).unwrap();
            fs::write(target.join("keep.txt"), b"keep").unwrap();

            let root = dir.path().join("root");
            fs::create_dir(&root).unwrap();
            let link = root.join("link");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &link).unwrap();
            #[cfg(windows)]
            match std::os::windows::fs::symlink_dir(&target, &link) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::PermissionDenied => return,
                Err(err) => panic!("directory symlink can be created: {err}"),
            }

            let mut task = DeletionTask::start(
                vec![if delete_parent { root } else { link.clone() }],
                2,
                MarkMode::Delete,
            )
            .unwrap();
            let events = finish(&mut task);

            assert_finished(&events, if delete_parent { 2 } else { 1 }, 0, false);
            assert!(!link.exists(), "the symlink itself should be gone");
            assert!(
                target.join("keep.txt").exists(),
                "the symlink target must not be deleted"
            );
        }
    }

    #[test]
    fn reports_an_error_for_a_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let mut task =
            DeletionTask::start(vec![dir.path().join("does-not-exist")], 1, MarkMode::Delete)
                .unwrap();

        assert_finished(&finish(&mut task), 0, 1, false);
    }

    #[test]
    fn disappearance_after_enumeration_updates_the_tree_without_counting_a_removal() {
        let (send, receive) = bounded(1);
        let cancel = AtomicBool::new(false);
        let mut stats = RemovalStats::default();
        let path = Path::new("vanished");
        record_removal(
            Err(io::Error::from(io::ErrorKind::NotFound)),
            path,
            4,
            &send,
            &cancel,
            &mut stats,
        );
        assert!(matches!(
            receive.recv().unwrap(),
            DeletionEvent::Removed { target: 4, path: removed } if removed == path
        ));
        assert_eq!((stats.entries, stats.errors), (0, 0));
    }

    fn task_with_capacity(paths: Vec<PathBuf>, capacity: usize) -> DeletionTask {
        let started = Instant::now();
        let (send, events) = bounded(capacity);
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let handle = thread::spawn(move || {
            delete_paths(paths, 1, MarkMode::Delete, &send, &worker_cancel);
        });
        DeletionTask {
            events,
            cancel,
            started,
            mode: MarkMode::Delete,
            handle: Some(handle),
        }
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !condition() {
            assert!(
                Instant::now() < deadline,
                "worker did not reach the test gate"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn cancellation_finishes_the_current_removal_and_preserves_remaining_targets() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("first"), b"first").unwrap();
        fs::write(root.join("second"), b"second").unwrap();
        let next_root = dir.path().join("keep");
        fs::write(&next_root, b"keep").unwrap();
        let mut task = task_with_capacity(vec![root.clone(), next_root.clone()], 0);

        // The first successful unlink is blocked publishing its event. No second unlink
        // can start until we consume it, making cancellation independent of scheduling.
        wait_until(|| fs::read_dir(&root).unwrap().count() == 1);
        task.cancel.store(true, Ordering::Relaxed);
        let events = finish(&mut task);

        assert_finished(&events, 1, 0, true);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert!(next_root.exists());
        assert!(matches!(
            events.get(1),
            Some(DeletionEvent::TargetFinished {
                target: 0,
                errors: 0
            })
        ));
    }

    #[test]
    fn dropping_a_task_disconnects_a_full_channel_before_joining() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        for name in ["first", "second", "third"] {
            fs::write(root.join(name), name).unwrap();
        }
        let task = task_with_capacity(vec![root.clone()], 1);
        wait_until(|| fs::read_dir(&root).unwrap().count() == 1);
        assert_eq!(task.events.len(), 1);

        let (finished, receive) = bounded(1);
        thread::spawn(move || {
            drop(task);
            finished.send(()).unwrap();
        });
        receive
            .recv_timeout(Duration::from_secs(5))
            .expect("dropping a task must unblock publishers and join the worker");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }
}
