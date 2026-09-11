use super::*;
use crate::Options;
use std::os::unix::fs::PermissionsExt as _;

fn options(apfs_clone_metadata: bool) -> Options {
    Options {
        apfs_clone_metadata,
        skip_metadata: false,
    }
}

#[test]
fn parallel_metadata_requires_repeated_waiting_regardless_of_clock_speed() {
    let directory = tempfile::tempdir().unwrap();
    for unit in [Duration::from_micros(1), Duration::from_millis(10)] {
        let mut reader = ReadDir::open(Arc::from(directory.path()), 1, options(false)).unwrap();
        for (elapsed, cpu, message) in [
            (unit * 2, Some(unit * 2), "Busy, however slow the CPU."),
            (unit * 3, Some(unit), "One waiting refill is insufficient."),
            (
                unit * 2,
                Some(unit),
                "Waiting must exceed CPU time; reset the streak.",
            ),
            (unit * 3, Some(unit), "One waiting refill is insufficient."),
            (unit * 3, None, "A missing CPU measurement also resets it."),
            (unit * 3, Some(unit), "One waiting refill is insufficient."),
            (
                unit * 3,
                Some(Duration::ZERO),
                "Ignore measurements below clock resolution.",
            ),
            (unit * 3, Some(unit), "One waiting refill is insufficient."),
        ] {
            reader.record_metadata_time(elapsed, cpu);
            assert!(!reader.metadata_is_io_bound(), "{message}");
        }
        reader.record_metadata_time(unit * 3, Some(unit));
        assert!(
            reader.metadata_is_io_bound(),
            "repeated waiting should expose parallel work"
        );
    }
}

#[test]
fn cpu_bound_probe_leaves_directory_stats_for_streaming() {
    let directory = tempfile::tempdir().unwrap();
    let child = directory.path().join("child");
    fs::create_dir(&child).unwrap();
    let mut reader = ReadDir::open(Arc::from(directory.path()), 1, options(false)).unwrap();
    // Force a CPU-bound sample without depending on filesystem latency or scheduling.
    let mut cpu_times = [Some(Duration::ZERO), Some(Duration::MAX)].into_iter();
    let prefix = reader.probe_metadata_with_cpu_time(|| {
        cpu_times.next().expect("only one refill should be probed")
    });
    assert!(
        prefix.is_empty(),
        "CPU-bound probing must not parse entries"
    );

    fs::remove_dir(child).unwrap();
    let entry = reader.next().unwrap().unwrap();
    assert_eq!(entry.file_name, "child");
    let error = entry
        .metadata
        .unwrap()
        .err()
        .expect("the directory stat must happen after the probe");
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(
        reader.next().is_none(),
        "this would be Some if the entry was cached during the probe"
    );
}

#[test]
fn metadata_and_entries_keep_clone_tracking_compact() {
    assert_eq!(
        size_of::<Metadata>(),
        80,
        "effective data allocation and clone identity must not require another discriminant"
    );
    assert_eq!(
        size_of::<Entry>(),
        144,
        "native directory entries must retain compact inline metadata"
    );
}

#[test]
fn directory_reader_rejects_regular_files_before_iteration() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("file");
    fs::write(&file, b"content").unwrap();

    let error = ReadDir::open(Arc::from(file), 0, options(false))
        .err()
        .expect("regular files must be rejected before iteration");
    assert_eq!(
        error.raw_os_error(),
        Some(libc::ENOTDIR),
        "opening a regular file as a directory must fail with ENOTDIR"
    );
}

#[test]
fn fallback_reader_enumerates_entries_with_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let fallback_directory = tempfile::tempdir().unwrap();
    fs::write(fallback_directory.path().join("fallback-file"), b"content").unwrap();

    let mut reader = ReadDir::open(Arc::from(directory.path()), 1, options(false)).unwrap();
    reader.fallback = Some(fs::read_dir(fallback_directory.path()).unwrap());
    assert!(
        reader.probe_metadata().is_empty(),
        "a bulk probe must not buffer the fallback iterator"
    );
    let entry = reader.next().unwrap().unwrap();

    assert_eq!(
        entry.file_name, "fallback-file",
        "the native directory is empty, so this entry must come from the injected fallback reader"
    );
    assert_eq!(
        entry.metadata.unwrap().unwrap().len(),
        7,
        "the fallback reader must collect per-entry metadata"
    );
}

