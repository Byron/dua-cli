use crate::interactive::app_test::utils::{
    adapt, initialized_app_and_terminal_from_paths, WritableFixture,
};
use anyhow::Result;
use pretty_assertions::assert_eq;
use std::convert::TryInto;
use termion::{event::Key, input::TermRead};

#[test]
fn basic_user_journey_with_deletion() -> Result<()> {
    let fixture = WritableFixture::from("sample-02");
    let (mut terminal, mut app) = initialized_app_and_terminal_from_paths(&[fixture.root.clone()])?;

    // With a selection of items
    app.process_events(&mut terminal, adapt(b"doddd".keys()))?;

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
        &mut terminal,
        vec![
            Key::Char('\t').try_into().unwrap(),
            Key::Ctrl('r').try_into().unwrap(),
        ]
        .into_iter(),
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
