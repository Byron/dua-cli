use std::{collections::BTreeSet, fs, path::PathBuf};

use dua_core::{Order, RootEvent, walk, walk_roots};

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
