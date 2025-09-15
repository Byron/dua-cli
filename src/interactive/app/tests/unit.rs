use crate::interactive::app::tests::utils::{
    debug, initialized_app_and_terminal_from_fixture, sample_01_tree, sample_02_tree,
};
use crate::interactive::widgets::glob_search;
use anyhow::Result;
use dua::traverse::TreeIndex;
use gix_glob::pattern::Case;
use pretty_assertions::assert_eq;

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