#[test]
fn type_only_reads_do_not_require_metadata_access() {
    let directory = tempfile::tempdir().unwrap();
    let restricted = directory.path().join("restricted");
    fs::create_dir(&restricted).unwrap();
    fs::write(restricted.join("file"), b"content").unwrap();
    fs::create_dir(restricted.join("child")).unwrap();
    fs::set_permissions(&restricted, fs::Permissions::from_mode(0o400)).unwrap();

    let entries = crate::read_dir(
        &restricted,
        Options {
            skip_metadata: true,
            ..options(true)
        },
    )
    .unwrap()
    .collect::<Vec<_>>();
    fs::set_permissions(&restricted, fs::Permissions::from_mode(0o700)).unwrap();

    assert_eq!(entries.len(), 2);
    for entry in entries {
        let entry = entry.unwrap();
        assert!(entry.metadata.is_none());
        assert_eq!(entry.file_type.is_dir(), entry.file_name == "child");
        assert_eq!(entry.file_type.is_file(), entry.file_name == "file");
    }
}

#[test]
fn bulk_metadata_matches_std_for_regular_files_resource_forks_and_symlinks() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("file");
    fs::write(&file, b"data").unwrap();
    let data_blocks = fs::metadata(&file).unwrap().blocks();
    let resource = file.join("..namedfork/rsrc");
    fs::write(resource, [7; 8192]).unwrap();
    let resource_blocks = fs::metadata(&file).unwrap().blocks();
    assert!(
        resource_blocks > data_blocks,
        "resource fork must allocate storage: data={data_blocks}, total={resource_blocks}"
    );
    std::os::unix::fs::symlink("file", directory.path().join("link")).unwrap();

    let ordinary_metadata = ReadDir::open(Arc::from(directory.path()), 1, options(false))
        .unwrap()
        .find(|entry| entry.as_ref().is_ok_and(|entry| entry.file_name == "file"))
        .expect("the resource-fork fixture must be enumerated")
        .unwrap()
        .metadata
        .unwrap()
        .unwrap();
    assert_eq!(
        ordinary_metadata.data_allocated_size(),
        ordinary_metadata.allocated_size(),
        "without extended metadata, data allocation must fall back to total allocation"
    );

    let entries = ReadDir::open(Arc::from(directory.path()), 1, options(true))
        .unwrap()
        .collect::<io::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(
        entries.len(),
        2,
        "regular file and symbolic link must both be enumerated"
    );
    for entry in entries {
        let direct = fs::symlink_metadata(entry.path()).unwrap();
        let metadata = entry.metadata.unwrap().unwrap();
        assert_eq!(
            metadata.len(),
            direct.len(),
            "{:?} logical size",
            entry.file_name
        );
        assert_eq!(
            metadata.allocated_size(),
            direct.blocks() * STAT_BLOCK_BYTES,
            "{:?} allocated size",
            entry.file_name
        );
        if entry.file_name == "file" {
            assert_eq!(
                metadata.data_allocated_size(),
                data_blocks * STAT_BLOCK_BYTES,
                "resource-fork allocation must not be mistaken for shared data"
            );
        }
        assert_eq!(
            metadata.dev(),
            direct.dev(),
            "{:?} device number",
            entry.file_name
        );
        assert_eq!(
            metadata.ino(),
            direct.ino(),
            "{:?} inode number",
            entry.file_name
        );
        assert_eq!(
            metadata.modified().unwrap(),
            direct.modified().unwrap(),
            "{:?} modification time",
            entry.file_name
        );
        assert_eq!(
            entry.file_type.is_symlink(),
            direct.file_type().is_symlink(),
            "{:?} symbolic-link type",
            entry.file_name
        );
    }
}

