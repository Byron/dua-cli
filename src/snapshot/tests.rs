use super::*;
use std::{io::Cursor, path::PathBuf};

#[test]
fn replay_rejects_source_changes_after_validation() {
    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(&mut traversal, root_index, "a", EntryData::default());
    let bytes = encoded(&traversal, &[root]);
    let mut replay = Replay::new(Cursor::new(bytes.clone())).unwrap();

    let mut changed = bytes[..bytes.len() - DIGEST_LEN].to_vec();
    changed[16] = b'b';
    *replay.reader.get_mut() = rehash(changed);

    assert!(
        replay
            .for_each_entry(|_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("changed since it was verified")
    );
}

#[test]
fn replay_defers_record_validation_until_replay() {
    let bytes = stream(&[record(0, 0, b"a", 0, 0, 0, None)], 1);
    let mut replay = Replay::new(Cursor::new(bytes)).unwrap();

    assert!(
        replay
            .for_each_entry(|_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("parent distance")
    );
}

#[test]
fn decoder_lends_record_names_and_reuses_depth_buffers() {
    let root_name = native_name_bytes(Path::new("root-with-a-long-name"));
    let child_name = native_name_bytes(Path::new("long-child-name"));
    let leaf_name = native_name_bytes(Path::new("x"));
    let bytes = stream(
        &[
            record(1, FLAG_DIRECTORY, &root_name, 0, 0, 0, None),
            native_record(1, FLAG_DIRECTORY, "a"),
            record(1, 0, &child_name, 0, 0, 0, None),
            native_record(3, FLAG_DIRECTORY, "b"),
            record(1, 0, &leaf_name, 0, 0, 0, None),
        ],
        5,
    );
    let mut decoder = Decoder::new(Cursor::new(bytes)).unwrap();

    let borrowed_name = {
        let entry = decoder.next_entry(None).unwrap().unwrap();
        assert_eq!(entry.native_name, root_name);
        entry.native_name.as_ptr()
    };
    let name_offset = decoder
        .record
        .windows(root_name.len())
        .position(|bytes| bytes == root_name)
        .unwrap();
    assert_eq!(borrowed_name, decoder.record[name_offset..].as_ptr());
    let record_buffer = decoder.record.as_ptr();

    decoder.next_entry(None).unwrap().unwrap();
    assert_eq!(decoder.record.as_ptr(), record_buffer);
    decoder.next_entry(None).unwrap().unwrap();
    let depth_buffer = decoder.sibling_names[1].as_ptr();
    let depth_capacity = decoder.sibling_names[1].capacity();

    decoder.next_entry(None).unwrap().unwrap();
    assert_eq!(decoder.sibling_names[1].as_ptr(), depth_buffer);
    assert_eq!(decoder.sibling_names[1].capacity(), depth_capacity);
    assert!(decoder.sibling_names[1].is_empty());

    decoder.next_entry(None).unwrap().unwrap();
    assert_eq!(decoder.sibling_names[1].as_ptr(), depth_buffer);
    assert_eq!(decoder.sibling_names[1], leaf_name);
    assert!(decoder.next_entry(None).unwrap().is_none());
}

#[test]
fn compressed_snapshot_round_trips_and_replays() {
    let mut traversal = Traversal::new();
    let root_index = traversal.root_index;
    let root = add(
        &mut traversal,
        root_index,
        "root",
        EntryData {
            size: 42,
            is_dir: true,
            entry_count: Some(2),
            ..EntryData::default()
        },
    );
    add(
        &mut traversal,
        root,
        "child",
        EntryData {
            size: 7,
            ..EntryData::default()
        },
    );

    let bytes = compressed(&traversal, &[root]);
    assert!(is_zlib_header(&bytes));
    let snapshot = read(Cursor::new(&bytes)).unwrap();
    assert_eq!(
        snapshot
            .traversal
            .tree
            .data(snapshot.roots[0])
            .unwrap()
            .size,
        42
    );

    let mut replay = Replay::new(Cursor::new(bytes)).unwrap();
    for _ in 0..2 {
        let mut names = Vec::new();
        replay
            .for_each_entry(|entry| {
                names.push(entry.name().into_owned());
                Ok(())
            })
            .unwrap();
        assert_eq!(names, [PathBuf::from("root"), PathBuf::from("child")]);
    }
}

#[test]
fn rejects_invalid_compressed_envelopes() {
    let traversal = Traversal::new();
    let bytes = compressed(&traversal, &[]);

    assert!(read(Cursor::new(&bytes[..bytes.len() - 1])).is_err());

    let mut trailing = bytes.clone();
    trailing.push(0);
    let err = read(Cursor::new(trailing)).unwrap_err();
    assert!(err.to_string().contains("trailing"), "{err:#}");

    let mut concatenated = bytes.clone();
    concatenated.extend_from_slice(&bytes);
    let err = read(Cursor::new(concatenated)).unwrap_err();
    assert!(err.to_string().contains("trailing"), "{err:#}");
}

#[test]
fn rejects_noncanonical_sibling_order() {
    let root = native_record(1, FLAG_DIRECTORY, "root");
    let later = native_record(1, 0, "z");
    let earlier = native_record(2, 0, "a");

    assert!(
        read(Cursor::new(stream(&[root, later, earlier], 3)))
            .unwrap_err()
            .to_string()
            .contains("canonical order")
    );
}

#[test]
fn rejects_header_record_parent_and_name_errors() {
    let mut bad_magic = vec![0; HEADER_LEN];
    assert!(
        read(Cursor::new(&bad_magic))
            .unwrap_err()
            .to_string()
            .contains("magic")
    );

    bad_magic[..8].copy_from_slice(MAGIC);
    bad_magic[8..10].copy_from_slice(&VERSION.to_le_bytes());
    bad_magic[10] = PATH_ENCODING ^ 1;
    assert!(
        read(Cursor::new(&bad_magic))
            .unwrap_err()
            .to_string()
            .contains("path encoding")
    );

    assert!(
        read(Cursor::new(stream(&[record(0, 0, b"a", 0, 0, 0, None)], 1)))
            .unwrap_err()
            .to_string()
            .contains("parent distance")
    );

    assert!(
        read(Cursor::new(stream(
            &[record(1, 0x80, b"a", 0, 0, 0, None)],
            1
        )))
        .unwrap_err()
        .to_string()
        .contains("flags")
    );

    let file = native_record(1, 0, "file");
    let child = record(
        1,
        0,
        native_name_bytes(Path::new("child")).as_slice(),
        0,
        0,
        0,
        None,
    );
    assert!(
        read(Cursor::new(stream(&[file, child], 2)))
            .unwrap_err()
            .to_string()
            .contains("not a directory")
    );

    let directory = native_record(1, FLAG_DIRECTORY, "root");
    let child = record(
        1,
        0,
        native_name_bytes(Path::new(".")).as_slice(),
        0,
        0,
        0,
        None,
    );
    assert!(
        read(Cursor::new(stream(&[directory, child], 2)))
            .unwrap_err()
            .to_string()
            .contains("component")
    );

    let first_root = native_record(1, FLAG_DIRECTORY, "first");
    let second_root = native_record(2, FLAG_DIRECTORY, "second");
    let late_child = native_record(2, 0, "late");
    assert!(
        read(Cursor::new(stream(
            &[first_root, second_root, late_child],
            3
        )))
        .unwrap_err()
        .to_string()
        .contains("outside the current depth-first subtree")
    );
}

#[test]
fn rejects_mismatched_record_length_footer_and_timestamp() {
    let root = record(
        1,
        0,
        native_name_bytes(Path::new("root")).as_slice(),
        0,
        0,
        0,
        None,
    );
    let mut wrong_length = Vec::from([
        b'D',
        b'U',
        b'A',
        b'S',
        b'N',
        b'A',
        b'P',
        0,
        1,
        0,
        PATH_ENCODING,
        0,
    ]);
    push_uleb128(&mut wrong_length, (root.len() + 1) as u128);
    wrong_length.extend_from_slice(&root);
    wrong_length.extend_from_slice(&[0, 0, 1]);
    assert!(
        read(Cursor::new(rehash(wrong_length)))
            .unwrap_err()
            .to_string()
            .contains("record length")
    );

    assert!(
        read(Cursor::new(stream(&[root], 2)))
            .unwrap_err()
            .to_string()
            .contains("footer")
    );

    let bad_time = record(
        1,
        0,
        native_name_bytes(Path::new("root")).as_slice(),
        0,
        0,
        1_000_000_000,
        None,
    );
    assert!(
        read(Cursor::new(stream(&[bad_time], 1)))
            .unwrap_err()
            .to_string()
            .contains("nanoseconds")
    );
}

#[test]
fn rejects_noncanonical_overflowing_and_overlong_integers() {
    for malformed in [
        &[0x80, 0x00][..],
        &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02][..],
        &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80][..],
    ] {
        assert!(read_u64(&mut Cursor::new(malformed)).is_err());
    }
    assert!(read_u64(&mut Cursor::new([0x80])).is_err());
}

