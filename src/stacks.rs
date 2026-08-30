use crate::aggregate::TraversalProgress;
use crate::snapshot::Replay;
use crate::traverse::{
    BackgroundTraversal, Traversal, TraversalEntry, TraversalEvent, Tree, TreeIndex,
};
use crate::tree::metadata_io_error_count;
use crate::{WalkOptions, WalkResult};
use anyhow::{Context, Result};
use bstr::ByteSlice;
use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;

/// Traverse `paths` and write the tree to `out` as folded stacks, one entry per line, ready
/// to pipe into flame-graph tools like [`inferno`](https://github.com/jonhoo/inferno).
///
/// Each line is an entry's path from the traversal root, with its components separated by `;`,
/// followed by a single space and the entry's own size in bytes. A directory contributes only the
/// size of its own directory entry, as the sizes of everything it contains appear on the lines of
/// the contained entries.
///
/// Without `max_depth`, each entry's own size is known when it is traversed, so its line can be
/// written immediately without retaining the complete tree. With `max_depth`, sizes below the
/// limit must be folded into their nearest visible ancestor. Because an ancestor is visited before
/// its descendants, its final size is unknown until traversal finishes; the requested levels are
/// therefore retained and written afterward instead of streamed.
pub fn stacks(
    mut out: impl io::Write,
    err: Option<impl io::Write>,
    walk_options: WalkOptions,
    paths: Vec<PathBuf>,
    max_depth: Option<usize>,
) -> Result<WalkResult> {
    let mut traversal = Traversal::new();
    let stream = max_depth.is_none();
    // Mirror the interactive traversal so root nodes carry their input path as name.
    let pattern_roots = walk_options.ignore_patterns.as_ref().map(|_| paths.clone());
    let mut background = BackgroundTraversal::start(
        traversal.root_index,
        &walk_options,
        paths,
        pattern_roots.as_deref(),
        false,
        true,
    )?
    .retain_depth(max_depth.or(Some(0)));
    let mut progress = TraversalProgress::new(err);

    while let Ok(event) = background.event_rx.recv() {
        let stack = stream.then(|| stack_path(&event)).flatten();
        let size_before = traversal
            .tree
            .data(traversal.root_index)
            .context("traversal root is missing")?
            .size;
        let finished = background.integrate_traversal_event(&mut traversal, event) == Some(true);
        let own_size = traversal
            .tree
            .data(traversal.root_index)
            .context("traversal root is missing")?
            .size
            .checked_sub(size_before)
            .context("traversal size decreased")?;
        if let Some(stack) = stack
            && own_size > 0
        {
            progress.clear();
            writeln!(out, "{stack} {own_size}")?;
        }
        progress.update(background.stats.entries_traversed);
        if finished {
            break;
        }
    }
    progress.clear();

    if !stream {
        let roots = background
            .root_nodes()
            .context("traversal did not produce a node for every root")?;
        write_stacks(&mut out, &traversal.tree, &roots, max_depth)?;
    }

    Ok(WalkResult {
        num_errors: background.stats.io_errors,
    })
}

/// Write an already completed traversal as folded stacks.
///
/// `roots` must contain the traversal's top-level nodes in their original input order. At
/// `max_depth`, an entry's line contains its full aggregate size, including hidden descendants.
/// The returned error count is derived from the stored metadata-error flags.
pub fn stacks_from_traversal(
    mut out: impl io::Write,
    traversal: &Traversal,
    roots: &[TreeIndex],
    max_depth: Option<usize>,
) -> Result<WalkResult> {
    write_stacks(&mut out, &traversal.tree, roots, max_depth)?;
    Ok(WalkResult {
        num_errors: metadata_io_error_count(&traversal.tree, roots),
    })
}

struct ReplayStackEntry {
    size: u128,
    children_size: u128,
    prefix_len: usize,
}

