use crate::interactive::app::tests::utils::{
    debug, initialized_app_and_terminal_from_fixture, sample_01_tree, sample_02_tree,
};
use crate::interactive::widgets::glob_search;
use anyhow::Result;
use dua::traverse::TreeIndex;
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
    let (expected_tree, _) = sample_02_tree();

    assert_eq!(
        debug(app.traversal.tree),
        debug(expected_tree),
        "filesystem graph is stable and matches the directory structure"
    );
    Ok(())
}

#[test]
fn glob_pattern() {
    let (tree, root_index) = sample_02_tree();
    let result = glob_search(&tree, root_index, "tests/fixtures/sample-02").unwrap();
    let expected = vec![TreeIndex::from(1)];
    assert_eq!(result, expected);
}
