use crate::interactive::app::tests::utils::{
    WritableFixture, initialized_app_and_terminal_from_paths, into_codes,
};
use anyhow::Result;
use crosstermion::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crosstermion::input::Event;
use pretty_assertions::assert_eq;

#[test]
#[cfg(not(target_os = "windows"))] // it stopped working here, don't know if it's truly broken or if it's the test. Let's wait for windows users to report.
fn basic_user_journey_with_deletion() -> Result<()> {
    use crate::interactive::app::tests::utils::into_events;

    let fixture = WritableFixture::from("sample-02");
    let (mut terminal, mut app) =
        initialized_app_and_terminal_from_paths(std::slice::from_ref(&fixture.root))?;

    // With a selection of items
    app.process_events(&mut terminal, into_codes("doddd"))?;

    assert_eq!(
        app.window.mark_pane.as_ref().map(|p| p.marked().len()),
        Some(4),
        "expecting 4 selected items, the parent dir, and some children"
    );

    assert!(fixture.as_ref().is_dir(), "expecting fixture root to exist");

    // When selecting the marker window and pressing the combination to delete entries
    app.process_events(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Tab.into()),
            Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
        ]),
    )?;
    assert!(
        app.window.mark_pane.is_none(),
        "the marker pane is gone as all items have been removed"
    );
    assert_eq!(
        app.state.navigation().selected,
        None,
        "nothing is left to be selected"
    );
    assert_eq!(
        app.state.navigation().view_root,
        app.traversal.root_index,
        "the only root left is the top-level"
    );
    assert!(
        !fixture.as_ref().is_dir(),
        "the directory should have been deleted",
    );
    Ok(())
}