#[test]
fn rejects_record_and_name_allocation_limits() {
    let mut oversized_record = Vec::from([
        b'D',
        b'U',
        b'A',
        b'S',
        b'N',
        b'A',
        b'P',
        0,
        1,
        0,
        PATH_ENCODING,
        0,
    ]);
    push_uleb128(&mut oversized_record, (MAX_RECORD_LEN + 1) as u128);
    assert!(
        read(Cursor::new(oversized_record))
            .unwrap_err()
            .to_string()
            .contains("record exceeds")
    );

    let mut record = vec![1, 0];
    push_uleb128(&mut record, (MAX_NAME_LEN + 1) as u128);
    let mut oversized_name = Vec::from([
        b'D',
        b'U',
        b'A',
        b'S',
        b'N',
        b'A',
        b'P',
        0,
        1,
        0,
        PATH_ENCODING,
        0,
    ]);
    push_uleb128(&mut oversized_name, record.len() as u128);
    oversized_name.extend(record);
    assert!(
        read(Cursor::new(oversized_name))
            .unwrap_err()
            .to_string()
            .contains("name exceeds")
    );
}

#[cfg(windows)]
#[test]
fn rejects_odd_windows_name_bytes() {
    let record = record(1, 0, b"x", 0, 0, 0, None);
    assert!(
        read(Cursor::new(stream(&[record], 1)))
            .unwrap_err()
            .to_string()
            .contains("odd byte length")
    );
}

