use std::{fs, sync::atomic::Ordering, time::Instant};

use anyhow::Result;
use crossbeam::channel::{Receiver, Sender, never, unbounded};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use dua::{WalkResult, traverse::TreeIndex};
use tempfile::TempDir;
use tui::{Terminal, backend::TestBackend};

use super::utils::{
    index_by_name, initialized_app_and_terminal_from_paths, into_codes, into_events,
};
use crate::interactive::app::{
    deletion::DeletionEvent, deletion_progress::FilesystemDeletion, state::FocussedPane,
    terminal::TerminalApp,
};

fn prepared() -> Result<(TempDir, Terminal<TestBackend>, TerminalApp)> {
    let fixture = TempDir::new()?;
    fs::create_dir(fixture.path().join("delete"))?;
    fs::write(fixture.path().join("delete/remove.bin"), [0; 64])?;
    fs::write(fixture.path().join("delete/zero.bin"), [])?;
    fs::create_dir(fixture.path().join("keep"))?;
    fs::write(fixture.path().join("keep/keep.bin"), [0; 32])?;
    let (mut terminal, mut app) =
        initialized_app_and_terminal_from_paths(&[fixture.path().canonicalize()?])?;
    app.process_events_once(&mut terminal, into_codes("o"))?;
    let target = index_by_name(&app, "delete");
    app.state.navigation.select(Some(target));
    app.process_events_once(&mut terminal, into_codes(" "))?;
    assert!(
        app.window
            .mark
            .as_ref()
            .unwrap()
            .marked()
            .contains_key(&target)
    );
    Ok((fixture, terminal, app))
}

fn inject(app: &mut TerminalApp, target: TreeIndex) -> (Sender<DeletionEvent>, Sender<Instant>) {
    let (send, events) = unbounded();
    let (tick, ticks) = unbounded();
    let tree = app.state.tree_view(&mut app.traversal);
    let mut deletion = FilesystemDeletion::for_test(&tree, vec![target], events);
    deletion.tick = ticks;
    app.state.deletion = Some(deletion);
    app.state.reset_message();
    (send, tick)
}

fn step(
    app: &mut TerminalApp,
    terminal: &mut Terminal<TestBackend>,
    input: &Receiver<Event>,
) -> Result<Option<WalkResult>> {
    app.state.process_event(
        &mut app.window,
        &mut app.traversal,
        &mut app.display,
        terminal,
        input,
        &app.config,
    )
}

fn key(app: &mut TerminalApp, terminal: &mut Terminal<TestBackend>, key: KeyEvent) -> Result<()> {
    assert!(step(app, terminal, &into_events([Event::Key(key)]))?.is_none());
    Ok(())
}

#[test]
fn deletion_ticks_update_remaining_bytes_without_input_and_keep_failed_marks() -> Result<()> {
    let (fixture, mut terminal, mut app) = prepared()?;
    let target = index_by_name(&app, "delete");
    let file = index_by_name(&app, "remove.bin");
    let zero = index_by_name(&app, "zero.bin");
    // Give the directory itself a known size on every platform. Removing its files must
    // preserve these bytes, including when a hardlink/APFS-deduplicated entry has size zero.
    let scanned = app.traversal.tree.data(target).unwrap().size;
    let mut ancestor = Some(target);
    while let Some(index) = ancestor {
        app.traversal
            .tree
            .update(index, |entry| entry.size = entry.size - scanned + 81);
        ancestor = app.traversal.tree.parent(index);
    }
    app.state.stats.total_bytes = app.state.stats.total_bytes.map(|size| size - scanned + 81);
    let tree = app.state.tree_view(&mut app.traversal);
    app.window.mark.as_mut().unwrap().reconcile(&tree);
    let total = tree.total_size();
    let count = tree.tree().data(target).unwrap().entry_count.unwrap();
    let (send, tick) = inject(&mut app, target);
    let no_input = never();
    let before = terminal.backend().buffer().clone();

    fs::remove_file(fixture.path().join("delete/remove.bin"))?;
    send.send(DeletionEvent::Removed {
        target: 0,
        path: fixture.path().canonicalize()?.join("delete/remove.bin"),
    })?;
    assert!(step(&mut app, &mut terminal, &no_input)?.is_none());
    assert!(
        app.traversal.tree.contains(file),
        "results wait for a redraw batch"
    );
    tick.send(Instant::now())?;
    assert!(step(&mut app, &mut terminal, &no_input)?.is_none());
    assert!(!app.traversal.tree.contains(file));
    assert_eq!(app.traversal.tree.data(target).unwrap().size, 17);
    assert_eq!(app.window.mark.as_ref().unwrap().total_size(), 17);
    assert_eq!(app.state.stats.total_bytes, Some(total - 64));
    assert!(app.state.message.as_ref().unwrap().contains("remaining"));
    assert_ne!(terminal.backend().buffer(), &before);
    assert!(app.state.is_deleting());

    fs::remove_file(fixture.path().join("delete/zero.bin"))?;
    send.send(DeletionEvent::Removed {
        target: 0,
        path: fixture.path().canonicalize()?.join("delete/zero.bin"),
    })?;
    step(&mut app, &mut terminal, &no_input)?;
    tick.send(Instant::now())?;
    step(&mut app, &mut terminal, &no_input)?;
    assert!(!app.traversal.tree.contains(zero));
    assert_eq!(app.traversal.tree.data(target).unwrap().size, 17);
    assert_eq!(
        app.traversal.tree.data(target).unwrap().entry_count,
        Some(count - 2)
    );

    send.send(DeletionEvent::TargetFinished {
        target: 0,
        errors: 1,
    })?;
    send.send(DeletionEvent::Finished {
        entries: 2,
        errors: 1,
        cancelled: false,
    })?;
    assert!(step(&mut app, &mut terminal, &no_input)?.is_none());
    assert!(!app.state.is_deleting());
    let pane = app.window.mark.as_ref().unwrap();
    assert_eq!(pane.total_size(), 17);
    assert_eq!(pane.marked()[&target].num_errors_during_deletion, 1);
    key(&mut app, &mut terminal, KeyCode::Char(' ').into())?;
    assert!(
        app.window.mark.is_none(),
        "completion releases the mutation lock"
    );
    Ok(())
}

