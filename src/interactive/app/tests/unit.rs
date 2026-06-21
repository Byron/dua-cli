use crate::interactive::app::tests::utils::{
    debug, initialized_app_and_terminal_from_fixture, sample_01_tree, sample_02_tree,
};
use crate::interactive::app::{state::AppState, tree_view::TreeView};
use crate::interactive::widgets::glob_search;
use crate::interactive::{EntryCheck, MTimeSort, SortMode, sorted_entries};
use anyhow::Result;
use dua::traverse::{EntryData, Traversal, Tree, TreeIndex};
use dua::{TraversalSorting, WalkOptions};
use gix::glob::pattern::Case;
use pretty_assertions::assert_eq;
use std::path::PathBuf;
use std::time::Instant;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn it_can_handle_ending_traversal_reaching_top_but_skipping_levels() -> Result<()> {
    let (_, app) = initialized_app_and_terminal_from_fixture(&["sample-01"])?;
    let expected_tree = sample_01_tree();

    assert_eq!(
        debug(app.traversal.tree),
        debug(expected_tree),
        "filesystem graph is stable and matches the directory structure"
    );
    Ok(())
}

#[test]
fn it_can_handle_ending_traversal_without_reaching_the_top() -> Result<()> {
    let (_, app) = initialized_app_and_terminal_from_fixture(&["sample-02"])?;
    let (expected_tree, _) = sample_02_tree(true);

    assert_eq!(
        debug(app.traversal.tree),
        debug(expected_tree),
        "filesystem graph is stable and matches the directory structure"
    );
    Ok(())
}

#[test]
fn it_can_do_a_glob_search() {
    let (tree, root_index) = sample_02_tree(false);
    let result = glob_search(&tree, root_index, "tests/fixtures/sample-02", Case::Fold).unwrap();
    let expected = vec![TreeIndex::from(1)];
    assert_eq!(result, expected);
}

#[test]
fn it_can_do_a_case_sensitive_glob_search() {
    let (tree, root_index) = sample_02_tree(false);
    let result_insensitive =
        glob_search(&tree, root_index, "TESTS/FIXTURES/SAMPLE-02", Case::Fold).unwrap();
    assert_eq!(result_insensitive, vec![TreeIndex::from(1)]);

    let result_sensitive = glob_search(
        &tree,
        root_index,
        "TESTS/FIXTURES/SAMPLE-02",
        Case::Sensitive,
    )
    .unwrap();
    assert!(result_sensitive.is_empty());
}

#[test]
fn it_can_sort_directory_mtimes_by_recursive_entries() {
    fn mtime(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn add_entry(
        tree: &mut Tree,
        parent: Option<TreeIndex>,
        name: &str,
        mtime_seconds: u64,
        is_dir: bool,
    ) -> TreeIndex {
        let idx = tree.add_node(EntryData {
            name: PathBuf::from(name),
            mtime: mtime(mtime_seconds),
            is_dir,
            ..Default::default()
        });
        if let Some(parent) = parent {
            tree.add_edge(parent, idx, ());
        }
        idx
    }

    let mut tree = Tree::new();
    let root = add_entry(&mut tree, None, "", 1, true);
    let first = add_entry(&mut tree, Some(root), "first", 10, true);
    add_entry(&mut tree, Some(first), "old-child", 20, false);
    let nested = add_entry(&mut tree, Some(first), "nested", 30, true);
    add_entry(&mut tree, Some(nested), "new-grandchild", 90, false);

    let second = add_entry(&mut tree, Some(root), "second", 40, true);
    add_entry(&mut tree, Some(second), "new-child", 60, false);

    let recursive = sorted_entries(
        &tree,
        root,
        SortMode::MTimeDescending(MTimeSort::RecursiveChildrenNewest),
        None,
        EntryCheck::Disabled,
    );
    assert_eq!(
        recursive[0].name,
        PathBuf::from("first"),
        "newest descendant mtime sorts first because it contains the 90-second grandchild"
    );
    assert_eq!(
        recursive[0].mtime,
        mtime(90),
        "recursive newest mode uses the newest descendant mtime for the first entry"
    );
    assert_eq!(
        recursive[1].name,
        PathBuf::from("second"),
        "the second directory sorts after first because its newest child is only 60 seconds"
    );
    assert_eq!(
        recursive[1].mtime,
        mtime(60),
        "recursive newest mode uses the second directory's newest child mtime"
    );

    let recursive_oldest = sorted_entries(
        &tree,
        root,
        SortMode::MTimeDescending(MTimeSort::RecursiveChildrenOldest),
        None,
        EntryCheck::Disabled,
    );
    assert_eq!(
        recursive_oldest[0].name,
        PathBuf::from("second"),
        "descending oldest-descendant sort puts second first because its oldest child is 60 seconds"
    );
    assert_eq!(
        recursive_oldest[0].mtime,
        mtime(60),
        "recursive oldest mode uses the oldest descendant mtime for the first sorted entry"
    );
    assert_eq!(
        recursive_oldest[1].name,
        PathBuf::from("first"),
        "first sorts second because its oldest descendant is only 20 seconds"
    );
    assert_eq!(
        recursive_oldest[1].mtime,
        mtime(20),
        "recursive oldest mode uses the first directory's oldest descendant mtime"
    );

    let mut traversal = Traversal {
        tree,
        root_index: root,
        start_time: Instant::now(),
        cost: None,
    };
    let mut state = AppState::new(
        WalkOptions {
            threads: 1,
            apparent_size: true,
            count_hard_links: false,
            sorting: TraversalSorting::AlphabeticalByFileName,
            cross_filesystems: false,
            ignore_dirs: Default::default(),
        },
        Vec::new(),
    );
    state.navigation.view_root = root;
    state.sorting = SortMode::MTimeDescending(MTimeSort::Entry);
    state.entries = sorted_entries(
        &traversal.tree,
        root,
        state.sorting,
        None,
        EntryCheck::Disabled,
    );

    let tree_view = TreeView {
        traversal: &mut traversal,
        glob_tree_root: None,
    };
    state.cycle_mtime_sort_mode(&tree_view);
    assert_eq!(
        state.sorting,
        SortMode::MTimeDescending(MTimeSort::RecursiveChildrenNewest),
        "cycling from entry mode while mtime sorting selects recursive newest without changing direction"
    );
    assert_eq!(
        state.entries[0].name,
        PathBuf::from("first"),
        "entries are recomputed immediately after changing the active mtime sort mode"
    );
    assert_eq!(
        state.entries[0].mtime,
        mtime(90),
        "recomputed entries expose the recursive newest mtime value"
    );

    state.cycle_mtime_sort_mode(&tree_view);
    assert_eq!(
        state.sorting,
        SortMode::MTimeDescending(MTimeSort::RecursiveChildrenOldest),
        "cycling again selects recursive oldest without changing direction"
    );
    assert_eq!(
        state.entries[0].name,
        PathBuf::from("second"),
        "entries are resorted by oldest descendant mtime after the mode change"
    );
    assert_eq!(
        state.entries[0].mtime,
        mtime(60),
        "recomputed entries expose the recursive oldest mtime value"
    );
}