#[test]
fn bulk_metadata_preserves_fractional_pre_epoch_timestamps() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("file");
    let file = fs::File::create(&path).unwrap();
    let expected = std::time::UNIX_EPOCH - std::time::Duration::from_millis(900);
    file.set_times(fs::FileTimes::new().set_modified(expected))
        .unwrap();
    let direct = file.metadata().unwrap().modified().unwrap();

    let entry = ReadDir::open(Arc::from(directory.path()), 1, options(false))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();

    assert_eq!(
        direct, expected,
        "the filesystem must preserve the fractional pre-epoch timestamp used by this test"
    );
    assert_eq!(
        entry.metadata.unwrap().unwrap().modified().unwrap(),
        direct,
        "bulk metadata must preserve fractional pre-epoch timestamps"
    );
}

#[test]
fn readable_directory_without_search_permission_preserves_entry_errors() {
    let directory = tempfile::tempdir().unwrap();
    let restricted = directory.path().join("restricted");
    fs::create_dir(&restricted).unwrap();
    fs::write(restricted.join("file"), b"content").unwrap();
    fs::set_permissions(&restricted, fs::Permissions::from_mode(0o400)).unwrap();

    let entries = ReadDir::open(Arc::from(restricted.as_path()), 1, options(false))
        .unwrap()
        .collect::<Vec<_>>();
    fs::set_permissions(&restricted, fs::Permissions::from_mode(0o700)).unwrap();

    assert_eq!(
        entries.len(),
        1,
        "the readable directory must still enumerate its single entry"
    );
    let entry = entries.into_iter().next().unwrap().unwrap();
    assert_eq!(
        entry.file_name, "file",
        "directory enumeration must retain the entry name"
    );
    assert_eq!(
        entry.file_type.kind, VREG,
        "list-only metadata must retain the regular-file type"
    );
    let error = entry
        .metadata
        .unwrap()
        .err()
        .expect("metadata must retain the directory search-permission error");
    assert_eq!(
        error.kind(),
        io::ErrorKind::PermissionDenied,
        "the entry metadata error must preserve the directory permission failure"
    );
}

