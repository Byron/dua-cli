use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

/// State of a controlled walk, including whether its current work has stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Status {
    /// Work can proceed.
    Running,
    /// A pause was requested; in-progress work is still finishing.
    Pausing,
    /// No work is active and new work cannot start until resumed.
    Paused,
    /// The iterator finished, or its control was cancelled.
    Finished,
}

#[derive(Default)]
struct State {
    paused: bool,
    cancelled: bool,
    active: usize,
    excluded: HashSet<PathBuf>,
}

/// Share pause, resume and removed-path exclusion across related filesystem walks.
///
/// Pausing is asynchronous: keep consuming output while [`Self::status`] is `Pausing`,
/// since an active worker may be blocked publishing a batch. Once `Paused`, no new
/// filesystem job can start. Buffered output is retained and can still be consumed.
#[derive(Clone, Default)]
pub struct Control {
    state: Arc<(Mutex<State>, Condvar)>,
}

#[allow(
    clippy::missing_panics_doc,
    reason = "only poisoning of private synchronization state can panic"
)]
impl Control {
    /// Stop starting new jobs, allowing in-progress jobs to finish.
    pub fn pause(&self) {
        self.state.0.lock().unwrap().paused = true;
    }

    /// Allow paused jobs to continue.
    pub fn resume(&self) {
        self.state.0.lock().unwrap().paused = false;
        self.state.1.notify_all();
    }

    /// Cancel this control and release threads waiting at a pause checkpoint.
    pub fn cancel(&self) {
        self.state.0.lock().unwrap().cancelled = true;
        self.state.1.notify_all();
    }

    /// Report whether work is running, finishing a pause, paused, or cancelled.
    #[must_use]
    pub fn status(&self) -> Status {
        let state = self.state.0.lock().unwrap();
        if state.cancelled {
            Status::Finished
        } else if !state.paused {
            Status::Running
        } else if state.active == 0 {
            Status::Paused
        } else {
            Status::Pausing
        }
    }

    /// Ignore jobs and buffered entries at or below a path removed while paused.
    pub fn exclude_path(&self, path: PathBuf) {
        self.state.0.lock().unwrap().excluded.insert(path);
    }

    /// Whether a path is below one of the excluded paths.
    #[must_use]
    pub fn is_excluded(&self, path: &Path) -> bool {
        let state = self.state.0.lock().unwrap();
        !state.excluded.is_empty() && path.ancestors().any(|path| state.excluded.contains(path))
    }

    /// Whether any paths have been excluded from this walk.
    #[must_use]
    pub fn has_exclusions(&self) -> bool {
        !self.state.0.lock().unwrap().excluded.is_empty()
    }

    /// Start a related unit of work, waiting if paused and returning `None` if cancelled.
    /// Hold the returned guard until the work and its output have finished.
    #[must_use]
    pub fn activity(&self) -> Option<Activity<'_>> {
        self.enter(None)
    }

    pub(crate) fn enter(&self, stopped: Option<&AtomicBool>) -> Option<Activity<'_>> {
        let mut state = self.state.0.lock().unwrap();
        loop {
            if state.cancelled || stopped.is_some_and(|stop| stop.load(Ordering::Relaxed)) {
                return None;
            }
            if !state.paused {
                state.active += 1;
                return Some(Activity(self));
            }
            state = self.state.1.wait(state).unwrap();
        }
    }

    pub(crate) fn wake(&self) {
        let _state = self.state.0.lock().unwrap();
        self.state.1.notify_all();
    }
}

/// Keeps a unit of work visible to [`Control::status`] until it is dropped.
pub struct Activity<'a>(&'a Control);

