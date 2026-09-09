use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect};

use super::utils::{into_codes, into_events, into_keys, untraversed_app_and_terminal_with_closure};
use crate::interactive::{
    SortMode,
    app::{clean_hub::CleanHub, state::Cursor},
    widgets::{COLOR_MARKED, MainWindowProps},
};

fn hub(app: &crate::TerminalApp) -> &CleanHub {
    app.state.clean_hub.as_ref().unwrap()
}

fn entry_names(app: &crate::TerminalApp) -> BTreeSet<PathBuf> {
    if let Some(hub) = &app.state.clean_hub
        && hub.root.is_none()
    {
        return hub.rows.iter().map(|row| row.entry.name.clone()).collect();
    }
    app.state
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect()
}

fn select_hub_entry(
    app: &mut crate::TerminalApp,
    terminal: &mut Terminal<TestBackend>,
    path: &Path,
) -> Result<()> {
    let position = hub(app)
        .rows
        .iter()
        .position(|row| row.entry.name == path)
        .unwrap();
    app.process_events_once(
        terminal,
        into_keys(
            [KeyCode::Home]
                .into_iter()
                .chain(std::iter::repeat_n(KeyCode::Down, position)),
        ),
    )?;
    assert_eq!(hub(app).selected_path(), Some(path));
    Ok(())
}

#[test]
fn clean_hub_titles_show_search_inputs_without_changing_browser_titles() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().canonicalize()?;
    let first = root.join("first-search");
    let second = root.join("second-search");
    for path in [&first, &second] {
        fs::create_dir_all(path.join("__pycache__"))?;
        fs::write(path.join("__pycache__/cache"), b"cache")?;
    }
    let title = |terminal: &Terminal<TestBackend>| {
        let buffer = terminal.backend().buffer();
        (0..buffer.area.width)
            .map(|x| buffer[(x, 1)].symbol())
            .collect::<String>()
    };
    for inputs in [vec![first.clone()], vec![first.clone(), second]] {
        let (_, mut app) = untraversed_app_and_terminal_with_closure(&inputs, Path::to_path_buf)?;
        let mut terminal = Terminal::new(TestBackend::new(512, 20))?;
        let cwd = std::env::current_dir()?;
        assert!(inputs.iter().all(|path| !path.starts_with(&cwd)));
        app.traverse_clean(None)?;
        app.process_events_once(&mut terminal, into_events([]))?;
        let expected = inputs
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let rendered = title(&terminal);
        assert!(rendered.contains(&format!(" {expected} ")), "{rendered}");
        assert!(!rendered.contains(&cwd.display().to_string()));
    }

    let (_, mut app) =
        untraversed_app_and_terminal_with_closure(std::slice::from_ref(&first), Path::to_path_buf)?;
    let mut terminal = Terminal::new(TestBackend::new(512, 20))?;
    app.traverse()?;
    app.process_events_once(&mut terminal, into_events([]))?;
    assert!(app.state.clean_hub.is_none());
    assert!(title(&terminal).contains(&format!(
        " {} ",
        std::env::current_dir()?.canonicalize()?.display()
    )));
    app.process_events_once(&mut terminal, into_codes("o"))?;
    assert!(title(&terminal).contains(&format!(" {} ", first.display())));
    Ok(())
}

