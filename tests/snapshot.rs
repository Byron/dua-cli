use dua::snapshot::{read, write};
use dua::traverse::{EntryData, Traversal, TreeIndex};
use petgraph::Direction;
use std::{
    ffi::OsString,
    io::Cursor,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

#[cfg(unix)]
const ONE_FILE: &[u8] = &[
    0x44, 0x55, 0x41, 0x53, 0x4e, 0x41, 0x50, 0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x01, 0x00, 0x01,
    0x61, 0x01, 0x00, 0x00, 0x00, 0x01, 0x69, 0x58, 0x7d, 0x2b, 0xa3, 0xd1, 0xef, 0x5b, 0x1d, 0x3d,
    0x5b, 0xef, 0xc7, 0x03, 0x6e, 0x80, 0xfb, 0x3a, 0xf4, 0x1a, 0xea, 0xa0, 0xd0, 0x8f, 0x12, 0xcf,
    0x15, 0x79, 0xc3, 0xb1, 0xa4, 0x7b,
];

fn add(
    traversal: &mut Traversal,
    parent: TreeIndex,
    name: impl Into<PathBuf>,
    data: EntryData,
) -> TreeIndex {
    let node = traversal.tree.add_node(EntryData {
        name: name.into(),
        ..data
    });
    traversal.tree.add_edge(parent, node, ());
    node
}

fn encoded(traversal: &Traversal, roots: &[TreeIndex]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write(&mut bytes, traversal, roots, None).unwrap();
    bytes
}

fn encoded_result(traversal: &Traversal, roots: &[TreeIndex]) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    write(&mut bytes, traversal, roots, None)?;
    Ok(bytes)
}

#[cfg(unix)]
#[test]
fn one_file_matches_the_golden_stream() {
    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(
        &mut traversal,
        root_index,
        "a",
        EntryData {
            size: 1,
            ..EntryData::default()
        },
    );

    let bytes = encoded(&traversal, &[root]);
    assert_eq!(bytes, ONE_FILE);

    let snapshot = read(Cursor::new(bytes)).unwrap();
    assert_eq!(snapshot.roots.len(), 1);
    assert_eq!(
        snapshot.traversal.tree[snapshot.roots[0]].name,
        Path::new("a")
    );
    assert_eq!(snapshot.traversal.tree[snapshot.roots[0]].size, 1);
    assert_eq!(
        snapshot.traversal.tree[snapshot.traversal.root_index].entry_count,
        Some(1)
    );
    assert_eq!(snapshot.traversal.cost, Some(Duration::ZERO));
    assert_eq!(encoded(&snapshot.traversal, &snapshot.roots), ONE_FILE);
}

#[test]
fn empty_snapshot_round_trips() {
    let traversal = Traversal::new();
    let bytes = encoded(&traversal, &[]);
    let snapshot = read(Cursor::new(&bytes)).unwrap();
    assert!(snapshot.roots.is_empty());
    assert_eq!(snapshot.traversal.tree.node_count(), 1);
    assert_eq!(
        snapshot.traversal.tree[snapshot.traversal.root_index].size,
        0
    );
    assert_eq!(
        snapshot.traversal.tree[snapshot.traversal.root_index].entry_count,
        None
    );
}