#[test]
fn deletion_allows_navigation_and_pane_controls_but_blocks_changes() -> Result<()> {
    let (_fixture, mut terminal, mut app) = prepared()?;
    let target = index_by_name(&app, "delete");
    let root = app.state.navigation.view_root;
    let nodes = app.traversal.tree.len();
    let (_send, _tick) = inject(&mut app, target);
    let cancel = app.state.deletion.as_ref().unwrap().task.cancel.clone();

    key(&mut app, &mut terminal, KeyCode::Char('o').into())?;
    assert_eq!(app.state.navigation.view_root, target);
    key(&mut app, &mut terminal, KeyCode::Char('j').into())?;
    assert!(app.state.navigation.selected.is_some());
    key(&mut app, &mut terminal, KeyCode::Char('u').into())?;
    assert_eq!(app.state.navigation.view_root, root);
    for action in [' ', 'x', 'd', 'a', 'X', 'I', 'r', 'R', 'U'] {
        key(&mut app, &mut terminal, KeyCode::Char(action).into())?;
        assert!(app.state.scan.is_none());
        assert_eq!(app.traversal.tree.len(), nodes);
        assert_eq!(
            app.window
                .mark
                .as_ref()
                .unwrap()
                .marked()
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            [target]
        );
        assert_eq!(
            app.state.message.as_deref(),
            Some(app.state.language.ui_text().deletion_running)
        );
    }
    key(&mut app, &mut terminal, KeyCode::Tab.into())?;
    assert!(app.state.focussed == FocussedPane::Mark);
    for action in [
        KeyCode::Char('x').into(),
        KeyCode::Char('a').into(),
        KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
    ] {
        key(&mut app, &mut terminal, action)?;
        assert_eq!(app.window.mark.as_ref().unwrap().marked().len(), 1);
    }
    key(&mut app, &mut terminal, KeyCode::Esc.into())?;
    assert!(app.state.focussed == FocussedPane::Main);
    key(&mut app, &mut terminal, KeyCode::Char('?').into())?;
    assert!(app.window.help.is_some());
    key(&mut app, &mut terminal, KeyCode::Esc.into())?;
    assert!(app.window.help.is_none());
    key(&mut app, &mut terminal, KeyCode::Char(']').into())?;
    assert!(app.window.right_panes_minimized);
    key(&mut app, &mut terminal, KeyCode::Char('n').into())?;
    assert!(
        !cancel.load(Ordering::Relaxed),
        "closing panes does not cancel deletion"
    );
    assert!(app.state.is_deleting());
    Ok(())
}

