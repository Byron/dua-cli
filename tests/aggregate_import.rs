use bstr::ByteSlice;
use dua::ByteFormat;
use dua::snapshot::Replay;
use dua::traverse::{EntryData, Traversal};
use std::io::Cursor;

#[test]
fn invalid_snapshot_is_rejected_before_output() {
    assert!(Replay::new(Cursor::new(b"not a snapshot")).is_err());
}

#[test]
fn aggregate_import_renders_without_accessing_stored_paths() {
    let dir = tempfile::tempdir().unwrap();
    let missing_root = dir.path().join("snapshot-root-that-does-not-exist");
    let mut traversal = Traversal::new();
    let root = traversal.tree.add_child(
        traversal.root_index,
        &missing_root,
        EntryData {
            size: 5,
            entry_count: Some(1),
            is_dir: true,
            ..EntryData::default()
        },
    );
    traversal.tree.add_child(
        root,
        "child",
        EntryData {
            size: 2,
            ..EntryData::default()
        },
    );

    let mut snapshot = Vec::new();
    dua::snapshot::write(&mut snapshot, &traversal, &[root], Some(2)).unwrap();

    let mut replay = Replay::new(Cursor::new(snapshot)).unwrap();
    let mut flat = Vec::new();
    dua::aggregate_replay(
        (&mut flat, false),
        &mut replay,
        true,
        true,
        ByteFormat::Bytes,
    )
    .unwrap();
    let flat_stdout = String::from_utf8(flat).unwrap();
    assert!(flat_stdout.contains(&format!("5 b {}", missing_root.display())));
    assert!(!flat_stdout.contains("child"));

    let mut tree = Vec::new();
    dua::aggregate_tree_from_replay(
        (&mut tree, false),
        &mut replay,
        ByteFormat::Bytes,
        1,
        true,
        true,
    )
    .unwrap();
    assert!(String::from_utf8(tree).unwrap().contains("2 b   child"));

    let mut stacks = Vec::new();
    dua::stacks_from_replay(&mut stacks, &mut replay, None).unwrap();
    assert!(String::from_utf8(stacks).unwrap().contains(";child 2"));

    assert!(!missing_root.exists());
}

#[test]
fn aggregate_import_displays_the_maximum_snapshot_size() {
    let mut traversal = Traversal::new();
    let root = traversal.tree.add_child(
        traversal.root_index,
        "largest",
        EntryData {
            size: u128::MAX,
            ..EntryData::default()
        },
    );

    let mut snapshot = Vec::new();
    dua::snapshot::write(&mut snapshot, &traversal, &[root], Some(2)).unwrap();

    let mut replay = Replay::new(Cursor::new(snapshot)).unwrap();
    let mut output = Vec::new();
    dua::aggregate_replay(
        (&mut output, false),
        &mut replay,
        true,
        true,
        ByteFormat::Bytes,
    )
    .unwrap();
    insta::assert_snapshot!(
        output.as_bstr(),
        "maximum snapshot size in raw bytes",
        @"340282366920938463463374607431768211455 b largest"
    );
}
