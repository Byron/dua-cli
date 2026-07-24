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

fn marked_file_names(app: &TerminalApp, message: &str) -> BTreeSet<String> {
    app.window
        .mark_pane
        .as_ref()
        .expect(message)
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
        .collect()
}

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
#[cfg(unix)]
fn gitignored_entries_are_marked_with_dedicated_key() -> Result<()> {
    let fixture = TempDir::new()?;
    let root = fixture.path();
    fs::create_dir_all(root.join(".git/objects"))?;
    fs::create_dir_all(root.join(".git/refs/heads"))?;
    fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n")?;
    fs::write(
        root.join(".git/config"),
        b"[core]
	repositoryformatversion = 0
	filemode = true
	bare = false
",
    )?;
    fs::write(
        root.join(".gitignore"),
        b"ignored.log
ignored_dir/
ignored-link
target/
remove.tmp
$precious.tmp
!keep.tmp
",
    )?;
    fs::write(root.join("ignored.log"), [])?;
    fs::create_dir_all(root.join("ignored_dir"))?;
    fs::write(root.join("ignored_dir/file"), [])?;
    fs::write(root.join("remove.tmp"), [])?;
    fs::write(root.join("precious.tmp"), [])?;
    fs::write(root.join("keep.tmp"), [])?;
    std::os::unix::fs::symlink(root.join("keep.tmp"), root.join("ignored-link"))?;
    fs::create_dir_all(root.join("target/debug"))?;
    fs::write(root.join("target/debug/app"), [])?;
    fs::write(root.join("target/output.bin"), [])?;

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

    let gitignored_names = app
        .state
        .entries
        .iter()
        .filter(|entry| {
            app.state
                .gitignored_entries
                .as_ref()
                .is_some_and(|entries| entries.contains(&entry.index))
        })
        .map(|entry| entry.name.to_string_lossy().to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        gitignored_names,
        BTreeSet::from([
            "ignored.log".to_string(),
            "ignored_dir".to_string(),
            "ignored-link".to_string(),
            // "precious.tmp".to_string(), # precious file is notably absent from highlighted files
            "remove.tmp".to_string(),
            "target".to_string(),
        ])
    );
    assert_eq!(
        app.state
            .cleanup_candidates
            .as_ref()
            .map_or(0, BTreeSet::len),
        1,
        "built-in cleanup candidates stay separate"
    );
    assert_eq!(
        app.state.message.as_deref(),
        Some("1 cleanup, 5 gitignored (X|I)"),
        "footer message advertises both cleanup and gitignore shortcuts"
    );
    app.process_events(&mut terminal, into_codes("i"))?;
    assert!(
        app.state.gitignored_entries.is_none(),
        "gitignored entry detection can be disabled"
    );
    assert_eq!(
        app.state.message.as_deref(),
        Some("1 cleanup candidate (X)"),
        "footer message drops the gitignore shortcut when disabled"
    );
    app.process_events(&mut terminal, into_codes("i"))?;
    assert_eq!(
        app.state.message.as_deref(),
        Some("1 cleanup, 5 gitignored (X|I)"),
        "gitignored entry detection can be enabled again"
    );

    let target_index = app
        .state
        .entries
        .iter()
        .find(|entry| entry.name == std::path::Path::new("target"))
        .expect("target directory is visible")
        .index;
    app.state.navigation_mut().select(Some(target_index));
    app.process_events(&mut terminal, into_codes("o"))?;

    let target_gitignored_names = app
        .state
        .entries
        .iter()
        .filter(|entry| {
            app.state
                .gitignored_entries
                .as_ref()
                .is_some_and(|entries| entries.contains(&entry.index))
        })
        .map(|entry| entry.name.to_string_lossy().to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        target_gitignored_names,
        BTreeSet::from(["debug".to_string(), "output.bin".to_string()]),
        "entries inside an ignored directory are ignored as well as we use repository discovery"
    );

    app.process_events(&mut terminal, into_codes("u"))?;
    app.process_events(&mut terminal, into_codes("I"))?;

    assert_eq!(
        marked_file_names(&app, "gitignored entries are marked"),
        BTreeSet::from([
            "ignored.log".to_string(),
            "ignored_dir".to_string(),
            "ignored-link".to_string(),
            "remove.tmp".to_string(),
            "target".to_string(),
        ])
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

    assert_eq!(
        app.state
            .cleanup_candidates
            .as_ref()
            .map_or(0, BTreeSet::len),
        3
    );
    app.process_events(&mut terminal, into_codes("t"))?;
    assert!(
        app.state.cleanup_candidates.is_none(),
        "cleanup candidate detection can be disabled"
    );
    app.process_events(&mut terminal, into_codes("t"))?;
    assert_eq!(
        app.state
            .cleanup_candidates
            .as_ref()
            .map_or(0, BTreeSet::len),
        3,
        "cleanup candidate detection can be enabled again"
    );

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
        app.state
            .cleanup_candidates
            .as_ref()
            .is_some_and(BTreeSet::is_empty),
        "glob views should not offer cleanup candidates"
    );

    app.process_events(
        &mut terminal,
        into_events([Event::Key(KeyCode::Char('q').into())]),
    )?;
    assert_eq!(
        app.state
            .cleanup_candidates
            .as_ref()
            .map_or(0, BTreeSet::len),
        3
    );

    app.process_events(&mut terminal, into_codes("X"))?;
    app.process_events(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Tab.into()),
            Event::Key(KeyCode::Char('a').into()),
        ]),
    )?;
    app.process_events(&mut terminal, into_codes("X"))?;

    assert_eq!(
        marked_file_names(&app, "cleanup candidates are marked"),
        BTreeSet::from([
            "__pycache__".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
        ])
    );
    Ok(())
}
