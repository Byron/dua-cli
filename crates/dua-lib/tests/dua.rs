use std::{collections::BTreeSet, fs, path::PathBuf};

use dua_core::{MultiWalk, Order, RootEvent, walk};

fn collect_events(mut walk: MultiWalk) -> Vec<(usize, RootEvent)> {
    std::iter::from_fn(|| walk.recv(None).unwrap()).collect()
}

#[test]
fn traversal_errors_retain_the_failing_path() {
    let dir = tempfile::tempdir().unwrap();
    for order in [Order::Completion, Order::ParentFirst] {
        for options in [
            dua_core::Options::default(),
            dua_core::Options::default().skip_metadata(),
        ] {
            let missing = dir.path().join("missing");
            let error = walk(&missing, 2, order, options, |_| true)
                .next()
                .unwrap()
                .err()
                .unwrap();
            assert_eq!(error.path(), Some(missing.as_path()));
            assert_eq!(error.io_error().kind(), std::io::ErrorKind::NotFound);
            assert!(error.to_string().contains(&missing.display().to_string()));

            let children = [dir.path().join("child-a"), dir.path().join("child-b")];
            for child in &children {
                fs::create_dir(child).unwrap();
            }
            let removed_children = children.clone();
            let events = collect_events(MultiWalk::prepare(2, order, options).start_paths(
                [(7, dir.path().to_owned())],
                move |_, entry| {
                    if removed_children.contains(&entry.path()) {
                        fs::remove_dir(entry.path()).unwrap();
                    }
                    true
                },
            ));
            let mut errors = Vec::new();
            for (index, event) in &events {
                assert_eq!(*index, 7);
                if let RootEvent::Entries(entries) = event {
                    errors.extend(entries.iter().filter_map(|entry| entry.as_ref().err()));
                }
            }
            assert_eq!(errors.len(), 2);
            let paths = errors
                .iter()
                .map(|error| {
                    assert_eq!(error.io_error().kind(), std::io::ErrorKind::NotFound);
                    error.path().unwrap().to_owned()
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(paths, children.into());
            assert!(matches!(events.last(), Some((7, RootEvent::Finished))));
        }
    }
}

#[test]
fn streamed_roots_and_io_conversions_retain_error_context() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    let (mut sender, mut walk) =
        MultiWalk::prepare(2, Order::Completion, dua_core::Options::default()).stream_roots();
    sender.add_root(7, missing.clone(), |_| true).unwrap();
    drop(sender);
    let Some((7, RootEvent::Entries(entries))) = walk.recv(None).unwrap() else {
        panic!("the missing root must emit a batch");
    };
    let Ok([Err(error)]) = <[_; 1]>::try_from(entries) else {
        panic!("the missing root must fail");
    };
    assert_eq!(error.path(), Some(missing.as_path()));
    let code = error.io_error().raw_os_error().unwrap();
    assert_eq!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(code)
    );
    let converted = std::io::Error::from(error);
    assert_eq!(converted.kind(), std::io::ErrorKind::NotFound);
    assert_eq!(
        converted
            .get_ref()
            .unwrap()
            .downcast_ref::<dua_core::Error>()
            .unwrap()
            .path(),
        Some(missing.as_path())
    );
    assert!(matches!(
        walk.recv(None).unwrap(),
        Some((7, RootEvent::Finished))
    ));
    assert!(walk.recv(None).unwrap().is_none());

    let supplied = dua_core::Error::from(std::io::Error::other("supplied error"));
    assert_eq!(supplied.path(), None);
    assert_eq!(supplied.to_string(), "supplied error");
}

#[test]
fn streamed_roots_visit_batches_before_completion() {
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("child"), b"data").unwrap();
    let depths = Arc::new(Mutex::new(Vec::new()));
    let visited = Arc::clone(&depths);
    let caller = thread::current().id();
    let (mut sender, mut walk) =
        MultiWalk::prepare(2, Order::ParentFirst, dua_core::Options::default())
            .visit_batches(move |worker, root, entries| {
                assert!(worker < 2);
                assert_eq!(root, 7);
                assert_ne!(thread::current().id(), caller);
                visited
                    .lock()
                    .unwrap()
                    .extend(entries.into_iter().map(|entry| entry.unwrap().depth));
            })
            .stream_roots();
    sender.add_root(7, dir.path().to_owned(), |_| true).unwrap();
    drop(sender);

    let timeout = Some(Duration::from_secs(5));
    assert!(matches!(
        walk.recv(timeout).unwrap(),
        Some((7, RootEvent::Finished))
    ));
    assert_eq!(*depths.lock().unwrap(), [0, 1]);
    assert!(walk.recv(timeout).unwrap().is_none());
}

