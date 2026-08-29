use dua::{
    ByteFormat, diff_snapshots,
    snapshot::Replay,
    traverse::{EntryData, Traversal, TreeIndex},
};
use std::{io::Cursor as IoCursor, path::Path};

fn add(
    traversal: &mut Traversal,
    parent: TreeIndex,
    name: impl AsRef<Path>,
    size: u128,
    is_dir: bool,
) -> TreeIndex {
    let node = traversal.tree.add_node(EntryData {
        name: name.as_ref().into(),
        size,
        entry_count: is_dir.then_some(0),
        is_dir,
        ..EntryData::default()
    });
    traversal.tree.add_edge(parent, node, ());
    node
}

fn replay(traversal: &Traversal, root: TreeIndex) -> Replay<IoCursor<Vec<u8>>> {
    replay_roots(traversal, &[root])
}

fn replay_roots(traversal: &Traversal, roots: &[TreeIndex]) -> Replay<IoCursor<Vec<u8>>> {
    let mut bytes = Vec::new();
    dua::snapshot::write(&mut bytes, traversal, roots, None).unwrap();
    Replay::new(IoCursor::new(bytes)).unwrap()
}

fn normalized_paths(output: String) -> String {
    output.replace(std::path::MAIN_SEPARATOR, "/")
}

fn fixture(before: bool) -> Replay<IoCursor<Vec<u8>>> {
    let mut traversal = Traversal::new();
    let synthetic = traversal.root_index;
    let root = add(
        &mut traversal,
        synthetic,
        "root",
        if before { 13 } else { 19 },
        true,
    );
    if before {
        add(&mut traversal, root, "gone", 3, false);
        add(&mut traversal, root, "grown", 2, false);
        let old_dir = add(&mut traversal, root, "old-dir", 7, true);
        add(&mut traversal, old_dir, "descendant", 7, false);
    } else {
        add(&mut traversal, root, "added", 4, false);
        add(&mut traversal, root, "grown", 5, false);
        let new_dir = add(&mut traversal, root, "new-dir", 9, true);
        add(&mut traversal, new_dir, "descendant", 9, false);
    }
    add(&mut traversal, root, "same", 1, false);
    replay(&traversal, root)
}

fn diff(
    old: Replay<IoCursor<Vec<u8>>>,
    new: Replay<IoCursor<Vec<u8>>>,
    directories_only: bool,
) -> String {
    diff_at_depth(old, new, directories_only, None, 5)
}