#[test]
fn clean_groups_siblings_and_marks_only_candidates() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().canonicalize()?;
    gix::init(&root)?;
    fs::write(root.join(".gitignore"), b".*_cache/\n__pycache__/\n")?;
    let project = root.join("project");
    let candidates = [
        project.join(".mypy_cache"),
        project.join(".pytest_cache"),
        project.join("git/__pycache__"),
        project.join("git/.ruff_cache"),
        project.join("test/__pycache__"),
    ];
    let singleton = root.join("project.v4-breaking/test/__pycache__");
    for (index, path) in candidates.iter().chain([&singleton]).enumerate() {
        fs::create_dir_all(path.join("cache"))?;
        fs::write(path.join("cache/payload"), vec![b'x'; 100 * (index + 1)])?;
    }
    fs::write(singleton.join("cache/payload"), vec![b'x'; 10_000])?;
    fs::write(project.join("source.py"), b"keep this source")?;
    let (mut terminal, mut app) =
        untraversed_app_and_terminal_with_closure(std::slice::from_ref(&root), Path::to_path_buf)?;
    app.traverse_clean(None)?;
    app.process_events_once(&mut terminal, into_events([]))?;

    assert!(hub(&app).root.is_none());
    assert!(app.state.entries.is_empty(), "the hub owns its rows");
    assert_eq!(hub(&app).rows.len(), 2, "siblings share a hub row");
    assert_eq!(
        app.traversal
            .tree
            .children(app.traversal.root_index)
            .count(),
        6
    );
    let group = hub(&app)
        .rows
        .iter()
        .find(|row| row.entry.name == project)
        .unwrap();
    let members = &group.members;
    assert_eq!(members.len(), 5);
    assert_eq!(
        group.entry.size,
        members
            .iter()
            .map(|index| app.traversal.tree.data(*index).unwrap().size)
            .sum::<u128>()
    );
    let order = hub(&app)
        .rows
        .iter()
        .map(|row| row.entry.name.clone())
        .collect::<Vec<_>>();
    app.process_events_once(&mut terminal, into_codes("smnc/"))?;
    assert_eq!(app.state.sorting, SortMode::SizeDescending);
    assert_eq!(
        hub(&app)
            .rows
            .iter()
            .map(|row| row.entry.name.clone())
            .collect::<Vec<_>>(),
        order,
        "ordinary sorting keys do not reorder hub rows"
    );
    assert!(app.window.glob.is_none(), "the hub has no search");
    select_hub_entry(&mut app, &mut terminal, &project)?;
    app.process_events_once(&mut terminal, into_codes(" "))?;
    let marked_paths = |app: &crate::TerminalApp| {
        app.window
            .mark
            .as_ref()
            .unwrap()
            .marked()
            .values()
            .map(|entry| entry.path.clone())
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(marked_paths(&app), BTreeSet::from(candidates.clone()));
    // Render before the terminal's NO_COLOR postprocessing.
    let group_color = |app: &mut crate::TerminalApp| {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 512, 20));
        let props = MainWindowProps {
            current_path: hub(app).title(),
            entries_traversed: app.state.stats.entries_traversed,
            total_bytes: app.state.stats.total_bytes.unwrap_or_default(),
            start: app.state.stats.start,
            elapsed: app.state.stats.elapsed,
            display: app.display,
            state: &app.state,
            config: &app.config,
        };
        app.window
            .render(props, buffer.area, &mut buffer, &mut Cursor::default());
        let y = 2 + hub(app)
            .rows
            .iter()
            .position(|row| row.entry.name == project)
            .expect("group row") as u16;
        let x = (1..256)
            .find(|&x| buffer[(x, y)].symbol() == "/")
            .expect("group path");
        buffer[(x, y)].fg
    };
    assert_eq!(group_color(&mut app), COLOR_MARKED);
    app.process_events_once(&mut terminal, into_codes(" "))?;
    assert!(
        app.window.mark.is_none(),
        "a fully marked group toggles off"
    );

    app.process_events_once(&mut terminal, into_codes("o"))?;
    assert_eq!(hub(&app).root, Some(app.state.navigation.tree_root));
    assert_ne!(app.state.navigation.tree_root, app.traversal.root_index);
    assert_eq!(
        entry_names(&app),
        candidates
            .iter()
            .map(|path| path.strip_prefix(&project).unwrap().to_owned())
            .collect(),
        "a group lists its five real candidates directly"
    );
    assert_eq!(
        app.state.gitignored_entries,
        Some(app.state.entries.iter().map(|entry| entry.index).collect())
    );
    app.process_events_once(&mut terminal, into_codes("n/__pycache__"))?;
    app.process_events_once(&mut terminal, into_keys([KeyCode::Enter]))?;
    assert_eq!(app.state.sorting, SortMode::NameAscending);
    assert!(app.state.glob_navigation.is_some());
    assert_eq!(
        entry_names(&app),
        BTreeSet::from([candidates[2].clone(), candidates[4].clone()])
    );
    app.process_events_once(&mut terminal, into_codes("oR"))?;
    assert_eq!(entry_names(&app), BTreeSet::from([PathBuf::from("cache")]));
    app.process_events_once(&mut terminal, into_codes("u"))?;
    assert_eq!(
        entry_names(&app),
        BTreeSet::from([candidates[2].clone(), candidates[4].clone()])
    );
    app.process_events_once(&mut terminal, into_codes("q"))?;
    assert!(app.state.glob_navigation.is_none());
    assert_eq!(hub(&app).root, Some(app.state.navigation.tree_root));
    let selected = app
        .state
        .entries
        .iter()
        .find(|entry| entry.name == Path::new(".mypy_cache"))
        .unwrap()
        .index;
    app.state.navigation.select(Some(selected));
    app.process_events_once(&mut terminal, into_codes("o/cache"))?;
    app.process_events_once(&mut terminal, into_keys([KeyCode::Enter]))?;
    app.process_events_once(&mut terminal, into_codes("oR"))?;
    app.process_events_once(&mut terminal, into_codes("u"))?;
    assert_eq!(
        entry_names(&app),
        BTreeSet::from([candidates[0].join("cache")])
    );
    app.process_events_once(&mut terminal, into_codes("qu"))?;
    app.process_events_once(&mut terminal, into_codes(" u"))?;
    assert!(hub(&app).root.is_none());
    assert_eq!(
        app.state.message, None,
        "hub clears browser annotation counts"
    );
    assert_eq!(app.window.mark.as_ref().unwrap().marked().len(), 1);
    assert_ne!(group_color(&mut app), COLOR_MARKED);
    app.process_events_once(&mut terminal, into_codes(" "))?;
    assert_eq!(
        marked_paths(&app),
        BTreeSet::from(candidates.clone()),
        "partial marks are completed, not inverted"
    );
    app.process_events_once(&mut terminal, into_codes(" "))?;
    let delete = [
        Event::Key(KeyCode::Tab.into()),
        Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
    ];
    for remaining in (1..candidates.len()).rev() {
        // Keep the direct siblings until last so their group persists down to one survivor.
        let target = hub(&app)
            .selected_members()
            .iter()
            .copied()
            .find(|&index| app.traversal.tree.path_of(index) == candidates[remaining])
            .unwrap();
        app.process_events_once(&mut terminal, into_codes("o"))?;
        app.state.navigation.select(Some(target));
        if remaining == candidates.len() - 1 {
            app.process_events_once(&mut terminal, into_codes(" /*"))?;
            app.process_events_once(&mut terminal, into_keys([KeyCode::Enter]))?;
            app.state.navigation_mut().select(Some(target));
            app.process_events_once(&mut terminal, into_codes("o"))?;
        } else {
            app.process_events_once(&mut terminal, into_codes(" u"))?;
        }
        app.process_events_once(&mut terminal, into_events(delete.clone()))?;
        if let Some(glob) = &app.state.glob_navigation {
            assert_eq!(
                glob.view_root, glob.tree_root,
                "deleting a viewed match returns to the scoped results"
            );
            assert_eq!(app.state.entries.len(), remaining);
            app.process_events_once(&mut terminal, into_codes("qu"))?;
        }
        let selected = hub(&app)
            .rows
            .iter()
            .find(|row| Some(row.entry.name.as_path()) == hub(&app).selected_path())
            .unwrap();
        assert!(
            selected.entry.name.starts_with(&project),
            "selection follows the group and its final survivor"
        );
        assert_eq!(selected.members.len(), remaining);
        assert_eq!(
            selected
                .members
                .iter()
                .map(|&index| app.traversal.tree.path_of(index))
                .collect::<BTreeSet<_>>(),
            candidates[..remaining].iter().cloned().collect()
        );
        assert!(
            hub(&app).rows.iter().any(|row| row.entry.name == singleton),
            "the unrelated row stays visible"
        );
    }
    app.process_events_once(&mut terminal, into_codes(" "))?;
    app.process_events_once(&mut terminal, into_events(delete))?;
    assert!(project.is_dir());
    assert_eq!(fs::read(project.join("source.py"))?, b"keep this source");
    assert!(candidates.iter().all(|path| !path.exists()));
    assert!(singleton.is_dir());
    assert_eq!(hub(&app).rows.len(), 1, "empty groups disappear");
    assert_eq!(hub(&app).rows[0].entry.name, singleton);
    Ok(())
}

