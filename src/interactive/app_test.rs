use crate::interactive::{SortMode, TerminalApp};
use dua::{
    traverse::{EntryData, Tree, TreeIndex},
    ByteFormat, Color, TraversalSorting, WalkOptions,
};
use failure::Error;
use jwalk::{DirEntry, WalkDir};
use petgraph::prelude::NodeIndex;
use pretty_assertions::assert_eq;
use std::{
    env::temp_dir,
    ffi::OsStr,
    ffi::OsString,
    fmt,
    fs::{copy, create_dir_all, remove_dir, remove_file},
    io,
    io::ErrorKind,
    path::Path,
    path::PathBuf,
};
use termion::input::TermRead;
use tui::backend::TestBackend;
use tui_react::Terminal;

const FIXTURE_PATH: &'static str = "tests/fixtures";

fn debug(item: impl fmt::Debug) -> String {
    format!("{:?}", item)
}

#[test]
fn it_can_handle_ending_traversal_reaching_top_but_skipping_levels() -> Result<(), Error> {
    let (_, app) = initialized_app_and_terminal_from_fixture(&["sample-01"])?;
    let expected_tree = sample_01_tree();

    assert_eq!(
        debug(app.traversal.tree),
        debug(expected_tree),
        "filesystem graph is stable and matches the directory structure"
    );
    Ok(())
}

#[test]
fn it_can_handle_ending_traversal_without_reaching_the_top() -> Result<(), Error> {
    let (_, app) = initialized_app_and_terminal_from_fixture(&["sample-02"])?;
    let expected_tree = sample_02_tree();

    assert_eq!(
        debug(app.traversal.tree),
        debug(expected_tree),
        "filesystem graph is stable and matches the directory structure"
    );
    Ok(())
}

fn node_by_index(app: &TerminalApp, id: TreeIndex) -> &EntryData {
    app.traversal.tree.node_weight(id).unwrap()
}

fn node_by_name(app: &TerminalApp, name: impl AsRef<OsStr>) -> &EntryData {
    node_by_index(app, index_by_name(&app, name))
}

fn index_by_name_and_size(
    app: &TerminalApp,
    name: impl AsRef<OsStr>,
    size: Option<u64>,
) -> TreeIndex {
    let name = name.as_ref();
    let t: Vec<_> = app
        .traversal
        .tree
        .node_indices()
        .map(|idx| (idx, node_by_index(app, idx)))
        .filter_map(|(idx, e)| {
            if e.name == name && size.map(|s| s == e.size).unwrap_or(true) {
                Some(idx)
            } else {
                None
            }
        })
        .collect();
    match t.len() {
        1 => t[0],
        0 => panic!("Node named '{}' not found in tree", name.to_string_lossy()),
        n => panic!("Node named '{}' found {} times", name.to_string_lossy(), n),
    }
}

fn index_by_name(app: &TerminalApp, name: impl AsRef<OsStr>) -> TreeIndex {
    index_by_name_and_size(app, name, None)
}