fn diff_at_depth(
    mut old: Replay<IoCursor<Vec<u8>>>,
    mut new: Replay<IoCursor<Vec<u8>>>,
    directories_only: bool,
    max_depth: Option<usize>,
    summary_limit: usize,
) -> String {
    let mut out = Vec::new();
    diff_snapshots(
        (&mut out, false),
        &mut old,
        &mut new,
        ByteFormat::Bytes,
        directories_only,
        None,
        max_depth,
        summary_limit,
    )
    .unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn reports_a_collapsed_streaming_diff() {
    insta::assert_snapshot!(
        normalized_paths(diff(fixture(true), fixture(false), false)),
        "collapsed changes stream before the largest-change summary",
        @r"
    root/
      + 4 b added
      - 3 b gone
      ~ +3 b grown
      + 9 b new-dir/
      - 7 b old-dir/

    Largest removals (showing 2 of 2):
    - 7 b root/old-dir/
    - 3 b root/gone

    Largest additions (showing 2 of 2):
    + 9 b root/new-dir/
    + 4 b root/added

    Changes: 5
    "
    );
}

#[test]
fn handles_type_changes_duplicate_names_and_u128_deltas() {
    let mut old = Traversal::new();
    let old_synthetic = old.root_index;
    let old_root = add(&mut old, old_synthetic, "root", 0, true);
    let old_swap = add(&mut old, old_root, "swap", 5, true);
    add(&mut old, old_swap, "hidden", 5, false);
    add(&mut old, old_root, "same", 0, true);
    let old_second = add(&mut old, old_root, "same", 0, true);
    add(&mut old, old_second, "child", 1, false);
    add(&mut old, old_root, "huge", 0, false);

    let mut new = Traversal::new();
    let new_synthetic = new.root_index;
    let new_root = add(&mut new, new_synthetic, "root", 0, true);
    add(&mut new, new_root, "swap", 7, false);
    let new_first = add(&mut new, new_root, "same", 0, true);
    add(&mut new, new_first, "child", 2, false);
    add(&mut new, new_root, "same", 0, true);
    add(&mut new, new_root, "huge", u128::MAX, false);

    insta::assert_snapshot!(
        normalized_paths(diff(
            replay(&old, old_root),
            replay(&new, new_root),
            false
        )),
        "type changes, duplicate names and a maximum u128 delta",
        @r"
    root/
      ~ +340282366920938463463374607431768211455 b huge
      same/
        + 2 b child
      same/
        - 1 b child
      - 5 b swap/
      + 7 b swap

    Largest removals (showing 2 of 2):
    - 5 b root/swap/
    - 1 b root/same/child

    Largest additions (showing 2 of 2):
    + 7 b root/swap
    + 2 b root/same/child

    Changes: 5
    "
    );
    insta::assert_snapshot!(
        normalized_paths(diff(
            replay(&old, old_root),
            replay(&new, new_root),
            true
        )),
        "directory-only type changes",
        @r"
    root/
      - 5 b swap/

    Largest removals (showing 1 of 1):
    - 5 b root/swap/

    Changes: 1
    "
    );
}

#[test]
fn identical_snapshots_have_no_diff() {
    insta::assert_snapshot!(
        diff(fixture(true), fixture(true), false),
        "identical snapshots produce no report",
        @""
    );
}

#[test]
fn control_characters_cannot_split_output_lines() {
    let mut old = Traversal::new();
    let old_synthetic = old.root_index;
    let old_root = add(&mut old, old_synthetic, "root", 0, true);

    let mut new = Traversal::new();
    let new_synthetic = new.root_index;
    let new_root = add(&mut new, new_synthetic, "root", 1, true);
    add(&mut new, new_root, "line\nbreak", 1, false);

    insta::assert_snapshot!(
        normalized_paths(diff(
            replay(&old, old_root),
            replay(&new, new_root),
            false
        )),
        "control characters cannot create output lines",
        @r"
    root/
      + 1 b line�break

    Largest additions (showing 1 of 1):
    + 1 b root/line�break

    Changes: 1
    "
    );
}

#[test]
fn summary_limit_controls_largest_additions_and_removals() {
    let mut old = Traversal::new();
    let old_synthetic = old.root_index;
    let old_root = add(&mut old, old_synthetic, "root", 0, true);
    for size in 1..=6 {
        add(&mut old, old_root, format!("old-{size}"), size, false);
    }

    let mut new = Traversal::new();
    let new_synthetic = new.root_index;
    let new_root = add(&mut new, new_synthetic, "root", 0, true);
    for size in 11..=16 {
        add(&mut new, new_root, format!("new-{size}"), size, false);
    }

    insta::assert_snapshot!(
        normalized_paths(diff_at_depth(
            replay(&old, old_root),
            replay(&new, new_root),
            false,
            None,
            2
        )),
        "summary retains only the configured number of additions and removals",
        @r"
    root/
      + 11 b new-11
      + 12 b new-12
      + 13 b new-13
      + 14 b new-14
      + 15 b new-15
      + 16 b new-16
      - 1 b old-1
      - 2 b old-2
      - 3 b old-3
      - 4 b old-4
      - 5 b old-5
      - 6 b old-6

    Largest removals (showing 2 of 6):
    - 6 b root/old-6
    - 5 b root/old-5

    Largest additions (showing 2 of 6):
    + 16 b root/new-16
    + 15 b root/new-15

    Changes: 12
    "
    );
    insta::assert_snapshot!(
        normalized_paths(diff_at_depth(
            replay(&old, old_root),
            replay(&new, new_root),
            false,
            None,
            0
        )),
        "a zero summary limit hides largest additions and removals",
        @r"
    root/
      + 11 b new-11
      + 12 b new-12
      + 13 b new-13
      + 14 b new-14
      + 15 b new-15
      + 16 b new-16
      - 1 b old-1
      - 2 b old-2
      - 3 b old-3
      - 4 b old-4
      - 5 b old-5
      - 6 b old-6

    Changes: 12
    "
    );
}

#[test]
fn depth_keeps_context_and_does_not_limit_the_summary() {
    let mut old = Traversal::new();
    let old_synthetic = old.root_index;
    let old_root = add(&mut old, old_synthetic, "root", 0, true);
    add(&mut old, old_root, "dir", 0, true);

    let mut new = Traversal::new();
    let new_synthetic = new.root_index;
    let new_root = add(&mut new, new_synthetic, "root", 0, true);
    let new_dir = add(&mut new, new_root, "dir", 0, true);
    add(&mut new, new_dir, "added", 1, false);

    insta::assert_snapshot!(
        normalized_paths(diff_at_depth(
            replay(&old, old_root),
            replay(&new, new_root),
            false,
            Some(0),
            5
        )),
        "depth zero collapses the tree but not its summary",
        @r"
    root/ …

    Largest additions (showing 1 of 1):
    + 1 b root/dir/added

    Changes: 1
    "
    );
    insta::assert_snapshot!(
        normalized_paths(diff_at_depth(
            replay(&old, old_root),
            replay(&new, new_root),
            false,
            Some(1),
            5
        )),
        "depth one collapses descendants but not the summary",
        @r"
    root/
      dir/ …

    Largest additions (showing 1 of 1):
    + 1 b root/dir/added

    Changes: 1
    "
    );
    insta::assert_snapshot!(
        normalized_paths(diff_at_depth(
            fixture(true),
            fixture(false),
            true,
            Some(0),
            5
        )),
        "directory changes collapse at depth zero",
        @r"
    ~ +6 b root/ …

    Largest removals (showing 1 of 1):
    - 7 b root/old-dir/

    Largest additions (showing 1 of 1):
    + 9 b root/new-dir/

    Changes: 3
    "
    );

    let mut old_replay = replay(&old, old_root);
    let mut new_replay = replay(&new, new_root);
    let mut output = Vec::new();
    let prefix = Path::new("root").join("dir");
    diff_snapshots(
        (&mut output, false),
        &mut old_replay,
        &mut new_replay,
        ByteFormat::Bytes,
        false,
        Some(&prefix),
        Some(0),
        5,
    )
    .unwrap();
    insta::assert_snapshot!(
        normalized_paths(String::from_utf8(output).unwrap()),
        "the selected prefix is depth zero",
        @r"
    root/dir/ …

    Largest additions (showing 1 of 1):
    + 1 b root/dir/added

    Changes: 1
    "
    );
}

#[test]
fn tree_resets_between_roots() {
    let first = Path::new("first").join("root");
    let second = Path::new("second").join("root");
    let mut old = Traversal::new();
    let old_synthetic = old.root_index;
    let old_first = add(&mut old, old_synthetic, &first, 0, true);
    add(&mut old, old_first, "changed", 1, false);
    let old_second = add(&mut old, old_synthetic, &second, 0, true);
    add(&mut old, old_second, "changed", 2, false);

    let mut new = Traversal::new();
    let new_synthetic = new.root_index;
    let new_first = add(&mut new, new_synthetic, first, 0, true);
    add(&mut new, new_first, "changed", 2, false);
    let new_second = add(&mut new, new_synthetic, second, 0, true);
    add(&mut new, new_second, "changed", 3, false);

    insta::assert_snapshot!(
        normalized_paths(diff(
            replay_roots(&old, &[old_first, old_second]),
            replay_roots(&new, &[new_first, new_second]),
            false,
        )),
        "tree context resets between stored roots",
        @r"
    first/root/
      ~ +1 b changed
    second/root/
      ~ +1 b changed

    Changes: 2
    "
    );
}

#[test]
fn terminal_output_colors_changes_and_context() {
    let mut old = fixture(true);
    let mut new = fixture(false);
    let mut output = Vec::new();
    diff_snapshots(
        (&mut output, true),
        &mut old,
        &mut new,
        ByteFormat::Bytes,
        false,
        None,
        None,
        5,
    )
    .unwrap();
    insta::assert_snapshot!(
        normalized_paths(String::from_utf8(output).unwrap().replace('\u{1b}', "<ESC>")),
        "terminal colors distinguish changes and directory context",
        @r"
    <ESC>[36mroot/<ESC>[39m
    <ESC>[32m  + 4 b added<ESC>[39m
    <ESC>[31m  - 3 b gone<ESC>[39m
    <ESC>[33m  ~ +3 b grown<ESC>[39m
    <ESC>[32m  + 9 b new-dir/<ESC>[39m
    <ESC>[31m  - 7 b old-dir/<ESC>[39m

    Largest removals (showing 2 of 2):
    <ESC>[31m- 7 b root/old-dir/<ESC>[39m
    <ESC>[31m- 3 b root/gone<ESC>[39m

    Largest additions (showing 2 of 2):
    <ESC>[32m+ 9 b root/new-dir/<ESC>[39m
    <ESC>[32m+ 4 b root/added<ESC>[39m

    Changes: 5
    "
    );
}

#[test]
fn directories_only_reports_aggregate_directory_changes() {
    insta::assert_snapshot!(
        normalized_paths(diff(fixture(true), fixture(false), true)),
        "directory-only aggregate changes",
        @r"
    ~ +6 b root/
      + 9 b new-dir/
      - 7 b old-dir/

    Largest removals (showing 1 of 1):
    - 7 b root/old-dir/

    Largest additions (showing 1 of 1):
    + 9 b root/new-dir/

    Changes: 3
    "
    );
}

#[test]
fn prefix_is_component_aware_and_descends_through_excluded_ancestors() {
    let mut old = Traversal::new();
    let old_synthetic = old.root_index;
    let old_root = add(&mut old, old_synthetic, "root", 0, true);
    add(&mut old, old_root, "foo", 1, false);
    add(&mut old, old_root, "foobar", 2, false);

    let mut new = Traversal::new();
    let new_synthetic = new.root_index;
    let new_root = add(&mut new, new_synthetic, "root", 0, true);
    add(&mut new, new_root, "foo", 3, false);
    add(&mut new, new_root, "foobar", 4, false);

    let mut old_replay = replay(&old, old_root);
    let mut new_replay = replay(&new, new_root);
    let mut out = Vec::new();
    let prefix = Path::new("root").join("foo");
    diff_snapshots(
        (&mut out, false),
        &mut old_replay,
        &mut new_replay,
        ByteFormat::Bytes,
        false,
        Some(&prefix),
        None,
        5,
    )
    .unwrap();
    insta::assert_snapshot!(
        normalized_paths(String::from_utf8(out).unwrap()),
        "prefix selection is component-aware",
        @r"
    ~ +2 b root/foo

    Changes: 1
    "
    );

    let mut old_replay = replay(&old, old_root);
    let mut new_replay = replay(&new, new_root);
    let mut out = Vec::new();
    let prefix = Path::new("root").join("missing");
    diff_snapshots(
        (&mut out, false),
        &mut old_replay,
        &mut new_replay,
        ByteFormat::Bytes,
        false,
        Some(&prefix),
        None,
        5,
    )
    .unwrap();
    insta::assert_snapshot!(
        String::from_utf8(out).unwrap(),
        "an unmatched prefix produces no report",
        @""
    );

    let mut old = fixture(true);
    let mut new = fixture(false);
    let mut out = Vec::new();
    let prefix = Path::new("root").join("new-dir").join("descendant");
    diff_snapshots(
        (&mut out, false),
        &mut old,
        &mut new,
        ByteFormat::Bytes,
        false,
        Some(&prefix),
        None,
        5,
    )
    .unwrap();
    insta::assert_snapshot!(
        normalized_paths(String::from_utf8(out).unwrap()),
        "prefix selection descends through excluded ancestors",
        @r"
    + 9 b root/new-dir/descendant

    Largest additions (showing 1 of 1):
    + 9 b root/new-dir/descendant

    Changes: 1
    "
    );
}
