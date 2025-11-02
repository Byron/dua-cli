use anyhow::Result;
use crosstermion::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crosstermion::input::Event;
use pretty_assertions::assert_eq;
use std::ffi::OsString;

use crate::interactive::app::tests::utils::{into_codes, into_events};
use crate::interactive::widgets::Column;
use crate::interactive::{
    SortMode,
    app::tests::{
        FIXTURE_PATH,
        utils::{
            fixture_str, index_by_name, initialized_app_and_terminal_from_fixture, into_keys,
            node_by_index, node_by_name,
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
            SortMode::MTimeDescending,
            "it sets the sort mode to descending by mtime"
        );
        // when hitting the M key again
        app.process_events(&mut terminal, into_codes("m"))?;
        assert_eq!(
            app.state.sorting,
            SortMode::MTimeAscending,
            "it sets the sort mode to ascending by mtime"
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
        assert!(
            app.state.show_columns.contains(&Column::MTime),
            "hit the M key to show the entry count column"
        );

        app.process_events(&mut terminal, into_codes("M"))?;
        assert!(
            !app.state.show_columns.contains(&Column::MTime),
            "when hitting the M key again it hides the entry count column"
        );
    }

    // Glob pane open/close
    {
        app.process_events(&mut terminal, into_codes("/"))?;
        assert!(app.window.glob_pane.is_some(), "'/' shows the glob pane");

        app.process_events(
            &mut terminal,
            into_events([Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))]),
        )?;
        assert!(app.window.glob_pane.is_none(), "ESC closes the glob pane");
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
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
                "it marks only a single node",
            );
            assert!(
                app.window
                    .mark_pane
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
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
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
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
                "it toggled the previous selected item off",
            );

            assert!(
                app.window
                    .mark_pane
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
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
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
            app.window.mark_pane.as_ref().map(|p| p.has_focus()),
            "the marker pane starts out without focus",
        );

        assert_eq!(
            Some(2),
            app.window.mark_pane.as_ref().map(|p| p.marked().len()),
            "it has two items marked",
        );

        // when advancing the selection to the marker pane
        app.process_events(&mut terminal, into_keys(Some(KeyCode::Tab)))?;
        {
            assert_eq!(
                Some(true),
                app.window.mark_pane.as_ref().map(|p| p.has_focus()),
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
        app.window.mark_pane.as_ref().map(|p| p.marked().len()),
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
