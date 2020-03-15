use crate::interactive::app_test::utils::{
    initialized_app_and_terminal_from_paths, new_test_terminal, WritableFixture,
};
use failure::Error;
use pretty_assertions::assert_eq;
use termion::event::Key;
use termion::input::TermRead;

#[test]
fn basic_user_journey_with_deletion() -> Result<(), Error> {
    let fixture = WritableFixture::from("sample-02");
    let (terminal, mut app) = initialized_app_and_terminal_from_paths(&[fixture.root.clone()])?;

    // With a selection of items
    app.process_events(terminal, b"doddd".keys())?;

    assert_eq!(
        app.window.mark_pane.as_ref().map(|p| p.marked().len()),
        Some(4),
        "expecting 4 selected entries, the parent dir, and some children"
    );

    assert_eq!(
        fixture.as_ref().is_dir(),
        true,
        "expecting fixture root to exist"
    );

    // When selecting the marker window and pressing the combination to delete entries
    app.process_events(
        new_test_terminal()?,
        vec![Ok(Key::Char('\t')), Ok(Key::Ctrl('r'))].into_iter(),
    )?;
    assert_eq!(
        app.window.mark_pane.is_none(),
        true,
        "the marker pane is gone as all entries have been removed"
    );
    assert_eq!(app.state.selected, None, "nothing is left to be selected");
    assert_eq!(
        app.state.root, app.traversal.root_index,
        "the only root left is the top-level"
    );
    assert_eq!(
        fixture.as_ref().is_dir(),
        false,
        "the directory should have been deleted",
    );
    Ok(())
}