#[test]
fn all_entry_fields_and_multiple_roots_round_trip() {
    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let first = add(
        &mut traversal,
        root_index,
        "first/root",
        EntryData {
            size: 42,
            mtime: UNIX_EPOCH - Duration::from_nanos(1),
            entry_count: Some(2),
            metadata_io_error: true,
            is_dir: true,
            ..EntryData::default()
        },
    );
    add(
        &mut traversal,
        first,
        "child",
        EntryData {
            size: u128::MAX,
            mtime: UNIX_EPOCH + Duration::new(7, 8),
            entry_count: Some(0),
            ..EntryData::default()
        },
    );
    let second = add(
        &mut traversal,
        root_index,
        "empty",
        EntryData {
            entry_count: Some(0),
            is_dir: true,
            ..EntryData::default()
        },
    );

    let bytes = encoded(&traversal, &[first, second]);
    let snapshot = read(Cursor::new(&bytes)).unwrap();
    let first = snapshot.roots[0];
    let second = snapshot.roots[1];
    let child = snapshot
        .traversal
        .tree
        .neighbors_directed(first, Direction::Outgoing)
        .next()
        .unwrap();

    assert_eq!(snapshot.traversal.tree[first].name, Path::new("first/root"));
    assert_eq!(
        snapshot.traversal.tree[first].mtime,
        UNIX_EPOCH - Duration::from_nanos(1)
    );
    assert!(snapshot.traversal.tree[first].metadata_io_error);
    assert!(snapshot.traversal.tree[first].is_dir);
    assert_eq!(snapshot.traversal.tree[child].size, u128::MAX);
    assert_eq!(snapshot.traversal.tree[child].entry_count, Some(0));
    assert_eq!(
        snapshot.traversal.tree[child].mtime,
        UNIX_EPOCH + Duration::new(7, 8)
    );
    assert_eq!(snapshot.traversal.tree[second].entry_count, Some(0));
    assert_eq!(
        snapshot.traversal.tree[snapshot.traversal.root_index].size,
        42
    );
    assert_eq!(
        snapshot.traversal.tree[snapshot.traversal.root_index].entry_count,
        Some(2)
    );
    assert_eq!(encoded(&snapshot.traversal, &snapshot.roots), bytes);
}

#[test]
fn child_order_is_deterministic() {
    fn with_children(names: &[&str]) -> (Traversal, TreeIndex) {
        let mut traversal = Traversal::new();
        let root_index = traversal.root_index;
        let root = add(
            &mut traversal,
            root_index,
            "root",
            EntryData {
                is_dir: true,
                ..EntryData::default()
            },
        );
        for name in names {
            add(&mut traversal, root, *name, EntryData::default());
        }
        (traversal, root)
    }

    let (left, left_root) = with_children(&["z", "a"]);
    let (right, right_root) = with_children(&["a", "z"]);
    assert_eq!(encoded(&left, &[left_root]), encoded(&right, &[right_root]));

    let mut duplicates = Traversal::new();
    let synthetic_root = duplicates.root_index;
    let root = add(
        &mut duplicates,
        synthetic_root,
        "root",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    for size in [2, 1] {
        add(
            &mut duplicates,
            root,
            "same",
            EntryData {
                size,
                ..EntryData::default()
            },
        );
    }
    let bytes = encoded(&duplicates, &[root]);
    let snapshot = read(Cursor::new(&bytes)).unwrap();
    assert_eq!(encoded(&snapshot.traversal, &snapshot.roots), bytes);
}

#[cfg(unix)]
#[test]
fn invalid_utf8_name_round_trips() {
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};

    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(
        &mut traversal,
        root_index,
        PathBuf::from(OsString::from_vec(vec![0xff])),
        EntryData::default(),
    );
    let snapshot = read(Cursor::new(encoded(&traversal, &[root]))).unwrap();
    assert_eq!(
        snapshot.traversal.tree[snapshot.roots[0]]
            .name
            .as_os_str()
            .as_bytes(),
        &[0xff]
    );
}

#[cfg(windows)]
#[test]
fn unpaired_utf16_surrogate_round_trips() {
    use std::os::windows::ffi::{OsStrExt as _, OsStringExt as _};

    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(
        &mut traversal,
        root_index,
        PathBuf::from(OsString::from_wide(&[0xd800])),
        EntryData::default(),
    );
    let snapshot = read(Cursor::new(encoded(&traversal, &[root]))).unwrap();
    assert_eq!(
        snapshot.traversal.tree[snapshot.roots[0]]
            .name
            .as_os_str()
            .encode_wide()
            .collect::<Vec<_>>(),
        [0xd800]
    );
}