#[test]
fn simple_user_journey_read_only() -> Result<(), Error> {
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
            "it will sort entries in descending order by size"
        );

        let first_selected_path = OsString::from(format!("{}/{}", FIXTURE_PATH, long_root));
        assert_eq!(
            node_by_name(&app, &first_selected_path).name,
            first_selected_path,
            "the roots are always listed with the given (possibly long) names",
        );

        assert_eq!(
            node_by_name(&app, fixture_str(short_root)),
            node_by_index(&app, *app.state.selected.as_ref().unwrap()),
            "it selects the first node in the list",
        );

        assert_eq!(
            app.traversal.root_index, app.state.root,
            "the root is the 'virtual' root",
        );
    }

    // SORTING
    {
        // when hitting the S key
        app.process_events(&mut terminal, b"s".keys())?;
        assert_eq!(
            app.state.sorting,
            SortMode::SizeAscending,
            "it sets the sort mode to ascending by size"
        );
        assert_eq!(
            node_by_index(&app, app.state.entries[0].index),
            node_by_name(&app, fixture_str(long_root)),
            "it recomputes the cached entries"
        );
        // when hitting the S key again
        app.process_events(&mut terminal, b"s".keys())?;
        assert_eq!(
            app.state.sorting,
            SortMode::SizeDescending,
            "it sets the sort mode to descending by size"
        );
        assert_eq!(
            node_by_index(&app, app.state.entries[0].index),
            node_by_name(&app, fixture_str(short_root)),
            "it recomputes the cached entries"
        );
    }

    // Entry-Navigation
    {
        // when hitting the j key
        app.process_events(&mut terminal, b"j".keys())?;
        assert_eq!(
            node_by_name(&app, fixture_str(long_root)),
            node_by_index(&app, *app.state.selected.as_ref().unwrap()),
            "it moves the cursor down and selects the next entry based on the current sort mode"
        );
        // when hitting it while there is nowhere to go
        app.process_events(&mut terminal, b"j".keys())?;
        assert_eq!(
            node_by_name(&app, fixture_str(long_root)),
            node_by_index(&app, *app.state.selected.as_ref().unwrap()),
            "it stays at the previous position"
        );
        // when hitting the k key
        app.process_events(&mut terminal, b"k".keys())?;
        assert_eq!(
            node_by_name(&app, fixture_str(short_root)),
            node_by_index(&app, *app.state.selected.as_ref().unwrap()),
            "it moves the cursor up and selects the next entry based on the current sort mode"
        );
        // when hitting the k key again
        app.process_events(&mut terminal, b"k".keys())?;
        assert_eq!(
            node_by_name(&app, fixture_str(short_root)),
            node_by_index(&app, *app.state.selected.as_ref().unwrap()),
            "it stays at the current cursor position as there is nowhere to go"
        );
        // when hitting the o key with a directory selected
        app.process_events(&mut terminal, b"o".keys())?;
        {
            let new_root_idx = index_by_name(&app, fixture_str(short_root));
            assert_eq!(
                new_root_idx, app.state.root,
                "it enters the entry if it is a directory, changing the root"
            );
            assert_eq!(
                index_by_name(&app, "dir"),
                *app.state.selected.as_ref().unwrap(),
                "it selects the first entry in the directory"
            );

            // when hitting the u key while inside a sub-directory
            app.process_events(&mut terminal, b"u".keys())?;
            {
                assert_eq!(
                    app.traversal.root_index, app.state.root,
                    "it sets the root to be the (roots) parent directory, being the virtual root"
                );
                assert_eq!(
                    node_by_name(&app, fixture_str(short_root)),
                    node_by_index(&app, *app.state.selected.as_ref().unwrap()),
                    "changes the selection to the first item in the list of entries"
                );
            }
        }
        // when hitting the u key while inside of the root directory
        // We are moving the cursor down just to have a non-default selection
        app.process_events(&mut terminal, b"ju".keys())?;
        {
            assert_eq!(
                app.traversal.root_index, app.state.root,
                "it keeps the root - it can't go further up"
            );
            assert_eq!(
                node_by_name(&app, fixture_str(long_root)),
                node_by_index(&app, *app.state.selected.as_ref().unwrap()),
                "keeps the previous selection"
            );
        }
    }

    // Deletion
    {
        // when hitting the 'd' key (also move cursor back to start)
        app.process_events(&mut terminal, b"k".keys())?;
        let previously_selected_index = *app.state.selected.as_ref().unwrap();
        app.process_events(&mut terminal, b"d".keys())?;
        {
            assert_eq!(
                Some(1),
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
                "it marks only a single node",
            );
            assert!(
                app.window.mark_pane.as_ref().map_or(false, |p| p
                    .marked()
                    .contains_key(&previously_selected_index)),
                "it marks the selected node"
            );
            assert_eq!(
                app.state.selected.as_ref().unwrap().index(),
                app.state.entries[1].index.index(),
                "moves the cursor down one level to facilitate many markings in a row"
            );
        }

        // when hitting the 'd' key again
        {
            app.process_events(&mut terminal, b"d".keys())?;

            assert_eq!(
                Some(2),
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
                "it marks the currently selected, second node",
            );

            assert_eq!(
                app.state.selected.as_ref().unwrap().index(),
                app.state.entries[1].index.index(),
                "it could not advance the cursor, thus the newly marked item is still selected"
            );
        }

        // when hitting the 'd' key once again
        {
            app.process_events(&mut terminal, b"d".keys())?;

            assert_eq!(
                Some(1),
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
                "it toggled the previous selected entry off",
            );

            assert!(
                app.window.mark_pane.as_ref().map_or(false, |p| p
                    .marked()
                    .contains_key(&previously_selected_index)),
                "it leaves the first selected entry marked"
            );
        }
        // when hitting the spacebar (after moving up to the first entry)
        {
            app.process_events(&mut terminal, b"k ".keys())?;

            assert_eq!(
                None,
                app.window.mark_pane.as_ref().map(|p| p.marked().len()),
                "it toggles the item off",
            );

            assert_eq!(
                node_by_index(&app, previously_selected_index),
                node_by_index(&app, *app.state.selected.as_ref().unwrap()),
                "it does not advance the selection"
            );
        }
    }

    // Marking
    {
        // select something
        app.process_events(&mut terminal, b" j ".keys())?;
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
        app.process_events(&mut terminal, b"\t".keys())?;
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

struct WritableFixture {
    root: PathBuf,
}

impl Drop for WritableFixture {
    fn drop(&mut self) {
        delete_recursive(&self.root).unwrap();
    }
}

fn delete_recursive(path: impl AsRef<Path>) -> Result<(), io::Error> {
    let mut files: Vec<_> = Vec::new();
    let mut dirs: Vec<_> = Vec::new();

    for entry in WalkDir::new(&path).num_threads(1).into_iter() {
        let entry: DirEntry = entry.unwrap();
        let p = entry.path();
        match p.is_dir() {
            true => dirs.push(p),
            false => files.push(p),
        }
    }

    files
        .iter()
        .map(|f| remove_file(f))
        .chain(dirs.iter().map(|d| remove_dir(d)))
        .collect::<Result<_, _>>()
}

fn copy_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), Error> {
    for entry in WalkDir::new(&src).num_threads(1).into_iter() {
        let entry: DirEntry = entry?;
        let entry_path = entry.path();
        entry_path
            .strip_prefix(&src)
            .map_err(Error::from)
            .and_then(|relative_entry_path| {
                let dst = dst.as_ref().join(relative_entry_path);
                if entry_path.is_dir() {
                    create_dir_all(dst).map_err(Into::into)
                } else {
                    copy(&entry_path, dst)
                        .map(|_| ())
                        .or_else(|e| {
                            if let ErrorKind::AlreadyExists = e.kind() {
                                Ok(())
                            } else {
                                Err(e)
                            }
                        })
                        .map_err(Into::into)
                }
            })?;
    }
    Ok(())
}

