use crate::interactive::app::tests::utils::{
    WritableFixture, initialized_app_and_terminal_from_paths, into_codes, into_events,
    new_test_terminal,
};
use crate::interactive::terminal::TerminalApp;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use dua::{ByteFormat, Config, TraversalSorting, WalkOptions};
use pretty_assertions::assert_eq;
use std::{collections::BTreeSet, fs};
use tempfile::TempDir;

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

#[test]
#[cfg(not(target_os = "windows"))]
fn cleanup_candidates_are_marked_with_one_key_after_entering_project_dir() -> Result<()> {
    let fixture = TempDir::new()?;
    let root = fixture.path();
    fs::create_dir_all(root.join("target/debug"))?;
    fs::write(root.join("target/debug/app"), [])?;
    fs::create_dir_all(root.join("node_modules/package"))?;
    fs::write(root.join("node_modules/package/index.js"), [])?;
    fs::create_dir_all(root.join("__pycache__"))?;
    fs::write(root.join("__pycache__/module.pyc"), [])?;
    fs::create_dir_all(root.join("build"))?;
    fs::write(root.join("build/release-artifact"), [])?;

    let mut terminal = new_test_terminal()?;
    let walk_options = WalkOptions {
        threads: 1,
        apparent_size: true,
        count_hard_links: false,
        sorting: TraversalSorting::AlphabeticalByFileName,
        cross_filesystems: false,
        ignore_dirs: Default::default(),
    };
    let (_key_send, key_receive) = crossbeam::channel::bounded(0);
    let mut app = TerminalApp::initialize(
        &mut terminal,
        walk_options,
        ByteFormat::Metric,
        true,
        vec![root.to_owned()],
        Config::default(),
    )?;
    app.traverse()?;
    app.run_until_traversed(&mut terminal, key_receive)?;

    app.process_events(&mut terminal, into_codes("o"))?;

    assert_eq!(app.state.cleanup_candidates.len(), 3);

    app.process_events(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Char('/').into()),
            Event::Key(KeyCode::Char('t').into()),
            Event::Key(KeyCode::Char('a').into()),
            Event::Key(KeyCode::Char('r').into()),
            Event::Key(KeyCode::Char('g').into()),
            Event::Key(KeyCode::Char('e').into()),
            Event::Key(KeyCode::Char('t').into()),
            Event::Key(KeyCode::Enter.into()),
        ]),
    )?;
    assert!(
        app.state.cleanup_candidates.is_empty(),
        "glob views should not offer cleanup candidates"
    );

    app.process_events(
        &mut terminal,
        into_events([Event::Key(KeyCode::Char('q').into())]),
    )?;
    assert_eq!(app.state.cleanup_candidates.len(), 3);

    app.process_events(&mut terminal, into_codes("X"))?;
    app.process_events(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Tab.into()),
            Event::Key(KeyCode::Char('a').into()),
        ]),
    )?;
    app.process_events(&mut terminal, into_codes("X"))?;

    let marked_names = app
        .window
        .mark_pane
        .as_ref()
        .expect("cleanup candidates are marked")
        .marked()
        .values()
        .map(|entry| {
            entry
                .path
                .file_name()
                .expect("marked path has a final component")
                .to_string_lossy()
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        marked_names,
        BTreeSet::from([
            "__pycache__".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
        ])
    );
    Ok(())
}