#[test]
fn clean_group_refresh_revalidates_members_without_widening_scope() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().canonicalize()?;
    let project = root.join("project");
    let kept = project.join(".mypy_cache");
    let rejected = project.join(".pytest_cache");
    for path in [&kept, &rejected] {
        fs::create_dir_all(path)?;
        fs::write(path.join("cache"), b"cache")?;
    }
    fs::write(project.join("source.py"), b"source")?;
    let ignore = root.join("ignore");
    fs::write(&ignore, b"/project/.pytest_cache/excluded\n")?;
    let (mut terminal, mut app) = untraversed_app_and_terminal_with_closure(
        &[root.clone(), project.clone()],
        Path::to_path_buf,
    )?;
    app.state.walk_options.ignore_patterns = dua::IgnorePatterns::from_files(&[ignore])?;
    app.traverse_clean(None)?;
    app.process_events_once(&mut terminal, into_events([]))?;
    assert_eq!(hub(&app).rows.len(), 1);
    assert_eq!(hub(&app).rows[0].entry.name, project);

    fs::write(rejected.join("excluded"), b"keep")?;
    let added = project.join(".ruff_cache");
    fs::create_dir(&added)?;
    fs::write(added.join("cache"), b"new")?;
    app.process_events_once(&mut terminal, into_codes("xr"))?;
    assert!(app.window.mark.is_none(), "refresh clears stale marks");
    assert_eq!(hub(&app).rows.len(), 1);
    assert_eq!(
        hub(&app).rows[0].entry.name,
        kept,
        "a singleton is flattened"
    );
    assert_eq!(hub(&app).inputs, [root, project.clone()]);

    app.process_events_once(&mut terminal, into_codes("R"))?;
    assert_eq!(hub(&app).rows.len(), 1);
    assert_eq!(hub(&app).rows[0].entry.name, project);
    app.process_events_once(&mut terminal, into_codes("oR"))?;
    assert_eq!(
        entry_names(&app),
        BTreeSet::from([
            Path::new(".mypy_cache").to_owned(),
            Path::new(".ruff_cache").to_owned()
        ]),
    );
    assert_eq!(fs::read(project.join("source.py"))?, b"source");
    assert!(rejected.is_dir());
    fs::write(added.join(".git"), b"gitdir: elsewhere")?;
    app.process_events_once(&mut terminal, into_codes("/.ruff_cache"))?;
    app.process_events_once(&mut terminal, into_keys([KeyCode::Enter]))?;
    app.process_events_once(&mut terminal, into_codes("oR"))?;
    assert!(
        app.state.entries.is_empty(),
        "rejected glob roots leave no stale view"
    );
    app.process_events_once(&mut terminal, into_codes("q"))?;
    assert_eq!(
        entry_names(&app),
        BTreeSet::from([PathBuf::from(".mypy_cache")])
    );
    Ok(())
}

