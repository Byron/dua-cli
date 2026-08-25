use crate::aggregate::output_colored_path;
use crate::traverse::{BackgroundTraversal, EntryData, Traversal, Tree, TreeIndex};
use crate::{ByteFormat, WalkOptions, WalkResult};
use anyhow::{Context, Result};
use owo_colors::AnsiColors as Color;
use petgraph::Direction;
use std::io;
use std::path::PathBuf;

/// Traverse `paths` and write an indented tree of their disk usage to `out`, descending up to
/// `max_depth` levels below each given root.
///
/// The given roots are at depth `0`, so `max_depth` of `0` lists only them (the same set of entries the
/// flat aggregation prints), `1` also lists their children, and higher values reveal deeper entries.
/// When `compute_total` is set and more than one root is given, a trailing `total` line is written.
/// `sort_by_size_in_bytes` sorts the children at each level ascending by size, otherwise they are
/// left in the order they were discovered.
pub fn aggregate_tree(
    out: (impl io::Write, bool),
    walk_options: WalkOptions,
    byte_format: ByteFormat,
    paths: Vec<PathBuf>,
    max_depth: usize,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
) -> Result<WalkResult> {
    let (mut out, out_supports_colors) = out;
    let output_options = (byte_format, out_supports_colors);
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
    )?
    .retain_depth(Some(max_depth));

    while let Ok(event) = background.event_rx.recv() {
        if background
            .integrate_traversal_event(&mut traversal, event)
            .unwrap_or(false)
        {
            break;
        }
    }

    let num_errors = background.stats.io_errors;
    let mut roots = background
        .root_nodes
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .context("traversal did not produce a node for every root")?;
    if sort_by_size_in_bytes {
        roots.sort_by_key(|root| traversal.tree[*root].size);
    }
    let mut total = 0u128;
    for root in &roots {
        total += traversal.tree[*root].size;
        write_subtree(
            &mut out,
            &traversal.tree,
            *root,
            0,
            max_depth,
            sort_by_size_in_bytes,
            output_options,
        )?;
    }

    if roots.len() > 1 && compute_total {
        write_entry(
            &mut out,
            "total",
            total,
            false,
            num_errors,
            0,
            output_options,
        )?;
    }

    Ok(WalkResult { num_errors })
}

/// Write `index` and, while there is depth budget left, its descendants, indented by their level.
fn write_subtree(
    out: &mut impl io::Write,
    tree: &Tree,
    index: TreeIndex,
    depth: usize,
    max_depth: usize,
    sort_by_size_in_bytes: bool,
    output_options: (ByteFormat, bool),
) -> io::Result<()> {
    let entry: &EntryData = &tree[index];
    let name = entry.name.to_string_lossy();
    write_entry(
        out,
        &name,
        entry.size,
        entry.is_dir,
        u64::from(entry.metadata_io_error),
        depth,
        output_options,
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
            sort_by_size_in_bytes,
            output_options,
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
    num_errors: u64,
    indent_level: usize,
    (byte_format, out_supports_colors): (ByteFormat, bool),
) -> io::Result<()> {
    output_colored_path(
        out,
        out_supports_colors,
        format!("{}{name}", "  ".repeat(indent_level)),
        num_bytes,
        num_errors,
        is_dir.then_some(Color::Cyan),
        byte_format,
    )
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

    fn lines(out: &[u8]) -> Vec<String> {
        std::str::from_utf8(out)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn depth_limits_how_far_the_tree_descends() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested/deep"), b"1234567890").unwrap();

        let mut shallow = Vec::new();
        aggregate_tree(
            (&mut shallow, false),
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().to_owned()],
            0,
            true,
            true,
        )
        .unwrap();
        let shallow = lines(&shallow);
        assert_eq!(
            shallow.len(),
            1,
            "a depth of zero prints only the given root: {shallow:?}"
        );
        assert!(shallow[0].contains(&dir.path().to_string_lossy().into_owned()));

        let mut deep = Vec::new();
        aggregate_tree(
            (&mut deep, false),
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().to_owned()],
            2,
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
            (&mut out, false),
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().to_owned()],
            1,
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
            (&mut with_total, false),
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().join("a"), dir.path().join("b")],
            0,
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
            (&mut without_total, false),
            walk_options(),
            ByteFormat::Bytes,
            vec![dir.path().join("a"), dir.path().join("b")],
            0,
            false,
            false,
        )
        .unwrap();
        assert!(
            !String::from_utf8(without_total).unwrap().contains("total"),
            "no total line when it is turned off"
        );
    }

    #[test]
    fn failed_roots_are_printed_in_input_order() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");
        let valid = dir.path().join("valid");
        std::fs::write(&valid, b"content").unwrap();

        let mut out = Vec::new();
        let result = aggregate_tree(
            (&mut out, false),
            walk_options(),
            ByteFormat::Bytes,
            vec![missing.clone(), valid.clone()],
            0,
            true,
            false,
        )
        .unwrap();
        let out = lines(&out);

        assert_eq!(result.num_errors, 1);
        assert!(out[0].contains(&missing.to_string_lossy().into_owned()));
        assert!(out[0].contains("<1 IO Error>"));
        assert!(out[1].contains(&valid.to_string_lossy().into_owned()));
        assert!(out[2].contains("total  <1 IO Error>"));
    }
}