#[test]
fn walkers_are_available_to_consumers() {
    let dir = tempfile::tempdir().unwrap();
    let roots = [dir.path().join("a"), dir.path().join("b")];
    for root in &roots {
        fs::create_dir_all(root.join("child")).unwrap();
    }

    let paths = walk(
        &roots[0],
        2,
        Order::ParentFirst,
        dua_core::Options::default().queued_batches_per_worker(128),
        |_| true,
    )
    .map(|entry| {
        entry
            .unwrap()
            .path()
            .strip_prefix(&roots[0])
            .unwrap()
            .into()
    })
    .collect::<BTreeSet<PathBuf>>();
    assert_eq!(paths, [PathBuf::new(), PathBuf::from("child")].into());

    let events = collect_events(
        MultiWalk::prepare(
            2,
            Order::Completion,
            dua_core::Options::default().queued_batches_per_worker(0),
        )
        .start_paths(roots.iter().cloned().enumerate(), |_, _| true),
    );
    for root_idx in 0..roots.len() {
        let last_entry = events
            .iter()
            .rposition(|(idx, event)| *idx == root_idx && matches!(event, RootEvent::Entries(_)))
            .unwrap();
        let finished = events
            .iter()
            .position(|(idx, event)| *idx == root_idx && matches!(event, RootEvent::Finished))
            .unwrap();
        assert!(last_entry < finished);
    }
}

#[test]
fn sparse_root_indices_do_not_size_internal_storage() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let events = collect_events(
        MultiWalk::prepare(1, Order::Completion, dua_core::Options::default())
            .start_paths([(usize::MAX, file.path().to_owned())], |_, _| true),
    );
    assert!(events.iter().all(|(root_idx, _)| *root_idx == usize::MAX));
    assert!(matches!(events.last(), Some((_, RootEvent::Finished))));
}

#[test]
#[should_panic(expected = "root indices must be unique")]
fn duplicate_root_indices_are_rejected() {
    let file = tempfile::NamedTempFile::new().unwrap();
    MultiWalk::prepare(1, Order::Completion, dua_core::Options::default()).start_paths(
        [(7, file.path().to_owned()), (7, file.path().to_owned())],
        |_, _| true,
    );
}

#[test]
fn type_only_walks_keep_types_without_collecting_metadata() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("nested/child")).unwrap();
    fs::write(dir.path().join("nested/file"), b"content").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("nested", dir.path().join("link")).unwrap();
        std::os::unix::fs::symlink("missing", dir.path().join("broken")).unwrap();
    }
    let options = dua_core::Options::default().skip_metadata();
    for order in [Order::ParentFirst, Order::Completion] {
        let expected = walk(dir.path(), 2, order, dua_core::Options::default(), |_| true)
            .map(|entry| {
                let entry = entry.unwrap();
                (
                    entry.path(),
                    (
                        entry.depth,
                        entry.file_type.is_dir(),
                        entry.file_type.is_symlink(),
                    ),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut walker = walk(dir.path(), 2, order, options, |_| true);
        for pass in 0..2 {
            if pass != 0 {
                assert!(walker.restart());
            }
            let entries = walker.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
            let actual = entries
                .into_iter()
                .map(|entry| {
                    assert!(
                        entry.metadata.is_none(),
                        "metadata was not requested: {:?}",
                        entry.path()
                    );
                    (
                        entry.path(),
                        (
                            entry.depth,
                            entry.file_type.is_dir(),
                            entry.file_type.is_symlink(),
                        ),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            assert_eq!(actual, expected);
        }

        let mut paths = BTreeSet::new();
        let mut walk = MultiWalk::prepare(2, order, options)
            .start_paths([(7, dir.path().to_owned())], |_, _| true);
        while let Some((root, batch)) = walk.recv(None).unwrap() {
            assert_eq!(root, 7);
            match batch {
                RootEvent::Entries(entries) => {
                    for entry in entries {
                        let entry = entry.unwrap();
                        assert!(entry.metadata.is_none());
                        paths.insert(entry.path());
                    }
                }
                RootEvent::Finished => assert_eq!(paths, expected.keys().cloned().collect()),
                RootEvent::WorkerFailed(error) => panic!("worker failed: {error}"),
            }
        }
    }
}
