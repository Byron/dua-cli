use crate::aggregate::TraversalProgress;
use crate::{InodeFilter, WalkOptions, WalkResult, WalkRoot, crossdev};
use anyhow::Result;
use bstr::ByteSlice;
#[cfg(not(any(windows, target_os = "macos")))]
use filesize::PathExt;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(not(any(windows, target_os = "macos")))]
fn size_on_disk(entry: &crate::walk::Entry, metadata: &crate::walk::Metadata) -> io::Result<u64> {
    entry.path().size_on_disk_fast(metadata)
}

#[cfg(windows)]
#[allow(clippy::unnecessary_wraps)]
fn size_on_disk(entry: &crate::walk::Entry, metadata: &crate::walk::Metadata) -> io::Result<u64> {
    Ok(if entry.file_type.is_dir() {
        0
    } else {
        metadata.allocated_size()
    })
}

/// Traverse `paths` and write entries to `out` as folded stacks as soon as they are discovered,
/// ready to pipe into flame-graph tools like [`inferno`](https://github.com/jonhoo/inferno).
///
/// Each line is an entry's path from the traversal root, with its components separated by `;`,
/// followed by a single space and the entry's own size in bytes. A directory contributes only its
/// own directory-entry size, as the sizes of everything it contains appear on separate lines.
/// When `max_depth` truncates multiple entries to the same stack, their sizes are combined.
pub fn stacks(
    mut out: impl io::Write,
    err: Option<impl io::Write>,
    walk_options: WalkOptions,
    paths: Vec<PathBuf>,
    max_depth: Option<usize>,
) -> Result<WalkResult> {
    let mut result = WalkResult::default();
    let mut roots = Vec::with_capacity(paths.len());
    let mut device_ids = vec![0; paths.len()];
    let has_ignore_patterns = walk_options.ignore_patterns.is_some();

    for (root_idx, path) in paths.iter().enumerate() {
        let device_id = if walk_options.cross_filesystems {
            0
        } else {
            let Ok(device_id) = crossdev::init(path) else {
                result.num_errors += 1;
                continue;
            };
            device_id
        };
        device_ids[root_idx] = device_id;
        roots.push(WalkRoot {
            index: root_idx,
            pattern_root: has_ignore_patterns.then(|| path.clone()),
            path: path.clone(),
            #[cfg(any(windows, target_os = "macos"))]
            entry: None,
            device_id,
        });
    }

    let mut progress = TraversalProgress::new(err);
    // Entries contain their own metadata size and full parent path, so completion order can retain
    // the walker's full parallelism without building a parent-first aggregate tree.
    let events = walk_options.iter_from_paths(roots, false, crate::walk::Order::Completion);
    let write_result = write_stack_events(
        &mut out,
        &mut progress,
        &walk_options,
        &paths,
        &device_ids,
        max_depth,
        &mut result,
        events,
    );
    progress.clear();
    write_result?;

    Ok(result)
}

/// Consume traversal events and write each uncollapsed entry before requesting the next event.
#[expect(
    clippy::too_many_arguments,
    reason = "the streaming state stays borrowed rather than copied into another owner"
)]
fn write_stack_events<W: io::Write, E: io::Write>(
    out: &mut W,
    progress: &mut TraversalProgress<E>,
    walk_options: &WalkOptions,
    paths: &[PathBuf],
    device_ids: &[u64],
    max_depth: Option<usize>,
    result: &mut WalkResult,
    events: impl Iterator<Item = (usize, crate::walk::RootEvent)>,
) -> io::Result<()> {
    let mut inodes = InodeFilter::default();
    let mut entries_traversed = 0;
    let mut depth_aggregates = HashMap::<String, u128>::new();

    for (root_idx, event) in events {
        let entry = match event {
            crate::walk::RootEvent::Entry(entry) => entry,
            crate::walk::RootEvent::Finished => continue,
        };
        entries_traversed += 1;
        progress.update(entries_traversed);

        let Ok(entry) = entry else {
            result.num_errors += 1;
            continue;
        };
        let own_size = u128::from(match &entry.metadata {
            Ok(metadata)
                if (walk_options.count_hard_links || inodes.add(&entry, metadata))
                    && (walk_options.cross_filesystems
                        || crossdev::is_same_device(device_ids[root_idx], metadata)) =>
            {
                if walk_options.apparent_size {
                    metadata.len()
                } else {
                    #[cfg(target_os = "macos")]
                    if walk_options.metadata_options.apfs_clone_metadata {
                        inodes.allocated_size(metadata)
                    } else {
                        metadata.allocated_size()
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        size_on_disk(&entry, metadata).unwrap_or_else(|_| {
                            result.num_errors += 1;
                            0
                        })
                    }
                }
            }
            Ok(_) => 0,
            Err(_) => {
                result.num_errors += 1;
                0
            }
        });

        if own_size > 0 {
            let stack = stack_for_entry(&entry, &paths[root_idx], max_depth);
            if max_depth.is_some_and(|depth| entry.depth >= depth) {
                *depth_aggregates.entry(stack).or_default() += own_size;
            } else {
                writeln!(out, "{stack} {own_size}")?;
            }
        }
    }
    progress.clear();

    for (stack, size) in depth_aggregates {
        writeln!(out, "{stack} {size}")?;
    }

    Ok(())
}

