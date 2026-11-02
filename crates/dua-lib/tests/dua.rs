use std::{collections::BTreeSet, fs, path::PathBuf};

use dua_core::{Order, RootEvent, walk, walk_roots};

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
            let events = walk_roots(
                [(7, dir.path().to_owned())],
                2,
                order,
                options,
                move |_, entry| {
                    if removed_children.contains(&entry.path()) {
                        fs::remove_dir(entry.path()).unwrap();
                    }
                    true
                },
            )
            .collect::<Vec<_>>();
            let errors = events
                .iter()
                .filter_map(|(index, event)| match event {
                    RootEvent::Entry(Err(error)) => {
                        assert_eq!(*index, 7);
                        Some(error)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
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
    let (mut sender, walk) =
        dua_core::stream_roots(2, Order::Completion, dua_core::Options::default());
    sender.add_root(7, missing.clone(), |_| true).unwrap();
    drop(sender);
    let mut events = walk.collect::<Vec<_>>().into_iter();
    let Some((7, RootEvent::Entry(Err(error)))) = events.next() else {
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
    assert!(matches!(events.next(), Some((7, RootEvent::Finished))));
    assert!(events.next().is_none());

    let supplied = dua_core::Error::from(std::io::Error::other("supplied error"));
    assert_eq!(supplied.path(), None);
    assert_eq!(supplied.to_string(), "supplied error");
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

    let events = walk_roots(
        roots.iter().cloned().enumerate(),
        2,
        Order::Completion,
        dua_core::Options::default().queued_batches_per_worker(0),
        |_, _| true,
    )
    .collect::<Vec<_>>();
    for root_idx in 0..roots.len() {
        let last_entry = events
            .iter()
            .rposition(|(idx, event)| *idx == root_idx && matches!(event, RootEvent::Entry(_)))
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
    let events = walk_roots(
        [(usize::MAX, file.path().to_owned())],
        1,
        Order::Completion,
        dua_core::Options::default(),
        |_, _| true,
    )
    .collect::<Vec<_>>();
    assert!(events.iter().all(|(root_idx, _)| *root_idx == usize::MAX));
    assert!(matches!(events.last(), Some((_, RootEvent::Finished))));
}

#[test]
#[should_panic(expected = "root indices must be unique")]
fn duplicate_root_indices_are_rejected() {
    let file = tempfile::NamedTempFile::new().unwrap();
    walk_roots(
        [(7, file.path().to_owned()), (7, file.path().to_owned())],
        1,
        Order::Completion,
        dua_core::Options::default(),
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
        let events = walk_roots([(7, dir.path().to_owned())], 2, order, options, |_, _| true);
        for (root, event) in events {
            assert_eq!(root, 7);
            match event {
                RootEvent::Entry(entry) => {
                    let entry = entry.unwrap();
                    assert!(entry.metadata.is_none());
                    paths.insert(entry.path());
                }
                RootEvent::Finished => assert_eq!(paths, expected.keys().cloned().collect()),
            }
        }
    }
}