impl Drop for Activity<'_> {
    fn drop(&mut self) {
        self.0.state.0.lock().unwrap().active -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MultiWalk, Options, Order, RootEvent};
    use crossbeam::channel::bounded;
    use std::{
        fs, thread,
        time::{Duration, Instant},
    };

    fn wait_for_pause(control: &Control) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while control.status() != Status::Paused {
            assert!(Instant::now() < deadline, "walk did not finish pausing");
            thread::yield_now();
        }
    }

    #[test]
    fn pause_drains_active_jobs_and_resume_skips_removed_subtrees() {
        for threads in [1, 4] {
            let dir = tempfile::tempdir().unwrap();
            fs::create_dir_all(dir.path().join("victim/nested")).unwrap();
            fs::write(dir.path().join("victim/nested/file"), b"removed").unwrap();
            fs::create_dir(dir.path().join("keep")).unwrap();
            fs::write(dir.path().join("keep/file"), b"keep").unwrap();
            let (started_tx, started_rx) = bounded(1);
            let (release_tx, release_rx) = bounded(1);
            let control = Control::default();
            let mut walk = MultiWalk::prepare(
                threads,
                Order::ParentFirst,
                Options::default().queued_batches_per_worker(0),
            )
            .with_control(control.clone())
            .start_paths([(0, dir.path().to_owned())], move |_, entry| {
                if entry.file_name == "victim" {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                }
                true
            });
            let reader = thread::spawn(move || {
                let mut paths = Vec::new();
                while let Some((_, event)) = walk.recv(None).unwrap() {
                    match event {
                        RootEvent::Entries(entries) => {
                            paths.extend(entries.into_iter().map(|entry| entry.unwrap().path()));
                        }
                        RootEvent::Finished => {}
                        RootEvent::WorkerFailed(error) => panic!("{error}"),
                    }
                }
                paths
            });
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            control.pause();
            assert_eq!(control.status(), Status::Pausing);
            release_tx.send(()).unwrap();
            wait_for_pause(&control);
            let victim = dir.path().join("victim");
            fs::remove_dir_all(&victim).unwrap();
            control.exclude_path(victim.clone());
            control.resume();
            let paths = reader.join().unwrap();
            assert!(paths.contains(&dir.path().join("keep/file")));
            assert!(!paths.contains(&victim.join("nested/file")));
        }
    }

    #[test]
    fn dropping_a_paused_walk_releases_waiting_workers() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("child")).unwrap();
        let control = Control::default();
        let (started_tx, started_rx) = bounded(1);
        let (release_tx, release_rx) = bounded(1);
        let walk = MultiWalk::prepare(2, Order::ParentFirst, Options::default())
            .with_control(control.clone())
            .start_paths([(0, dir.path().to_owned())], move |_, entry| {
                if entry.file_name == "child" {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                }
                true
            });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        control.pause();
        release_tx.send(()).unwrap();
        wait_for_pause(&control);
        let (done_tx, done_rx) = bounded(1);
        let dropper = thread::spawn(move || {
            drop(walk);
            done_tx.send(()).unwrap();
        });
        done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        dropper.join().unwrap();
    }

    #[test]
    fn cancellation_wakes_a_reader_waiting_on_paused_workers() {
        let control = Control::default();
        let (roots, mut walk) = MultiWalk::prepare(2, Order::ParentFirst, Options::default())
            .with_control(control.clone())
            .stream_roots();
        control.pause();
        assert_eq!(walk.status(), Status::Paused);
        let (done_tx, done_rx) = bounded(1);
        let reader = thread::spawn(move || {
            assert!(walk.recv(None).unwrap().is_none());
            done_tx.send(()).unwrap();
        });
        control.cancel();
        done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        reader.join().unwrap();
        drop(roots);
    }

    #[test]
    fn excluded_buffered_roots_are_filtered_without_a_worker_pool() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file");
        fs::write(&path, b"file").unwrap();
        let control = Control::default();
        let mut walk = MultiWalk::prepare(1, Order::ParentFirst, Options::default())
            .with_control(control.clone())
            .start_paths([(0, path.clone())], |_, _| true);
        control.pause();
        control.exclude_path(path);
        assert!(walk.control().is_some());
        assert!(matches!(
            walk.recv(None).unwrap(),
            Some((0, RootEvent::Finished))
        ));
        assert!(walk.recv(None).unwrap().is_none());
        assert_eq!(walk.status(), Status::Finished);
    }
}