/// Build one entry's stack, truncating the path when requested.
fn stack_for_entry(entry: &crate::walk::Entry, root: &Path, max_depth: Option<usize>) -> String {
    let mut stack = frame(root.as_os_str());
    if entry.depth > 0 {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("walk entries stay below their root");
        let depth = max_depth.map_or(entry.depth, |depth| depth.min(entry.depth));
        for component in relative.components().take(depth) {
            stack.push(';');
            stack.push_str(&frame(component.as_os_str()));
        }
    }
    stack
}

/// Turn an entry name into a single flame-graph frame, encoding the `;` frame separator, control
/// characters, and the `\` escape marker.
fn frame(name: &OsStr) -> String {
    let mut encoded = String::new();
    for chunk in name.as_encoded_bytes().utf8_chunks() {
        for character in chunk.valid().chars() {
            match character {
                '\\' => encoded.push_str(r"\\"),
                ';' => encoded.push_str(r"\x3b"),
                character if character.is_control() => encoded.extend(character.escape_default()),
                character => encoded.push(character),
            }
        }
        encoded.extend(chunk.invalid().escape_bytes());
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{frame, stacks, write_stack_events};
    use crate::{TraversalOptions, WalkOptions};
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::io;
    use std::rc::Rc;

    fn walk_options() -> WalkOptions {
        WalkOptions {
            threads: 1,
            count_hard_links: true,
            apparent_size: true,
            cross_filesystems: true,
            ignore_dirs: std::collections::BTreeSet::default(),
            ignore_patterns: None,
            metadata_options: TraversalOptions::default(),
        }
    }

    /// Parse folded output into a map of stack -> total size, tolerating names that contain spaces.
    fn folded(out: &[u8]) -> BTreeMap<String, u128> {
        let mut folded = BTreeMap::new();
        for line in std::str::from_utf8(out).unwrap().lines() {
            let (stack, size) = line.rsplit_once(' ').expect("a size follows each stack");
            *folded.entry(stack.to_owned()).or_default() +=
                size.parse::<u128>().expect("a numeric size");
        }
        folded
    }

    fn folded_frame(path: impl AsRef<std::path::Path>) -> String {
        frame(path.as_ref().as_os_str())
    }

    struct MarkingWriter {
        bytes: Vec<u8>,
        wrote: Rc<Cell<bool>>,
    }

    impl io::Write for MarkingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.wrote.set(true);
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn writes_an_entry_before_requesting_the_next_event() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("first");
        std::fs::write(&file, b"content").unwrap();
        let entry = crate::walk::Entry::from_path(&file, TraversalOptions::default()).unwrap();

        let wrote = Rc::new(Cell::new(false));
        let wrote_before_next = Rc::clone(&wrote);
        let mut events = [
            (0, crate::walk::RootEvent::Entry(Ok(entry))),
            (0, crate::walk::RootEvent::Finished),
        ]
        .into_iter();
        let mut events_requested = 0;
        let events = std::iter::from_fn(move || {
            if events_requested > 0 {
                assert!(
                    wrote_before_next.get(),
                    "the previous entry must be written before another event is requested"
                );
            }
            events_requested += 1;
            events.next()
        });
        let mut out = MarkingWriter {
            bytes: Vec::new(),
            wrote,
        };
        let mut progress = crate::aggregate::TraversalProgress::new(None::<Vec<u8>>);
        let mut result = crate::WalkResult::default();

        write_stack_events(
            &mut out,
            &mut progress,
            &walk_options(),
            std::slice::from_ref(&file),
            &[0],
            None,
            &mut result,
            events,
        )
        .unwrap();

        assert_eq!(result.num_errors, 0);
        assert_eq!(folded(&out.bytes)[&folded_frame(file)], 7);
    }

    #[test]
    fn every_file_appears_with_its_size_below_its_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested/file"), b"content").unwrap();
        std::fs::write(dir.path().join("top"), b"hi").unwrap();

        let root = dir.path().to_owned();
        let mut out = Vec::new();
        let result = stacks(
            &mut out,
            None::<Vec<u8>>,
            walk_options(),
            vec![root.clone()],
            None,
        )
        .unwrap();
        assert_eq!(result.num_errors, 0);

        let folded = folded(&out);
        let base = folded_frame(root);
        assert_eq!(
            folded.get(&format!("{base};nested;file")),
            Some(&7),
            "the nested file is folded under its two directories with its own size"
        );
        assert_eq!(
            folded.get(&format!("{base};top")),
            Some(&2),
            "the top-level file appears directly under the root"
        );
    }

    #[test]
    fn folded_sizes_sum_to_the_reported_total() {
        use crate::traverse::{BackgroundTraversal, Traversal};

        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("a")).unwrap();
        std::fs::write(dir.path().join("a/one"), b"12345").unwrap();
        std::fs::write(dir.path().join("two"), b"678").unwrap();

        let mut out = Vec::new();
        stacks(
            &mut out,
            None::<Vec<u8>>,
            walk_options(),
            vec![dir.path().to_owned()],
            None,
        )
        .unwrap();
        let folded_total: u128 = folded(&out).values().sum();

        // Independently traverse the same tree to obtain the total `dua` itself reports.
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start(
            traversal.root_index,
            &walk_options(),
            vec![dir.path().to_owned()],
            None,
            false,
            true,
        )
        .unwrap();
        while background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            != Some(true)
        {}

        assert_eq!(
            folded_total, traversal.tree[traversal.root_index].size,
            "the folded lines account for every byte the traversal totals up"
        );
    }

    #[test]
    fn a_single_file_input_is_folded_as_one_line() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("solo");
        std::fs::write(&file, b"solo!").unwrap();

        let mut out = Vec::new();
        stacks(
            &mut out,
            None::<Vec<u8>>,
            walk_options(),
            vec![file.clone()],
            None,
        )
        .unwrap();

        let folded = folded(&out);
        assert_eq!(folded.len(), 1);
        assert_eq!(folded.get(&folded_frame(file)), Some(&5));
    }

    #[test]
    fn depth_rolls_hidden_descendants_into_the_last_frame() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested/file"), b"content").unwrap();

        let mut out = Vec::new();
        stacks(
            &mut out,
            None::<Vec<u8>>,
            walk_options(),
            vec![dir.path().to_owned()],
            Some(1),
        )
        .unwrap();

        let folded = folded(&out);
        let nested = format!("{};nested", folded_frame(dir.path()));
        assert!(folded.contains_key(&nested));
        assert!(!folded.keys().any(|stack| stack.ends_with(";file")));
        assert_eq!(
            std::str::from_utf8(&out)
                .unwrap()
                .lines()
                .filter(|stack| stack.starts_with(&nested))
                .count(),
            1,
            "a collapsed stack is emitted once"
        );
        assert_eq!(
            folded[&nested],
            u128::from(std::fs::metadata(dir.path().join("nested")).unwrap().len()) + 7,
            "the boundary frame includes its own size and every hidden descendant"
        );
    }

    #[test]
    fn frame_names_are_encoded_without_collisions() {
        let encode = |name: &str| frame(std::ffi::OsStr::new(name));

        assert_eq!(encode("a;b"), r"a\x3bb");
        assert_eq!(encode("a_b"), "a_b");
        assert_eq!(encode(r"a\x3bb"), r"a\\x3bb");
        assert_eq!(encode("a\nb"), r"a\nb");
    }
}
