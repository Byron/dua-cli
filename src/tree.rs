use crate::traverse::{BackgroundTraversal, EntryData, Traversal, Tree, TreeIndex};
use crate::{ByteFormat, WalkOptions, WalkResult};
use anyhow::Result;
use owo_colors::{AnsiColors as Color, OwoColorize};
use petgraph::Direction;
use std::io;
use std::path::PathBuf;

/// Traverse `paths` and write an indented tree of their disk usage to `out`, descending up to
/// `max_depth` levels below each given root.
///
/// The given roots form the first level, so `max_depth` of 1 lists only them (the same set of
/// entries the flat aggregation prints), while higher values reveal nested directories and files.
/// When `compute_total` is set and more than one root is given, a trailing `total` line is written.
/// `sort_by_size_in_bytes` sorts the children at each level ascending by size, otherwise they are
/// left in the order they were discovered.
pub fn aggregate_tree(
    mut out: impl io::Write,
    walk_options: WalkOptions,
    byte_format: ByteFormat,
    paths: Vec<PathBuf>,
    max_depth: usize,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
) -> Result<WalkResult> {
    let mut traversal = Traversal::new();
    if paths.is_empty() {
        return Ok(WalkResult::default());
    }

    let pattern_roots = walk_options
        .ignore_patterns
        .as_ref()
        .map(|_| paths.as_slice());
    let mut background = BackgroundTraversal::start(
        traversal.root_index,
        &walk_options,
        paths.clone(),
        pattern_roots,
        false,
        true,
    )?;

    while let Ok(event) = background.event_rx.recv() {
        if background
            .integrate_traversal_event(&mut traversal, event)
            .unwrap_or(false)
        {
            break;
        }
    }

    let roots = sorted_children(&traversal.tree, traversal.root_index, sort_by_size_in_bytes);
    let mut total = 0u128;
    for root in &roots {
        total += traversal.tree[*root].size;
        write_subtree(
            &mut out,
            &traversal.tree,
            *root,
            1,
            max_depth,
            byte_format,
            sort_by_size_in_bytes,
        )?;
    }

    if roots.len() > 1 && compute_total {
        write_entry(&mut out, "total", total, false, false, 0, byte_format)?;
    }

    Ok(WalkResult {
        num_errors: background.stats.io_errors,
    })
}

/// Write `index` and, while there is depth budget left, its descendants, indented by their level.
fn write_subtree(
    out: &mut impl io::Write,
    tree: &Tree,
    index: TreeIndex,
    depth: usize,
    max_depth: usize,
    byte_format: ByteFormat,
    sort_by_size_in_bytes: bool,
) -> io::Result<()> {
    let entry: &EntryData = &tree[index];
    let name = entry.name.to_string_lossy();
    write_entry(
        out,
        &name,
        entry.size,
        entry.is_dir,
        entry.metadata_io_error,
        depth - 1,
        byte_format,
    )?;

    if depth >= max_depth {
        return Ok(());
    }
    for child in sorted_children(tree, index, sort_by_size_in_bytes) {
        write_subtree(
            out,
            tree,
            child,
            depth + 1,
            max_depth,
            byte_format,
            sort_by_size_in_bytes,
        )?;
    }
    Ok(())
}

/// Return the children of `index`, ordered by size ascending when `sort_by_size_in_bytes` is set,
/// otherwise in the order they were discovered during the traversal.
fn sorted_children(tree: &Tree, index: TreeIndex, sort_by_size_in_bytes: bool) -> Vec<TreeIndex> {
    let mut children: Vec<TreeIndex> = tree
        .neighbors_directed(index, Direction::Outgoing)
        .collect();
    // `petgraph` yields neighbors in the reverse of their insertion order, so undo that to recover
    // the discovery order the walk produced.
    children.reverse();
    if sort_by_size_in_bytes {
        children.sort_by_key(|child| tree[*child].size);
    }
    children
}

