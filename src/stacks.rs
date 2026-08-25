use crate::traverse::{BackgroundTraversal, EntryData, Traversal, Tree, TreeIndex};
use crate::{WalkOptions, WalkResult};
use anyhow::Result;
use bstr::ByteSlice;
use petgraph::Direction;
use std::io;
use std::path::PathBuf;

/// Traverse `paths` and write the tree to `out` as folded stacks, one entry per line, ready
/// to pipe into flame-graph tools like [`inferno`](https://github.com/jonhoo/inferno).
///
/// Each line is an entry's path from the traversal root, with its components separated by `;`,
/// followed by a single space and the entry's own size in bytes. A directory contributes only the
/// size of its own directory entry, as the sizes of everything it contains appear on the lines of
/// the contained entries.
pub fn stacks(
    mut out: impl io::Write,
    walk_options: WalkOptions,
    paths: Vec<PathBuf>,
    max_depth: Option<usize>,
) -> Result<WalkResult> {
    let mut traversal = Traversal::new();
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
    .retain_depth(max_depth);

    loop {
        let event = background.event_rx.recv()?;
        if background.integrate_traversal_event(&mut traversal, event) == Some(true) {
            break;
        }
    }

    write_stacks(&mut out, &traversal.tree, traversal.root_index)?;

    Ok(WalkResult {
        num_errors: background.stats.io_errors,
    })
}

/// Write every entry below `root` as a folded stack line with its own (exclusive) size.
fn write_stacks(mut out: impl io::Write, tree: &Tree, root: TreeIndex) -> io::Result<()> {
    // Depth-first, carrying the folded prefix that was built from the ancestors' names.
    let mut stack: Vec<(TreeIndex, String)> = tree
        .neighbors_directed(root, Direction::Outgoing)
        .map(|child| (child, frame(&tree[child])))
        .collect();

    while let Some((index, prefix)) = stack.pop() {
        let mut children_size = 0u128;
        for child in tree.neighbors_directed(index, Direction::Outgoing) {
            children_size += tree[child].size;
            stack.push((child, format!("{prefix};{}", frame(&tree[child]))));
        }
        // A directory's own size is what remains after accounting for its contents; a file has no
        // children and so contributes its entire size. Zero-sized entries are left out as they add
        // nothing to a flame graph.
        let own_size = tree[index].size.saturating_sub(children_size);
        if own_size > 0 {
            writeln!(out, "{prefix} {own_size}")?;
        }
    }
    Ok(())
}

/// Turn an entry name into a single flame-graph frame, encoding the `;` frame separator, control
/// characters, and the `\` escape marker.
fn frame(entry: &EntryData) -> String {
    let mut encoded = String::new();
    for chunk in entry.name.as_os_str().as_encoded_bytes().utf8_chunks() {
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
    use super::{frame, stacks};
    use crate::traverse::EntryData;
    use crate::{TraversalOptions, WalkOptions};
    use std::collections::BTreeMap;

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

    #[test]
    fn every_file_appears_with_its_size_below_its_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested/file"), b"content").unwrap();
        std::fs::write(dir.path().join("top"), b"hi").unwrap();

        let root = dir.path().to_owned();
        let mut out = Vec::new();
        let result = stacks(&mut out, walk_options(), vec![root.clone()], None).unwrap();
        assert_eq!(result.num_errors, 0);

        let folded = folded(&out);
        let base = root.to_string_lossy().replace(';', "_");
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
        stacks(&mut out, walk_options(), vec![dir.path().to_owned()], None).unwrap();
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
        stacks(&mut out, walk_options(), vec![file.clone()], None).unwrap();

        let folded = folded(&out);
        assert_eq!(folded.len(), 1);
        assert_eq!(
            folded.get(&file.to_string_lossy().replace(';', "_")),
            Some(&5)
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
            walk_options(),
            vec![dir.path().to_owned()],
            Some(1),
        )
        .unwrap();

        let folded = folded(&out);
        let nested = format!("{};nested", dir.path().to_string_lossy().replace(';', "_"));
        assert!(folded.contains_key(&nested));
        assert!(!folded.keys().any(|stack| stack.ends_with(";file")));
    }

    #[test]
    fn frame_names_are_encoded_without_collisions() {
        let encode = |name: &str| {
            frame(&EntryData {
                name: name.into(),
                ..EntryData::default()
            })
        };

        assert_eq!(encode("a;b"), r"a\x3bb");
        assert_eq!(encode("a_b"), "a_b");
        assert_eq!(encode(r"a\x3bb"), r"a\\x3bb");
        assert_eq!(encode("a\nb"), r"a\nb");
    }
}