#[test]
fn rejects_corruption_bad_digest_truncation_and_trailing_data() {
    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(&mut traversal, root_index, "a", EntryData::default());
    let bytes = encoded(&traversal, &[root]);

    let mut corrupt = bytes.clone();
    corrupt[16] = b'b';
    assert!(
        read(Cursor::new(corrupt))
            .unwrap_err()
            .to_string()
            .contains("checksum")
    );

    let mut bad_digest = bytes.clone();
    *bad_digest.last_mut().unwrap() ^= 1;
    assert!(
        read(Cursor::new(bad_digest))
            .unwrap_err()
            .to_string()
            .contains("checksum")
    );

    for end in [0, 11, 21, bytes.len() - 1] {
        assert!(
            read(Cursor::new(&bytes[..end])).is_err(),
            "accepted truncation at {end}"
        );
    }

    let mut trailing = bytes;
    trailing.push(0);
    assert!(
        read(Cursor::new(trailing))
            .unwrap_err()
            .to_string()
            .contains("trailing")
    );
}

#[test]
fn rejects_invalid_child_components_and_file_parents() {
    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(&mut traversal, root_index, "root", EntryData::default());
    add(&mut traversal, root, "child", EntryData::default());
    assert!(
        encoded_result(&traversal, &[root])
            .unwrap_err()
            .to_string()
            .contains("children")
    );

    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(
        &mut traversal,
        root_index,
        "root",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    add(&mut traversal, root, ".", EntryData::default());
    assert!(
        encoded_result(&traversal, &[root])
            .unwrap_err()
            .to_string()
            .contains("component")
    );
}

#[test]
fn writer_rejects_non_tree_inputs_without_a_global_visited_set() {
    let mut traversal = Traversal::new();
    let synthetic = traversal.root_index;
    let root = add(
        &mut traversal,
        synthetic,
        "root",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    assert!(
        encoded_result(&traversal, &[root, root])
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );

    let left = add(
        &mut traversal,
        root,
        "left",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    let right = add(
        &mut traversal,
        root,
        "right",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    let shared = add(&mut traversal, left, "shared", EntryData::default());
    traversal.tree.add_edge(right, shared, ());
    assert!(
        encoded_result(&traversal, &[root])
            .unwrap_err()
            .to_string()
            .contains("cycle or shared node")
    );

    let mut traversal = Traversal::new();
    let synthetic = traversal.root_index;
    let root = add(
        &mut traversal,
        synthetic,
        "root",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    let child = add(&mut traversal, root, "child", EntryData::default());
    traversal.tree.add_edge(root, child, ());
    assert!(
        encoded_result(&traversal, &[root])
            .unwrap_err()
            .to_string()
            .contains("cycle or shared node")
    );

    let mut traversal = Traversal::new();
    let synthetic = traversal.root_index;
    let root = add(
        &mut traversal,
        synthetic,
        "root",
        EntryData {
            is_dir: true,
            ..EntryData::default()
        },
    );
    traversal.tree[synthetic].name = "synthetic".into();
    traversal.tree.add_edge(root, synthetic, ());
    assert!(
        encoded_result(&traversal, &[root])
            .unwrap_err()
            .to_string()
            .contains("cycle or shared node")
    );
}

#[test]
fn rejects_synthetic_root_overflow() {
    for (first, second) in [
        (
            EntryData {
                size: u128::MAX,
                ..EntryData::default()
            },
            EntryData {
                size: 1,
                ..EntryData::default()
            },
        ),
        (
            EntryData {
                entry_count: Some(u64::MAX),
                ..EntryData::default()
            },
            EntryData {
                entry_count: Some(1),
                ..EntryData::default()
            },
        ),
    ] {
        let mut traversal = Traversal::new();
        let root_index = traversal.root_index;
        let first = add(&mut traversal, root_index, "first", first);
        let second = add(&mut traversal, root_index, "second", second);
        assert!(
            read(Cursor::new(encoded(&traversal, &[first, second])))
                .unwrap_err()
                .to_string()
                .contains("overflows")
        );
    }
}
