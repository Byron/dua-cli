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
        dua_core::Options::default(),
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
        dua_core::Options::default(),
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