#[test]
fn deleting_the_view_repairs_normal_and_glob_navigation_without_double_counting() -> Result<()> {
    for glob in [false, true] {
        let (fixture, mut terminal, mut app) = prepared()?;
        let target = index_by_name(&app, "delete");
        let file = index_by_name(&app, "remove.bin");
        let total = app.state.stats.total_bytes.unwrap();
        let size = app.traversal.tree.data(target).unwrap().size;
        let (send, tick) = inject(&mut app, target);
        if glob {
            for action in "/delete".chars() {
                key(&mut app, &mut terminal, KeyCode::Char(action).into())?;
            }
            key(&mut app, &mut terminal, KeyCode::Enter.into())?;
            assert!(app.state.glob_navigation.is_some());
        }
        key(&mut app, &mut terminal, KeyCode::Char('o').into())?;
        assert_eq!(app.state.navigation().view_root, target);

        let path = fixture.path().canonicalize()?.join("delete");
        fs::remove_dir_all(&path)?;
        send.send(DeletionEvent::Removed {
            target: 0,
            path: path.join("remove.bin"),
        })?;
        send.send(DeletionEvent::Removed { target: 0, path })?;
        step(&mut app, &mut terminal, &never())?;
        tick.send(Instant::now())?;
        step(&mut app, &mut terminal, &never())?;
        assert!(!app.traversal.tree.contains(target));
        assert!(!app.traversal.tree.contains(file));
        assert_eq!(app.state.stats.total_bytes, Some(total - size));
        assert!(app.window.mark.is_none());
        assert!(
            app.traversal
                .tree
                .contains(app.state.navigation().view_root)
        );
        assert!(
            app.state
                .entries
                .iter()
                .all(|entry| app.traversal.tree.contains(entry.index))
        );
        if let Some(navigation) = &app.state.glob_navigation {
            assert_eq!(navigation.view_root, navigation.tree_root);
            assert!(navigation.matches.is_empty());
            assert!(app.state.entries.is_empty());
            assert!(!navigation.bookmarks.contains_key(&target));
        }
        assert!(
            app.state.is_deleting(),
            "a removed root does not end the worker batch"
        );
        send.send(DeletionEvent::Finished {
            entries: 3,
            errors: 0,
            cancelled: false,
        })?;
        assert!(step(&mut app, &mut terminal, &never())?.is_none());
        assert!(!app.state.is_deleting());
    }
    Ok(())
}

#[test]
fn quit_cancels_pending_deletion_and_waits_for_inflight_results() -> Result<()> {
    let (fixture, mut terminal, mut app) = prepared()?;
    let target = index_by_name(&app, "delete");
    let file = index_by_name(&app, "remove.bin");
    let (send, _tick) = inject(&mut app, target);
    let quit = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    key(&mut app, &mut terminal, quit)?;
    assert!(
        app.state
            .deletion
            .as_ref()
            .unwrap()
            .task
            .cancel
            .load(Ordering::Relaxed)
    );
    assert_eq!(
        app.state.message.as_deref(),
        Some(app.state.language.ui_text().cancelling_deletion)
    );

    fs::remove_file(fixture.path().join("delete/remove.bin"))?;
    send.send(DeletionEvent::Removed {
        target: 0,
        path: fixture.path().canonicalize()?.join("delete/remove.bin"),
    })?;
    step(&mut app, &mut terminal, &never())?;
    key(&mut app, &mut terminal, quit)?;
    assert!(
        app.state.is_deleting(),
        "repeated quit still waits for the worker"
    );
    assert!(
        !app.traversal.tree.contains(file),
        "in-flight successes are retained"
    );
    send.send(DeletionEvent::Finished {
        entries: 1,
        errors: 0,
        cancelled: true,
    })?;
    assert!(step(&mut app, &mut terminal, &never())?.is_some());
    assert!(!app.state.is_deleting());
    assert!(fixture.path().join("delete/zero.bin").exists());
    Ok(())
}

#[test]
fn exhausted_input_drains_deletion_without_cancelling() -> Result<()> {
    let (fixture, mut terminal, mut app) = prepared()?;
    let target = index_by_name(&app, "delete");
    let file = index_by_name(&app, "remove.bin");
    let (send, _tick) = inject(&mut app, target);
    let input = into_events([]);
    assert!(step(&mut app, &mut terminal, &input)?.is_none());
    let deletion = app.state.deletion.as_ref().unwrap();
    assert!(deletion.input_closed);
    assert!(!deletion.task.cancel.load(Ordering::Relaxed));
    fs::remove_file(fixture.path().join("delete/remove.bin"))?;
    send.send(DeletionEvent::Removed {
        target: 0,
        path: fixture.path().canonicalize()?.join("delete/remove.bin"),
    })?;
    send.send(DeletionEvent::Finished {
        entries: 1,
        errors: 0,
        cancelled: false,
    })?;
    assert!(step(&mut app, &mut terminal, &input)?.is_some());
    assert!(!app.state.is_deleting());
    assert!(!app.traversal.tree.contains(file));
    Ok(())
}

#[test]
fn event_loop_errors_stop_deletion_before_the_app_can_be_leaked() -> Result<()> {
    for once in [false, true] {
        let (_fixture, mut terminal, mut app) = prepared()?;
        let target = index_by_name(&app, "delete");
        let (send, _tick) = inject(&mut app, target);
        let cancel = app.state.deletion.as_ref().unwrap().task.cancel.clone();
        drop(send);
        let result = if once {
            app.process_events_once(&mut terminal, never())
        } else {
            app.process_events(&mut terminal, never())
        };
        assert!(result.is_err());
        assert!(cancel.load(Ordering::Relaxed));
        assert!(!app.state.is_deleting());
    }
    Ok(())
}