#[cfg(windows)]
#[test]
fn rejects_windows_timestamp_precision_loss() {
    let record = record(
        1,
        0,
        native_name_bytes(Path::new("root")).as_slice(),
        0,
        0,
        1,
        None,
    );
    assert!(
        read(Cursor::new(stream(&[record], 1)))
            .unwrap_err()
            .to_string()
            .contains("loses precision")
    );
}

fn add(
    traversal: &mut Traversal,
    parent: TreeIndex,
    name: impl AsRef<Path>,
    data: EntryData,
) -> TreeIndex {
    traversal.tree.add_child(parent, name, data)
}

fn encoded(traversal: &Traversal, roots: &[TreeIndex]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write(&mut bytes, traversal, roots, None).unwrap();
    bytes
}

fn compressed(traversal: &Traversal, roots: &[TreeIndex]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write(&mut bytes, traversal, roots, Some(2)).unwrap();
    bytes
}

fn rehash(mut bytes: Vec<u8>) -> Vec<u8> {
    let mut hash = gix::hash::hasher(gix::hash::Kind::Sha256);
    hash.update(&bytes);
    bytes.extend_from_slice(hash.try_finalize().unwrap().as_slice());
    bytes
}

fn record(
    parent_distance: u64,
    flags: u8,
    name: &[u8],
    size: u128,
    seconds: i64,
    nanos: u32,
    entry_count: Option<u64>,
) -> Vec<u8> {
    let mut record = Vec::new();
    push_uleb128(&mut record, u128::from(parent_distance));
    record.push(flags);
    push_uleb128(&mut record, name.len() as u128);
    record.extend_from_slice(name);
    push_uleb128(&mut record, size);
    push_uleb128(&mut record, u128::from(zigzag_encode(seconds)));
    push_uleb128(&mut record, u128::from(nanos));
    if let Some(count) = entry_count {
        push_uleb128(&mut record, u128::from(count));
    }
    record
}

fn native_record(parent_distance: u64, flags: u8, name: &str) -> Vec<u8> {
    record(
        parent_distance,
        flags,
        &native_name_bytes(Path::new(name)),
        0,
        0,
        0,
        None,
    )
}

fn stream(records: &[Vec<u8>], footer_count: u64) -> Vec<u8> {
    let mut bytes = Vec::from([
        b'D',
        b'U',
        b'A',
        b'S',
        b'N',
        b'A',
        b'P',
        0,
        1,
        0,
        PATH_ENCODING,
        0,
    ]);
    for record in records {
        push_uleb128(&mut bytes, record.len() as u128);
        bytes.extend_from_slice(record);
    }
    bytes.push(0);
    push_uleb128(&mut bytes, u128::from(footer_count));
    rehash(bytes)
}