#[test]
fn clean_publishes_sized_candidates_and_refreshes_the_original_scope() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().canonicalize()?;
    let candidate = root.join("project/node_modules");
    fs::create_dir_all(candidate.join("__pycache__"))?;
    fs::write(candidate.join("data"), b"payload")?;
    fs::write(root.join("source"), b"keep")?;
    let expected_size = u128::from(candidate.metadata()?.len())
        + u128::from(candidate.join("__pycache__").metadata()?.len())
        + 7;

    let (mut terminal, mut app) =
        untraversed_app_and_terminal_with_closure(std::slice::from_ref(&root), Path::to_path_buf)?;
    app.traverse_clean(None)?;
    let (_keep_alive, events) = crossbeam::channel::bounded(0);
    while app.state.scan.is_some() {
        app.state.process_event(
            &mut app.window,
            &mut app.traversal,
            &mut app.display,
            &mut terminal,
            &events,
            &app.config,
        )?;
        for index in app.traversal.tree.children(app.traversal.root_index) {
            assert_eq!(
                app.traversal.tree.name(index).as_deref(),
                Some(candidate.as_path())
            );
            assert_eq!(app.traversal.tree.data(index).unwrap().size, expected_size);
        }
    }
    assert_eq!(hub(&app).rows.len(), 1);
    assert_eq!(hub(&app).rows[0].entry.name, candidate);
    assert_eq!(hub(&app).inputs, std::slice::from_ref(&root));
    assert!(app.state.root_path.is_none());

    app.process_events_once(&mut terminal, into_codes("U"))?;
    assert_eq!(hub(&app).rows.len(), 1, "parent scanning stays disabled");
    assert_eq!(hub(&app).inputs, std::slice::from_ref(&root));
    app.process_events_once(&mut terminal, into_codes("our"))?;
    assert_eq!(
        hub(&app).rows[0].entry.name,
        candidate,
        "refresh keeps the full root path"
    );

    let second = root.join("another/__pycache__");
    fs::create_dir_all(&second)?;
    fs::write(second.join("cache"), b"new")?;
    app.process_events_once(&mut terminal, into_codes("xR"))?;
    assert!(
        app.window.mark.is_none(),
        "refresh discards marks with recycled node indices"
    );
    assert_eq!(
        entry_names(&app),
        BTreeSet::from([candidate.clone(), second.clone()]),
    );
    assert_eq!(hub(&app).inputs, std::slice::from_ref(&root));

    select_hub_entry(&mut app, &mut terminal, &candidate)?;
    app.process_events_once(
        &mut terminal,
        into_events([
            Event::Key(KeyCode::Char('x').into()),
            Event::Key(KeyCode::Tab.into()),
            Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
        ]),
    )?;
    assert!(!candidate.exists());
    assert!(second.exists());
    assert_eq!(fs::read(root.join("source"))?, b"keep");
    Ok(())
}