fn write_entry(
    out: &mut impl io::Write,
    name: &str,
    num_bytes: u128,
    is_dir: bool,
    metadata_io_error: bool,
    indent_level: usize,
    byte_format: ByteFormat,
) -> io::Result<()> {
    let size = byte_format.display(num_bytes).to_string();
    let size = size.green();
    let size_width = byte_format.width();
    let indent = "  ".repeat(indent_level);
    let error = if metadata_io_error {
        "  <IO Error>"
    } else {
        ""
    };

    if is_dir {
        writeln!(
            out,
            "{size:>size_width$} {indent}{}{error}",
            name.color(Color::Cyan)
        )
    } else {
        writeln!(out, "{size:>size_width$} {indent}{name}{error}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk_options() -> WalkOptions {
        WalkOptions {
            threads: 1,
            count_hard_links: true,
            apparent_size: true,
            cross_filesystems: true,
            ignore_dirs: std::collections::BTreeSet::default(),
            ignore_patterns: None,
            metadata_options: crate::TraversalOptions::default(),
        }
    }

    /// Drop the color escape sequences so tests can assert on the plain text and its indentation.
    fn strip_ansi(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let mut chars = line.chars();
        while let Some(c) = chars.next() {
            if c == '\u{1b}' {
                for escaped in chars.by_ref() {
                    if escaped == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    fn lines(out: &[u8]) -> Vec<String> {
        String::from_utf8(out.to_vec())
            .unwrap()
            .lines()
            .map(strip_ansi)
            .collect()
    }

    #[test]
    fn depth_limits_how_far_the_tree_descends() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested/deep"), b"1234567890").unwrap();

        let mut shallow = Vec::new();
        aggregate_tree(
            &mut shallow,
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().to_owned()],
            1,
            true,
            true,
        )
        .unwrap();
        let shallow = lines(&shallow);
        assert_eq!(
            shallow.len(),
            1,
            "a depth of one prints only the given root: {shallow:?}"
        );
        assert!(shallow[0].contains(&dir.path().to_string_lossy().into_owned()));

        let mut deep = Vec::new();
        aggregate_tree(
            &mut deep,
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().to_owned()],
            3,
            true,
            true,
        )
        .unwrap();
        let deep = lines(&deep);
        assert!(
            deep.iter().any(|line| line.contains("nested")),
            "the nested directory shows up once we go deeper: {deep:?}"
        );
        assert!(
            deep.iter().any(|line| line.contains("deep")),
            "so does the file inside it: {deep:?}"
        );
        assert!(
            deep.iter().any(|line| line.contains("  nested")),
            "children are indented below their parent: {deep:?}"
        );
    }

    #[test]
    fn children_are_sorted_by_size_ascending_by_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("small"), b"1").unwrap();
        std::fs::write(dir.path().join("large"), vec![0u8; 4096]).unwrap();

        let mut out = Vec::new();
        aggregate_tree(
            &mut out,
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().to_owned()],
            2,
            true,
            true,
        )
        .unwrap();
        let out = String::from_utf8(out).unwrap();
        let small = out.find("small").expect("small file is listed");
        let large = out.find("large").expect("large file is listed");
        assert!(small < large, "the smaller child is printed first: {out:?}");
    }

    #[test]
    fn multiple_roots_get_a_total() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a"), b"aa").unwrap();
        std::fs::write(dir.path().join("b"), b"bbbb").unwrap();

        let mut with_total = Vec::new();
        aggregate_tree(
            &mut with_total,
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().join("a"), dir.path().join("b")],
            1,
            true,
            false,
        )
        .unwrap();
        assert!(
            String::from_utf8(with_total).unwrap().contains("total"),
            "several roots are summed up"
        );

        let mut without_total = Vec::new();
        aggregate_tree(
            &mut without_total,
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().join("a"), dir.path().join("b")],
            1,
            false,
            false,
        )
        .unwrap();
        assert!(
            !String::from_utf8(without_total).unwrap().contains("total"),
            "no total line when it is turned off"
        );
    }
}