impl From<&'static str> for WritableFixture {
    fn from(fixture_name: &str) -> Self {
        const TEMP_TLD_DIRNAME: &'static str = "dua-unit";

        let src = fixture(fixture_name);
        let dst = temp_dir().join(TEMP_TLD_DIRNAME);
        create_dir_all(&dst).unwrap();

        let dst = dst.join(fixture_name);
        copy_recursive(src, &dst).unwrap();
        WritableFixture { root: dst }
    }
}

#[test]
fn basic_user_journey_with_deletion() -> Result<(), Error> {
    let fixture = WritableFixture::from("sample-02");
    let (mut _terminal, mut _app) =
        initialized_app_and_terminal_from_paths(&[fixture.root.clone()])?;
    Ok(())
}

fn fixture(p: impl AsRef<Path>) -> PathBuf {
    Path::new(FIXTURE_PATH).join(p)
}

fn fixture_str(p: impl AsRef<Path>) -> String {
    fixture(p).to_str().unwrap().to_owned()
}

fn initialized_app_and_terminal_with_closure<P: AsRef<Path>>(
    fixture_paths: &[P],
    mut convert: impl FnMut(&Path) -> PathBuf,
) -> Result<(Terminal<TestBackend>, TerminalApp), Error> {
    let mut terminal = Terminal::new(TestBackend::new(40, 20))?;
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))?;

    let input = fixture_paths.iter().map(|c| convert(c.as_ref())).collect();
    let app = TerminalApp::initialize(
        &mut terminal,
        WalkOptions {
            threads: 1,
            byte_format: ByteFormat::Metric,
            color: Color::None,
            sorting: TraversalSorting::AlphabeticalByFileName,
        },
        input,
    )?;
    Ok((terminal, app))
}
fn initialized_app_and_terminal_from_paths(
    fixture_paths: &[PathBuf],
) -> Result<(Terminal<TestBackend>, TerminalApp), Error> {
    fn to_path_buf(p: &Path) -> PathBuf {
        p.to_path_buf()
    }
    initialized_app_and_terminal_with_closure(fixture_paths, to_path_buf)
}
fn initialized_app_and_terminal_from_fixture(
    fixture_paths: &[&str],
) -> Result<(Terminal<TestBackend>, TerminalApp), Error> {
    initialized_app_and_terminal_with_closure(fixture_paths, |p| fixture(p))
}