#[test]
fn clean_refresh_handles_empty_results_depth_zero_and_rejected_candidates() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().canonicalize()?;
    let (mut terminal, mut app) =
        untraversed_app_and_terminal_with_closure(std::slice::from_ref(&root), Path::to_path_buf)?;
    app.traverse_clean(Some(1))?;
    app.process_events_once(&mut terminal, into_events([]))?;
    assert!(hub(&app).is_empty());
    assert_eq!(
        app.state.message.as_deref(),
        Some(app.state.language.ui_text().no_cleanup_candidates)
    );

    let candidate = root.join("node_modules");
    fs::create_dir(&candidate)?;
    fs::write(candidate.join("data"), b"cache")?;
    app.process_events_once(&mut terminal, into_codes("R"))?;
    assert_eq!(hub(&app).rows.len(), 1);

    let (mut direct_terminal, mut direct_app) = untraversed_app_and_terminal_with_closure(
        std::slice::from_ref(&candidate),
        Path::to_path_buf,
    )?;
    direct_app.traverse_clean(Some(0))?;
    direct_app.process_events_once(&mut direct_terminal, into_events([]))?;
    assert_eq!(hub(&direct_app).rows[0].entry.name, candidate);

    fs::write(candidate.join(".git"), b"gitdir: elsewhere")?;
    app.process_events_once(&mut terminal, into_codes("xr"))?;
    assert!(
        hub(&app).is_empty(),
        "selected refresh revalidates the candidate"
    );
    assert!(app.window.mark.is_none());
    assert!(hub(&app).selected_path().is_none());
    assert_eq!(hub(&app).inputs, std::slice::from_ref(&root));

    fs::create_dir(root.join("__pycache__"))?;
    app.process_events_once(&mut terminal, into_codes("R"))?;
    assert_eq!(hub(&app).rows.len(), 1);
    assert_eq!(hub(&app).rows[0].entry.name, root.join("__pycache__"));
    Ok(())
}