/// Replay a verified snapshot as folded stacks while retaining only the current path.
pub fn stacks_from_replay<R: io::Read + io::Seek>(
    mut out: impl io::Write,
    replay: &mut Replay<R>,
    max_depth: Option<usize>,
) -> Result<WalkResult> {
    let mut num_errors = 0u64;
    let mut open = Vec::new();
    let mut prefix = String::new();
    replay.for_each_entry(|entry| {
        num_errors = num_errors.saturating_add(u64::from(entry.data.metadata_io_error));
        while open.len() > entry.depth {
            write_replay_stack(&mut out, &mut open, &mut prefix)?;
        }
        if max_depth.is_some_and(|max_depth| entry.depth > max_depth) {
            return Ok(());
        }

        if let Some(parent) = open.last_mut() {
            parent.children_size = parent
                .children_size
                .checked_add(entry.data.size)
                .context("stack child sizes overflowed")?;
        }
        if !open.is_empty() {
            prefix.push(';');
        }
        push_frame(&mut prefix, entry.name().as_os_str());
        open.push(ReplayStackEntry {
            size: entry.data.size,
            children_size: 0,
            prefix_len: prefix.len(),
        });
        Ok(())
    })?;
    while !open.is_empty() {
        write_replay_stack(&mut out, &mut open, &mut prefix)?;
    }
    Ok(WalkResult { num_errors })
}

fn write_replay_stack(
    out: &mut impl io::Write,
    open: &mut Vec<ReplayStackEntry>,
    prefix: &mut String,
) -> Result<()> {
    let entry = open.pop().expect("called with an open stack entry");
    let own_size = entry
        .size
        .checked_sub(entry.children_size)
        .context("stack children exceed their parent's size")?;
    debug_assert_eq!(prefix.len(), entry.prefix_len);
    if own_size > 0 {
        writeln!(out, "{prefix} {own_size}")?;
    }
    prefix.truncate(open.last().map_or(0, |entry| entry.prefix_len));
    Ok(())
}

fn stack_path(event: &TraversalEvent) -> Option<String> {
    let TraversalEvent::Entry(Ok(TraversalEntry(entry)), root, _, _) = event else {
        return None;
    };
    let mut stack = frame_name(root.as_os_str());
    if entry.depth > 0 {
        for component in entry
            .path()
            .strip_prefix(root.as_path())
            .expect("walk entries remain below their root")
            .components()
        {
            stack.push(';');
            stack.push_str(&frame_name(component.as_os_str()));
        }
    }
    Some(stack)
}

/// Write every entry below `roots` as a folded stack line with its own (exclusive) size.
fn write_stacks(
    mut out: impl io::Write,
    tree: &Tree,
    roots: &[TreeIndex],
    max_depth: Option<usize>,
) -> Result<()> {
    // Depth-first, carrying the folded prefix that was built from the ancestors' names.
    let mut stack: Vec<(TreeIndex, usize, String)> = roots
        .iter()
        .rev()
        .map(|&root| (root, 0, frame(tree, root)))
        .collect();

    while let Some((index, depth, prefix)) = stack.pop() {
        let mut children_size = 0u128;
        if max_depth.is_none_or(|max_depth| depth < max_depth) {
            for child in tree.children(index) {
                children_size = children_size
                    .checked_add(tree.data(child).expect("tree child exists").size)
                    .context("stack child sizes overflowed")?;
                stack.push((child, depth + 1, format!("{prefix};{}", frame(tree, child))));
            }
        }
        // A directory's own size is what remains after accounting for its contents; a file has no
        // children and so contributes its entire size. Zero-sized entries are left out as they add
        // nothing to a flame graph.
        let own_size = tree
            .data(index)
            .expect("tree entry exists")
            .size
            .checked_sub(children_size)
            .context("stack children exceed their parent's size")?;
        if own_size > 0 {
            writeln!(out, "{prefix} {own_size}")?;
        }
    }
    Ok(())
}

/// Turn an entry name into a single flame-graph frame, encoding the `;` frame separator, control
/// characters, and the `\` escape marker.
fn frame(tree: &Tree, index: TreeIndex) -> String {
    frame_name(tree.name(index).expect("tree entry exists").as_os_str())
}

fn frame_name(name: &OsStr) -> String {
    let mut encoded = String::new();
    push_frame(&mut encoded, name);
    encoded
}

fn push_frame(encoded: &mut String, name: &OsStr) {
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
}

#[cfg(test)]
mod tests {
    use super::{Replay, frame_name, stacks, stacks_from_replay, stacks_from_traversal};
    use crate::traverse::{EntryData, Traversal};
    use crate::{TraversalOptions, WalkOptions};
    use bstr::ByteSlice;
    use std::{collections::BTreeMap, ffi::OsStr};

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

    /// Parse folded output into a map of stack -> size, tolerating names that contain spaces by
    /// splitting on the final space only.
    fn folded(out: &[u8]) -> BTreeMap<String, u128> {
        std::str::from_utf8(out)
            .unwrap()
            .lines()
            .map(|line| {
                let (stack, size) = line.rsplit_once(' ').expect("a size follows each stack");
                (stack.to_owned(), size.parse().expect("a numeric size"))
            })
            .collect()
    }