fn sample_01_tree() -> Tree {
    let mut t = Tree::new();
    {
        let mut add_node = make_add_node(&mut t);
        let root_size = 1259070;
        let r = add_node("", root_size, None);
        {
            let s = add_node(&fixture_str("sample-01"), root_size, Some(r));
            {
                add_node(".hidden.666", 666, Some(s));
                add_node("a", 256, Some(s));
                add_node("b.empty", 0, Some(s));
                add_node("c.lnk", 1, Some(s));
                let d = add_node("dir", 1258024, Some(s));
                {
                    add_node("1000bytes", 1000, Some(d));
                    add_node("dir-a.1mb", 1_000_000, Some(d));
                    add_node("dir-a.kb", 1024, Some(d));
                    let e = add_node("empty-dir", 0, Some(d));
                    {
                        add_node(".gitkeep", 0, Some(e));
                    }
                    let sub = add_node("sub", 256_000, Some(d));
                    {
                        add_node("dir-sub-a.256kb", 256_000, Some(sub));
                    }
                }
                add_node("z123.b", 123, Some(s));
            }
        }
    }
    t
}

fn sample_02_tree() -> Tree {
    let mut t = Tree::new();
    {
        let mut add_node = make_add_node(&mut t);
        let root_size = 1540;
        let r = add_node("", root_size, None);
        {
            let s = add_node(
                format!("{}/{}", FIXTURE_PATH, "sample-02").as_str(),
                root_size,
                Some(r),
            );
            {
                add_node("a", 256, Some(s));
                add_node("b", 1, Some(s));
                let d = add_node("dir", 1283, Some(s));
                {
                    add_node("c", 257, Some(d));
                    add_node("d", 2, Some(d));
                    let e = add_node("empty-dir", 0, Some(d));
                    {
                        add_node(".gitkeep", 0, Some(e));
                    }
                    let sub = add_node("sub", 1024, Some(d));
                    {
                        add_node("e", 1024, Some(sub));
                    }
                }
            }
        }
    }
    t
}

fn make_add_node<'a>(
    t: &'a mut Tree,
) -> impl FnMut(&str, u64, Option<NodeIndex>) -> NodeIndex + 'a {
    move |name, size, maybe_from_idx| {
        let n = t.add_node(EntryData {
            name: OsString::from(name),
            size,
            metadata_io_error: false,
        });
        if let Some(from) = maybe_from_idx {
            t.add_edge(from, n, ());
        }
        n
    }
}