#[test]
fn clean_refresh_revalidates_the_owner_from_inside_a_candidate() -> Result<()> {
    for enter in ["o", "oo"] {
        for refresh in ["r", "R"] {
            let fixture = tempfile::tempdir()?;
            let root = fixture.path().canonicalize()?;
            let candidate = root.join("project/node_modules");
            fs::create_dir_all(candidate.join("nested"))?;
            fs::write(candidate.join("nested/cache"), b"cache")?;
            let (mut terminal, mut app) = untraversed_app_and_terminal_with_closure(
                std::slice::from_ref(&root),
                Path::to_path_buf,
            )?;
            app.traverse_clean(Some(2))?;
            app.process_events_once(&mut terminal, into_codes(enter))?;
            let view_path = crate::interactive::path_of(
                &app.traversal.tree,
                app.state.navigation.view_root,
                None,
            );

            app.process_events_once(&mut terminal, into_codes(refresh))?;
            assert_eq!(
                crate::interactive::path_of(
                    &app.traversal.tree,
                    app.state.navigation.view_root,
                    None,
                ),
                view_path,
                "a safe {enter}/{refresh} refresh restores the current directory"
            );

            // This lies outside the selected subtree when the view is inside `nested`.
            fs::create_dir(candidate.join(".git"))?;
            app.process_events_once(&mut terminal, into_codes("x"))?;
            app.process_events_once(&mut terminal, into_codes(refresh))?;
            assert!(
                app.traversal.clean_search_roots.is_empty(),
                "{enter}/{refresh} must reject the entire candidate"
            );
            assert!(app.state.entries.is_empty());
            assert!(app.window.mark.is_none());
            assert_eq!(fs::read(candidate.join("nested/cache"))?, b"cache");
        }
    }
    Ok(())
}

#[test]
fn clean_selected_refresh_preserves_ignore_pattern_roots() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().canonicalize()?;
    let candidate = root.join("project/node_modules");
    fs::create_dir_all(&candidate)?;
    fs::write(candidate.join("kept"), b"cache")?;
    let ignore = root.join("ignore");
    fs::write(&ignore, b"/project/node_modules/excluded\n")?;
    let (mut terminal, mut app) =
        untraversed_app_and_terminal_with_closure(std::slice::from_ref(&root), Path::to_path_buf)?;
    app.state.walk_options.ignore_patterns = dua::IgnorePatterns::from_files(&[ignore])?;
    app.traverse_clean(None)?;
    app.process_events_once(&mut terminal, into_events([]))?;
    assert_eq!(hub(&app).rows.len(), 1);
    fs::write(candidate.join("excluded"), b"ignored")?;
    app.process_events_once(&mut terminal, into_codes("r"))?;
    assert!(
        hub(&app).is_empty(),
        "refresh vetoes excluded descendants using the original pattern root"
    );
    Ok(())
}

#[test]
fn clean_refresh_preserves_discovery_roots_with_overlapping_inputs() -> Result<()> {
    for (depth, artifact, pattern) in [
        (None, "project/node_modules", "/node_modules/\n"),
        (Some(2), "project/node_modules", "/node_modules/\n"),
        (Some(1), "project/__pycache__", "/project/__pycache__/\n"),
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().canonicalize()?;
        let candidate = root.join(artifact);
        fs::create_dir_all(&candidate)?;
        fs::write(candidate.join("kept"), b"cache")?;
        let ignore = root.join("ignore");
        fs::write(&ignore, pattern)?;
        let (mut terminal, mut app) = untraversed_app_and_terminal_with_closure(
            &[root.clone(), root.join("project")],
            Path::to_path_buf,
        )?;
        app.state.walk_options.ignore_patterns = dua::IgnorePatterns::from_files(&[ignore])?;
        app.traverse_clean(depth)?;
        app.process_events_once(&mut terminal, into_events([]))?;
        assert_eq!(hub(&app).rows.len(), 1, "initial discovery at {depth:?}");
        for key in ["r", "R", "r"] {
            app.process_events_once(&mut terminal, into_codes(key))?;
            assert_eq!(hub(&app).rows.len(), 1, "{key} refresh at {depth:?}");
            assert_eq!(hub(&app).rows[0].entry.name, candidate);
        }
        app.process_events_once(&mut terminal, into_codes("o"))?;
        app.process_events_once(&mut terminal, into_codes("R"))?;
        assert_eq!(
            app.state.entries.len(),
            1,
            "refresh inside candidate at {depth:?}"
        );
        assert_eq!(app.state.entries[0].name, Path::new("kept"));
    }
    Ok(())
}