#[test]
fn bulk_metadata_identifies_clones_and_hard_links() {
    use std::os::unix::fs::FileExt as _;

    let directory = tempfile::tempdir().unwrap();
    let original = directory.path().join("original");
    let clone = directory.path().join("clone");
    let partial_clone = directory.path().join("partial-clone");
    fs::write(&original, [7; 8192]).unwrap();

    // On Apple platforms, std::fs::copy first tries fclonefileat(2), producing distinct
    // copy-on-write APFS inodes that initially share their data blocks. A non-APFS fallback
    // copies the bytes instead and intentionally fails the clone-ID assertions below.
    fs::copy(&original, &clone).unwrap();
    fs::copy(&original, &partial_clone).unwrap();
    let bytes_written = fs::OpenOptions::new()
        .write(true)
        .open(&partial_clone)
        .unwrap()
        .write_at(&[9], 0)
        .unwrap();
    assert_eq!(
        bytes_written, 1,
        "modifying one byte must diverge the partially cloned data fork"
    );
    fs::hard_link(&original, directory.path().join("hard-link")).unwrap();
    fs::write(clone.join("..namedfork/rsrc"), [3; 8192]).unwrap();
    std::os::unix::fs::symlink("original", directory.path().join("link")).unwrap();
    std::os::unix::fs::symlink("missing", directory.path().join("broken")).unwrap();

    let ordinary = ReadDir::open(Arc::from(directory.path()), 1, options(false))
        .unwrap()
        .find(|entry| {
            entry
                .as_ref()
                .is_ok_and(|entry| entry.file_name == "original")
        })
        .unwrap()
        .unwrap();
    assert_eq!(ordinary.metadata.unwrap().unwrap().clone_id(), None);
    assert_eq!(
        Entry::from_path(&original, options(false))
            .unwrap()
            .metadata
            .unwrap()
            .unwrap()
            .clone_id(),
        None
    );

    let entries = ReadDir::open(Arc::from(directory.path()), 1, options(true))
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name, entry.metadata.unwrap().unwrap())
        })
        .collect::<std::collections::HashMap<_, _>>();
    let reader = crate::read_dir(directory.path(), options(true).skip_metadata()).unwrap();
    for entry in reader {
        let entry = entry.unwrap();
        assert!(entry.metadata.is_none());
        let expected = &entries[&entry.file_name];
        let metadata = entry
            .read_metadata(options(true))
            .metadata
            .unwrap()
            .unwrap();
        assert_eq!(metadata.ino(), expected.ino());
        assert_eq!(metadata.nlink(), expected.nlink());
        assert_eq!(metadata.len(), expected.len());
        assert_eq!(metadata.allocated_size(), expected.allocated_size());
        assert_eq!(
            metadata.data_allocated_size(),
            expected.data_allocated_size()
        );
        assert_eq!(metadata.clone_id(), expected.clone_id());
    }
    let original = &entries[std::ffi::OsStr::new("original")];
    let cloned = &entries[std::ffi::OsStr::new("clone")];
    let partially_cloned = &entries[std::ffi::OsStr::new("partial-clone")];
    let hard_link = &entries[std::ffi::OsStr::new("hard-link")];

    assert_eq!(
        original.ino(),
        hard_link.ino(),
        "hard links must report the same inode"
    );
    assert_eq!(original.nlink(), 2, "bulk metadata must report both links");
    assert_ne!(
        original.ino(),
        cloned.ino(),
        "APFS clones must have distinct inodes"
    );
    let clone_id = original
        .clone_id()
        .expect("APFS must report a shared identifier for the cloned fixture");
    assert_eq!(
        cloned.clone_id(),
        Some(clone_id),
        "fully cloned data forks must report the same content identifier"
    );
    assert_ne!(
        partially_cloned.clone_id(),
        Some(clone_id),
        "partially shared data forks must not claim the original clone identity"
    );
    let root = Entry::from_path(&clone, options(true))
        .unwrap()
        .metadata
        .unwrap()
        .unwrap();
    assert_eq!(
        root.clone_id(),
        Some(clone_id),
        "explicit file roots must retain the bulk-enumerated clone identity"
    );
}

#[test]
fn bulk_device_numbers_match_std_on_devfs() {
    let directory = Path::new("/dev");
    let entry = ReadDir::open(Arc::from(directory), 1, options(false))
        .unwrap()
        .find(|entry| entry.as_ref().is_ok_and(|entry| entry.file_name == "null"))
        .expect("devfs contains /dev/null")
        .unwrap();
    let expected = fs::symlink_metadata(directory.join("null")).unwrap();

    assert_eq!(
        entry.metadata.unwrap().unwrap().dev(),
        expected.dev(),
        "bulk metadata must report the devfs device number returned by stat"
    );
}

#[test]
fn bulk_directory_reader_refills_its_buffer() {
    let directory = tempfile::tempdir().unwrap();
    for index in 0..700 {
        fs::write(
            directory
                .path()
                .join(format!("{index:04}-{}", "x".repeat(100))),
            [],
        )
        .unwrap();
    }

    for include_apfs in [false, true] {
        let mut reader =
            ReadDir::open(Arc::from(directory.path()), 1, options(include_apfs)).unwrap();
        let prefix = reader.probe_metadata();
        assert!(
            prefix.len() < 700,
            "the initial probe must buffer only a bounded prefix"
        );
        let mut names = std::collections::HashSet::new();
        for entry in prefix.into_iter().chain(reader) {
            let entry = entry.unwrap();
            assert!(entry.metadata.is_some(), "retain metadata from the probe");
            assert!(
                names.insert(entry.file_name),
                "never repeat a buffered entry"
            );
        }
        assert_eq!(names.len(), 700, "buffering must not skip entries");
    }
}