    fn folded_frame(path: impl Into<std::path::PathBuf>) -> String {
        frame_name(path.into().as_os_str())
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
            folded_total,
            traversal
                .tree
                .data(traversal.root_index)
                .expect("traversal root exists")
                .size,
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

    #[cfg(unix)]
    #[test]
    fn output_is_written_while_the_walk_is_still_running() {
        struct CreateFileOnFirstWrite {
            out: Vec<u8>,
            path: std::path::PathBuf,
        }

        impl std::io::Write for CreateFileOnFirstWrite {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                if self.out.is_empty() {
                    std::fs::write(&self.path, b"x")?;
                }
                self.out.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let dir = tempfile::tempdir().unwrap();
        let mut deepest = dir.path().to_owned();
        for _ in 0..200 {
            deepest.push("d");
        }
        std::fs::create_dir_all(&deepest).unwrap();
        std::fs::write(dir.path().join("trigger"), b"x").unwrap();
        let mut out = CreateFileOnFirstWrite {
            out: Vec::new(),
            path: deepest.join("late"),
        };

        stacks(
            &mut out,
            None::<Vec<u8>>,
            walk_options(),
            vec![dir.path().to_owned()],
            None,
        )
        .unwrap();

        let text = String::from_utf8(out.out).unwrap();
        assert!(
            text.lines().any(|line| line.ends_with(";late 1")),
            "the first stack line should be written before the deepest directory is read: {text:?}"
        );
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
    }

    #[test]
    fn completed_traversal_rolls_hidden_entries_into_the_cutoff() {
        let mut traversal = Traversal::new();
        let root = traversal.tree.add_child(
            traversal.root_index,
            "first",
            EntryData {
                size: 9,
                is_dir: true,
                ..EntryData::default()
            },
        );
        let cutoff = traversal.tree.add_child(
            root,
            "cutoff",
            EntryData {
                size: 9,
                is_dir: true,
                ..EntryData::default()
            },
        );
        traversal.tree.add_child(
            cutoff,
            "hidden",
            EntryData {
                size: 7,
                metadata_io_error: true,
                ..EntryData::default()
            },
        );
        let second = traversal.tree.add_child(
            traversal.root_index,
            "second",
            EntryData {
                size: 2,
                ..EntryData::default()
            },
        );

        let mut out = Vec::new();
        let result = stacks_from_traversal(&mut out, &traversal, &[root, second], Some(1)).unwrap();
        let mut snapshot = Vec::new();
        crate::snapshot::write(&mut snapshot, &traversal, &[root, second], None).unwrap();
        let mut replay = Replay::new(std::io::Cursor::new(snapshot)).unwrap();
        let mut replayed = Vec::new();
        let replayed_result = stacks_from_replay(&mut replayed, &mut replay, Some(1)).unwrap();
        assert_eq!(folded(&replayed), folded(&out));
        assert_eq!(replayed_result.num_errors, result.num_errors);
        insta::assert_snapshot!(out.as_bstr(), "depth cutoff rolls hidden descendants into parent", @r"
        first;cutoff 9
        second 2
        ");
        assert_eq!(result.num_errors, 1);
    }

    #[test]
    fn completed_traversal_rejects_invalid_aggregate_sizes() {
        for (parent_size, child_sizes, expected) in [
            (1, vec![2], "stack children exceed their parent's size"),
            (
                u128::MAX,
                vec![u128::MAX, 1],
                "stack child sizes overflowed",
            ),
        ] {
            let mut traversal = Traversal::new();
            let root = traversal.tree.add_child(
                traversal.root_index,
                "root",
                EntryData {
                    size: parent_size,
                    is_dir: true,
                    ..EntryData::default()
                },
            );
            for size in child_sizes {
                traversal.tree.add_child(
                    root,
                    "child",
                    EntryData {
                        size,
                        ..EntryData::default()
                    },
                );
            }

            let Err(error) = stacks_from_traversal(Vec::new(), &traversal, &[root], None) else {
                panic!("invalid aggregate sizes must fail");
            };
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn frame_names_are_encoded_without_collisions() {
        let encode = |name: &str| frame_name(OsStr::new(name));

        assert_eq!(encode("a;b"), r"a\x3bb");
        assert_eq!(encode("a_b"), "a_b");
        assert_eq!(encode(r"a\x3bb"), r"a\\x3bb");
        assert_eq!(encode("a\nb"), r"a\nb");
    }
}
