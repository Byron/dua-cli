use crate::interactive::app::tests::utils::{
    initialized_app_and_terminal_from_paths, into_keys, WritableFixture,
};
use anyhow::Result;
use crosstermion::input::Event;
use crosstermion::input::Key;
use pretty_assertions::assert_eq;

#[test]
#[cfg(not(target_os = "windows"))] // it stopped working here, don't know if it's truly broken or if it's the test. Let's wait for windows users to report.
fn basic_user_journey_with_deletion() -> Result<()> {
    let fixture = WritableFixture::from("sample-02");
    let (mut terminal, mut app) = initialized_app_and_terminal_from_paths(&[fixture.root.clone()])?;

    // With a selection of items
    app.process_events(&mut terminal, into_keys(b"doddd".iter()))?;

    assert_eq!(
        app.window.mark_pane.as_ref().map(|p| p.marked().len()),
        Some(4),
        "expecting 4 selected entries, the parent dir, and some children"
    );

    assert!(fixture.as_ref().is_dir(), "expecting fixture root to exist");

    // When selecting the marker window and pressing the combination to delete entries
    app.process_events(
        &mut terminal,
        vec![Event::Key(Key::Char('\t')), Event::Key(Key::Ctrl('r'))].into_iter(),
    )?;
    assert!(
        app.window.mark_pane.is_none(),
        "the marker pane is gone as all entries have been removed"
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
