use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use pretty_assertions::assert_eq;
use std::{ffi::OsString, fs, time::Duration};

use crate::interactive::app::tests::utils::{into_codes, into_events};
use crate::interactive::widgets::Column;
use crate::interactive::{
    MTimeSort, SortMode,
    app::tests::{
        FIXTURE_PATH,
        utils::{
            fixture, fixture_str, index_by_name, initialized_app_and_terminal_from_fixture,
            initialized_app_and_terminal_from_paths, into_keys, new_test_terminal, node_by_index,
            node_by_name, untraversed_app_and_terminal_from_fixture,
            untraversed_app_and_terminal_with_closure,
        },
    },
};

#[test]
fn init_from_pdu_results() -> Result<()> {
    use crate::interactive::app::tests::utils::new_test_terminal;
    let _terminal = new_test_terminal()?;

    Ok(())
}

#[test]
fn simple_user_journey_read_only() -> Result<()> {
    let long_root = "sample-02/dir";
    let short_root = "sample-01";
    let (mut terminal, mut app) =
        initialized_app_and_terminal_from_fixture(&[short_root, long_root])?;

    // POST-INIT
    // after initialization, we expect that...
    {
        assert_eq!(
            app.state.sorting,
            SortMode::SizeDescending,
            "it will sort items in descending order by size"
        );

        assert!(
            app.state.scan.is_none(),
            "it will not think it is still scanning as there is no traversal"
        );

        let first_selected_path = OsString::from(format!("{FIXTURE_PATH}/{long_root}"));
        assert_eq!(
            node_by_name(&app, &first_selected_path).name,
            first_selected_path,
            "the roots are always listed with the given (possibly long) names",
        );

        assert_eq!(
            node_by_name(&app, fixture_str(short_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "it selects the first node in the list",
        );

        assert_eq!(
            app.traversal.root_index,
            app.state.navigation().view_root,
            "the root is the 'virtual' root",
        );
    }

    // SORTING
    {
        // when hitting the N key
        app.process_events(&mut terminal, into_codes("n"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::NameAscending,
            "it sets the sort mode to ascending by name"
        );
        // when hitting the N key again
        app.process_events(&mut terminal, into_codes("n"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::NameDescending,
            "it sets the sort mode to descending by name"
        );
        // when hitting the M key
        app.process_events(&mut terminal, into_codes("m"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::MTimeDescending(MTimeSort::Entry),
            "it sets the sort mode to descending by mtime"
        );
        // when hitting the M key again
        app.process_events(&mut terminal, into_codes("m"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::MTimeAscending(MTimeSort::Entry),
            "it sets the sort mode to ascending by mtime"
        );
        // when hitting the M key
        app.process_events(&mut terminal, into_codes("M"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::MTimeAscending(MTimeSort::RecursiveChildrenNewest),
            "it cycles the mtime sort mode to deep newest"
        );
        // when hitting the M key again
        app.process_events(&mut terminal, into_codes("M"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::MTimeAscending(MTimeSort::RecursiveChildrenOldest),
            "it cycles the mtime sort mode to deep oldest"
        );
        // when hitting the m key again
        app.process_events(&mut terminal, into_codes("m"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::MTimeDescending(MTimeSort::RecursiveChildrenOldest),
            "it toggles mtime direction without changing the mtime sort mode"
        );
        // when hitting the C key
        app.process_events(&mut terminal, into_codes("c"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::CountDescending,
            "it sets the sort mode to descending by count"
        );
        // when hitting the C key again
        app.process_events(&mut terminal, into_codes("c"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::CountAscending,
            "it sets the sort mode to ascending by count"
        );
        assert_eq!(
            node_by_index(&app, app.state.entries[0].index),
            node_by_name(&app, fixture_str(long_root)),
            "it recomputes the cached items"
        );
        // when hitting the S key
        app.process_events(&mut terminal, into_codes("s"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::SizeDescending,
            "it sets the sort mode to descending by size"
        );
        assert_eq!(
            node_by_index(&app, app.state.entries[1].index),
            node_by_name(&app, fixture_str(long_root)),
            "it recomputes the cached items"
        );
        // when hitting the S key again
        app.process_events(&mut terminal, into_codes("s"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::SizeAscending,
            "it sets the sort mode to ascending by size"
        );
        // hit the S key again to get Descending - the rest depends on it
        app.process_events(&mut terminal, into_codes("s"))?;
        assert_eq!(app.state.sorting, SortMode::SizeDescending,);

        assert_eq!(
            node_by_index(&app, app.state.entries[0].index),
            node_by_name(&app, fixture_str(short_root)),
            "it recomputes the cached items"
        );
    }

    // Columns
    {
        app.process_events(&mut terminal, into_codes("C"))?;
        assert!(
            app.state.show_columns.contains(&Column::Count),
            "hit the C key to show the entry count column"
        );

        app.process_events(&mut terminal, into_codes("C"))?;
        assert!(
            !app.state.show_columns.contains(&Column::Count),
            "when hitting the C key again it hides the entry count column"
        );

        app.process_events(&mut terminal, into_codes("M"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::SizeDescending,
            "hit the M key to show modified times without changing non-mtime sorting"
        );
        assert!(
            app.state.show_columns.contains(&Column::MTime),
            "hit the M key to show the modified time column"
        );

        app.process_events(&mut terminal, into_codes("M"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::SizeDescending,
            "hit the M key again to hide modified times without changing non-mtime sorting"
        );
        assert!(
            !app.state.show_columns.contains(&Column::MTime),
            "when hitting the M key again it hides the modified time column"
        );
    }

    // Glob pane open/close
    {
        app.process_events(&mut terminal, into_codes("/"))?;
        assert!(app.window.glob.is_some(), "'/' shows the glob pane");

        app.process_events(
            &mut terminal,
            into_events([Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))]),
        )?;
        assert!(app.window.glob.is_none(), "ESC closes the glob pane");
    }

    // explicit full refresh
    {
        assert!(app.state.scan.is_none(), "no refresh in progress");

        app.process_events(&mut terminal, into_codes("R"))?;
        assert!(
            app.state.scan.is_some(),
            "'R' refreshes all entries in the view"
        );

        app.run_until_traversed(&mut terminal, into_codes(""))?;
        assert!(app.state.scan.is_none(), "refresh should finish eventually");
    }

    // explicit partial refresh
    {
        assert!(app.state.scan.is_none(), "no refresh in progress");

        app.process_events(&mut terminal, into_codes("j"))?;
        assert_eq!(
            node_by_name(&app, fixture_str(long_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "it moves the cursor down and selects the next item based on the current sort mode"
        );

        app.process_events(&mut terminal, into_codes("r"))?;
        assert!(
            app.state.scan.is_some(),
            "'r' refreshes all entries in the view"
        );

        app.run_until_traversed(&mut terminal, into_events([]))?;
        assert!(app.state.scan.is_none(), "Refresh should finish");

        assert_eq!(
            node_by_name(&app, fixture_str(long_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "previous selection is preserved after refresh"
        );
    }

    // Entry-Navigation
    {
        // when hitting the j key
        app.process_events(&mut terminal, into_codes("j"))?;
        assert_eq!(
            node_by_name(&app, fixture_str(long_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "it moves the cursor down and selects the next item based on the current sort mode"
        );
        // when hitting it while there is nowhere to go
        app.process_events(&mut terminal, into_codes("j"))?;
        assert_eq!(
            node_by_name(&app, fixture_str(long_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "it stays at the previous position"
        );
        // when hitting the k key
        app.process_events(&mut terminal, into_codes("k"))?;
        assert_eq!(
            node_by_name(&app, fixture_str(short_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "it moves the cursor up and selects the next item based on the current sort mode"
        );
        // when hitting the k key again
        app.process_events(&mut terminal, into_codes("k"))?;
        assert_eq!(
            node_by_name(&app, fixture_str(short_root)),
            node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
            "it stays at the current cursor position as there is nowhere to go"
        );
        // when hitting the o key with a directory selected
        app.process_events(&mut terminal, into_codes("o"))?;
        {
            let new_root_idx = index_by_name(&app, fixture_str(short_root));
            assert_eq!(
                new_root_idx,
                app.state.navigation().view_root,
                "it enters the item if it is a directory, changing the root"
            );
            assert_eq!(
                index_by_name(&app, "dir"),
                *app.state.navigation().selected.as_ref().unwrap(),
                "it selects the first item in the directory"
            );

            // when hitting the u key while inside a sub-directory
            app.process_events(&mut terminal, into_codes("u"))?;
            {
                assert_eq!(
                    app.traversal.root_index,
                    app.state.navigation().view_root,
                    "it sets the root to be the (roots) parent directory, being the virtual root"
                );
                assert_eq!(
                    node_by_name(&app, fixture_str(short_root)),
                    node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
                    "changes the selection to the first item in the list of items"
                );
            }
        }
        // when hitting the u key while inside of the root directory
        // We are moving the cursor down just to have a non-default selection
        app.process_events(&mut terminal, into_codes("ju"))?;
        {
            assert_eq!(
                app.traversal.root_index,
                app.state.navigation().view_root,
                "it keeps the root - it can't go further up"
            );
            assert_eq!(
                node_by_name(&app, fixture_str(long_root)),
                node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
                "keeps the previous selection"
            );
        }
    }

    // Deletion
    {
        // when hitting the 'd' key (also move cursor back to start)
        app.process_events(&mut terminal, into_codes("k"))?;
        let previously_selected_index = *app.state.navigation().selected.as_ref().unwrap();
        app.process_events(&mut terminal, into_codes("d"))?;
        {
            assert_eq!(
                Some(1),
                app.window.mark.as_ref().map(|p| p.marked().len()),
                "it marks only a single node",
            );
            assert!(
                app.window
                    .mark
                    .as_ref()
                    .is_some_and(|p| p.marked().contains_key(&previously_selected_index)),
                "it marks the selected node"
            );
            assert_eq!(
                app.state.navigation().selected.as_ref().unwrap().index(),
                app.state.entries[1].index.index(),
                "moves the cursor down one level to facilitate many markings in a row"
            );
        }

        // when hitting the 'd' key again
        {
            app.process_events(&mut terminal, into_codes("d"))?;

            assert_eq!(
                Some(2),
                app.window.mark.as_ref().map(|p| p.marked().len()),
                "it marks the currently selected, second node",
            );

            assert_eq!(
                app.state.navigation().selected.as_ref().unwrap().index(),
                app.state.entries[1].index.index(),
                "it could not advance the cursor, thus the newly marked item is still selected"
            );
        }

        // when hitting the 'd' key once again
        {
            app.process_events(&mut terminal, into_codes("d"))?;

            assert_eq!(
                Some(1),
                app.window.mark.as_ref().map(|p| p.marked().len()),
                "it toggled the previous selected item off",
            );

            assert!(
                app.window
                    .mark
                    .as_ref()
                    .is_some_and(|p| p.marked().contains_key(&previously_selected_index)),
                "it leaves the first selected item marked"
            );
        }
        // when hitting the spacebar (after moving up to the first entry)
        {
            app.process_events(&mut terminal, into_codes("k "))?;

            assert_eq!(
                None,
                app.window.mark.as_ref().map(|p| p.marked().len()),
                "it toggles the item off",
            );

            assert_eq!(
                node_by_index(&app, previously_selected_index),
                node_by_index(&app, *app.state.navigation().selected.as_ref().unwrap()),
                "it does not advance the selection"
            );
        }
    }

    // Marking
    {
        // select something
        app.process_events(&mut terminal, into_codes(" j "))?;
        assert_eq!(
            Some(false),
            app.window.mark.as_ref().map(|pane| pane.has_focus()),
            "the marker pane starts out without focus",
        );

        assert_eq!(
            Some(2),
            app.window.mark.as_ref().map(|p| p.marked().len()),
            "it has two items marked",
        );

        // when advancing the selection to the marker pane
        app.process_events(&mut terminal, into_keys(Some(KeyCode::Tab)))?;
        {
            assert_eq!(
                Some(true),
                app.window.mark.as_ref().map(|pane| pane.has_focus()),
                "after tabbing into it, it has focus",
            );
        }

        // TODO: a bunch of additional tests are missing (handling of markers, deselecting them)
        // Yes, caught me, no TDD for these things, just because in Rust it's not needed as things
        // tend to just work when they compile, and while experimenting, tests can be in the way.
        // However, if Dua should be more widely used, we need CI and these tests written.
    }

    Ok(())
}

#[test]
fn configured_key_scans_the_parent_without_retraversing_the_current_root() -> Result<()> {
    let (mut terminal, mut app) = initialized_app_and_terminal_from_fixture(&["sample-02/dir"])?;
    app.config = toml::from_str(
        r#"
        [keys]
        scan_parent = "P"
        "#,
    )?;
    let dir = index_by_name(&app, fixture_str("sample-02/dir"));
    let sub = index_by_name(&app, "sub");
    let dir_size = node_by_index(&app, dir).size;
    let nodes_before = app.traversal.tree.len();

    app.process_events(&mut terminal, into_codes("u"))?;
    assert_eq!(
        app.state.message.as_deref(),
        Some("Top level reached. Press P to scan the parent directory")
    );

    app.process_events(&mut terminal, into_codes("U"))?;
    assert!(
        app.state.scan.is_none(),
        "the replaced default no longer starts the parent scan"
    );

    app.process_events(&mut terminal, into_codes("P"))?;
    assert!(
        app.state.scan.is_some(),
        "configured key starts the parent scan"
    );
    app.run_until_traversed(&mut terminal, into_events([]))?;

    assert_eq!(
        crate::interactive::path_of(&app.traversal.tree, app.traversal.root_index, None),
        fixture("sample-02").canonicalize()?,
        "the shared parent becomes the new root"
    );
    assert_eq!(
        index_by_name(&app, "dir"),
        dir,
        "the existing root is reattached instead of replaced"
    );
    assert_eq!(
        index_by_name(&app, "sub"),
        sub,
        "the existing subtree keeps its node identities"
    );
    assert_eq!(
        node_by_index(&app, dir).size,
        dir_size,
        "an explicit root's own metadata is not counted twice"
    );
    assert_eq!(
        app.traversal.tree.len(),
        nodes_before + 2,
        "only the two previously unseen siblings are added"
    );
    assert_eq!(
        app.state.stats.entries_traversed, 3,
        "the existing directory is observed but its descendants are not traversed"
    );

    Ok(())
}

#[test]
fn shift_u_wraps_a_complete_root_at_its_natural_position() -> Result<()> {
    let current_root = fixture("sample-02/dir").canonicalize()?;
    let root_paths = fs::read_dir(&current_root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    let (mut terminal, mut app) = initialized_app_and_terminal_from_paths(&root_paths)?;
    app.state.root_path = Some(current_root.clone());

    let old_root = app.traversal.root_index;
    let sub = index_by_name(&app, current_root.join("sub"));
    let size_before = node_by_index(&app, old_root).size;
    let count_before = node_by_index(&app, old_root)
        .entry_count
        .unwrap_or_default();
    let nodes_before = app.traversal.tree.len();

    app.process_events(&mut terminal, into_codes("U"))?;
    app.run_until_traversed(&mut terminal, into_events([]))?;

    assert_eq!(
        crate::interactive::path_of(&app.traversal.tree, app.traversal.root_index, None),
        current_root.parent().unwrap(),
        "the complete root's parent becomes the new root"
    );
    assert_eq!(
        index_by_name(&app, "dir"),
        old_root,
        "the previous root becomes its naturally named child"
    );
    assert_eq!(index_by_name(&app, "sub"), sub);
    let promoted_root = node_by_index(&app, old_root);
    assert_eq!(
        promoted_root.size,
        size_before + u128::from(current_root.metadata()?.len()),
        "the promoted node gains the directory entry's own size"
    );
    assert_eq!(promoted_root.entry_count, Some(count_before + 1));
    assert_eq!(app.traversal.tree.len(), nodes_before + 3);
    assert_eq!(app.state.stats.entries_traversed, 3);

    Ok(())
}

#[test]
fn configured_keybinding_replaces_only_its_default() -> Result<()> {
    let (mut terminal, mut app) =
        initialized_app_and_terminal_from_fixture(&["sample-01", "sample-02"])?;
    app.config = toml::from_str(
        r#"
        [keys]
        sort_by_name = "ctrl+b"
        "#,
    )?;

    let initial_selection = app.state.navigation().selected;
    app.process_events(&mut terminal, into_codes("j"))?;
    assert_ne!(
        app.state.navigation().selected,
        initial_selection,
        "an unspecified binding keeps its default"
    );

    app.process_events(&mut terminal, into_codes("n"))?;
    assert_eq!(
        app.state.sorting,
        SortMode::SizeDescending,
        "the overridden default no longer invokes the action"
    );

    app.process_events(
        &mut terminal,
        into_events([Event::Key(KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::CONTROL,
        ))]),
    )?;
    assert_eq!(app.state.sorting, SortMode::NameAscending);
    Ok(())
}

#[test]
fn once_finishes_traversal_without_user_events() -> Result<()> {
    let (mut terminal, mut app) = untraversed_app_and_terminal_from_fixture(&["sample-01"])?;
    app.traverse()?;

    let result = app.process_events_once(&mut terminal, into_events([]))?;

    assert_eq!(result.num_errors, 0);
    assert!(
        app.state.scan.is_none(),
        "once mode should stop after traversal completes"
    );

    Ok(())
}

#[test]
fn tracks_terminal_focus_events() -> Result<()> {
    let (mut terminal, mut app) = initialized_app_and_terminal_from_fixture(&["sample-01"])?;

    app.process_events(&mut terminal, into_events([Event::FocusLost]))?;
    assert!(!app.state.terminal_focus.is_focussed());

    app.process_events(&mut terminal, into_events([Event::FocusGained]))?;
    assert!(app.state.terminal_focus.is_focussed());
    Ok(())
}

#[test]
fn once_replays_user_events_after_traversal() -> Result<()> {
    let (mut terminal, mut app) = untraversed_app_and_terminal_from_fixture(&["sample-01"])?;
    app.traverse()?;

    app.process_events_once(&mut terminal, into_codes("n"))?;

    assert_eq!(
        app.state.sorting,
        SortMode::NameAscending,
        "once mode should replay supplied key events after traversal"
    );

    Ok(())
}

#[test]
fn once_allows_replayed_quit_to_exit() -> Result<()> {
    let (mut terminal, mut app) = untraversed_app_and_terminal_from_fixture(&["sample-01"])?;
    app.traverse()?;

    let result = app.process_events_once(&mut terminal, into_codes("q"))?;

    assert_eq!(result.num_errors, 0);

    Ok(())
}

#[test]
fn once_waits_for_replayed_refresh_to_finish() -> Result<()> {
    let (mut terminal, mut app) = untraversed_app_and_terminal_from_fixture(&["sample-01"])?;
    app.traverse()?;

    let result = app.process_events_once(&mut terminal, into_codes("R"))?;

    assert_eq!(result.num_errors, 0);
    assert!(
        app.state.scan.is_none(),
        "once mode should wait for refreshes started by replayed events"
    );

    Ok(())
}

#[test]
fn snapshot_roundtrip_is_read_only() -> Result<()> {
    use crate::interactive::terminal::TerminalApp;
    use dua::{ByteFormat, Config};

    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("root");
    fs::create_dir(&root)?;
    fs::write(root.join("a"), b"a")?;
    fs::write(root.join("b"), b"bb")?;
    fs::create_dir(root.join("dir"))?;
    fs::write(root.join("dir/file"), b"content")?;
    let snapshot_dir = tempfile::tempdir()?;
    let snapshot_path = snapshot_dir.path().join("scan.dua");
    fs::write(&snapshot_path, b"old snapshot")?;
    let (mut terminal, mut scanned) = untraversed_app_and_terminal_with_closure(
        std::slice::from_ref(&root),
        std::path::Path::to_path_buf,
    )?;
    scanned.traverse_and_export(snapshot_path.clone(), Some(2))?;
    assert_eq!(
        fs::read(&snapshot_path)?,
        b"old snapshot",
        "export waits for the traversal to finish"
    );
    scanned.run_until_traversed(&mut terminal, into_events([]))?;

    let snapshot = dua::snapshot::read(fs::File::open(&snapshot_path)?)?;
    let root_paths = snapshot
        .roots
        .iter()
        .map(|root| {
            snapshot
                .traversal
                .tree
                .name(*root)
                .expect("snapshot root exists")
                .into_owned()
        })
        .collect();
    let snapshot_load_duration = Duration::from_millis(123);
    let mut terminal = new_test_terminal()?;
    let mut app = TerminalApp::initialize(
        &mut terminal,
        scanned.state.walk_options.clone(),
        ByteFormat::Metric,
        true,
        root_paths,
        None,
        Config::default(),
        snapshot.traversal,
        Some(snapshot_load_duration),
    )?;

    assert!(app.state.read_only);
    assert_eq!(app.state.stats.elapsed, Some(snapshot_load_duration));
    assert!(app.state.scan.is_none(), "import starts no traversal");
    assert!(app.state.gitignored_entries.is_none());

    fs::remove_file(root.join("a"))?;
    app.process_events(&mut terminal, into_codes("o"))?;
    let missing = app
        .state
        .entries
        .iter()
        .find(|entry| entry.name == std::path::Path::new("a"))
        .expect("snapshot contains a");
    assert!(
        missing.exists,
        "snapshot entries are not checked against the local filesystem"
    );
    let missing_index = missing.index;

    app.process_events(&mut terminal, into_codes("R"))?;
    assert!(app.state.scan.is_none(), "refresh remains disabled");
    assert_eq!(
        app.state.message.as_deref(),
        Some("Snapshots are read-only")
    );

    app.process_events(&mut terminal, into_codes("U"))?;
    assert!(app.state.scan.is_none(), "parent scan remains disabled");
    assert_eq!(
        app.state.message.as_deref(),
        Some("Snapshots are read-only")
    );

    app.state.navigation_mut().select(Some(missing_index));
    app.process_events(&mut terminal, into_codes("O"))?;
    let missing_message = format!("Snapshot path is unavailable: {}", root.join("a").display());
    assert_eq!(app.state.message.as_deref(), Some(missing_message.as_str()));

    app.process_events(&mut terminal, into_codes("i"))?;
    assert!(app.state.gitignored_entries.is_none());
    assert_eq!(
        app.state.message.as_deref(),
        Some("Gitignored entry detection is unavailable for snapshots")
    );

    let victim = root.join("b");
    let victim_index = app
        .state
        .entries
        .iter()
        .find(|entry| entry.name == std::path::Path::new("b"))
        .expect("snapshot contains b")
        .index;
    app.state.navigation_mut().select(Some(victim_index));
    app.process_events(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Char(' ').into()),
            Event::Key(KeyCode::Tab.into()),
            Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
        ]),
    )?;
    assert!(victim.exists(), "delete is disabled for snapshots");
    assert_eq!(
        app.state.message.as_deref(),
        Some("Snapshots are read-only")
    );

    #[cfg(feature = "trash-move")]
    {
        app.process_events(
            &mut terminal,
            into_events([Event::Key(KeyEvent::new(
                KeyCode::Char('t'),
                KeyModifiers::CONTROL,
            ))]),
        )?;
        assert!(victim.exists(), "move to trash is disabled for snapshots");
    }

    app.process_events(&mut terminal, into_keys([KeyCode::Tab]))?;
    let marked_paths = app
        .window
        .mark
        .take()
        .expect("marking remains available")
        .into_paths()
        .collect::<Vec<_>>();
    assert_eq!(marked_paths, [victim]);

    app.process_events(&mut terminal, into_codes("n"))?;
    assert_eq!(app.state.sorting, SortMode::NameAscending);
    app.process_events(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Char('/').into()),
            Event::Key(KeyCode::Char('d').into()),
            Event::Key(KeyCode::Char('i').into()),
            Event::Key(KeyCode::Char('r').into()),
            Event::Key(KeyCode::Enter.into()),
        ]),
    )?;
    assert!(
        app.state.glob_navigation.is_some(),
        "globbing remains available"
    );

    Ok(())
}

#[test]
fn quit_instantly_when_nothing_marked() -> Result<()> {
    let short_root = "sample-01";
    let (mut terminal, mut app) = initialized_app_and_terminal_from_fixture(&[short_root])?;

    // When pressing 'q' without any items marked for deletion
    let result = app.process_events(&mut terminal, into_codes("q"))?;

    assert_eq!(
        result.num_errors, 0,
        "it should quit instantly without errors"
    );

    Ok(())
}

#[test]
fn quit_requires_two_presses_when_items_marked() -> Result<()> {
    let short_root = "sample-01";
    let (mut terminal, mut app) = initialized_app_and_terminal_from_fixture(&[short_root])?;

    // Mark an item for deletion
    app.process_events(&mut terminal, into_codes("d"))?;

    assert_eq!(
        app.window.mark.as_ref().map(|p| p.marked().len()),
        Some(1),
        "expecting one marked item"
    );

    // First 'q' press should set pending_exit
    app.process_events(&mut terminal, into_codes("q"))?;

    assert!(
        app.state.pending_exit,
        "first 'q' should set pending_exit when items are marked"
    );

    // Second 'q' press should quit
    let result = app.process_events(&mut terminal, into_codes("q"))?;

    assert_eq!(
        result.num_errors, 0,
        "second 'q' should quit the application"
    );

    Ok(())
}
