use crate::interactive::{
    app::navigation::Navigation,
    sorted_entries,
    state::FocussedPane,
    widgets::{glob_search, MainWindow, MainWindowProps},
    CursorDirection, CursorMode, DisplayOptions, MarkEntryMode,
};
use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crosstermion::input::Event;
use dua::{
    traverse::{BackgroundTraversal, EntryData, Traversal, TreeIndex},
    WalkResult,
};
use std::path::PathBuf;
use tui::backend::Backend;
use tui_react::Terminal;

use super::state::{AppState, Cursor};
use super::tree_view::TreeView;

impl AppState {
    pub fn navigation_mut(&mut self) -> &mut Navigation {
        self.glob_navigation
            .as_mut()
            .unwrap_or(&mut self.navigation)
    }

    pub fn navigation(&self) -> &Navigation {
        self.glob_navigation.as_ref().unwrap_or(&self.navigation)
    }

    pub fn draw<B>(
        &mut self,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            current_path: tree_view.current_path(self.navigation().view_root),
            entries_traversed: self.stats.entries_traversed,
            total_bytes: self.stats.total_bytes,
            start: self.stats.start,
            elapsed: self.stats.elapsed,
            display,
            state: self,
        };

        let mut cursor = Cursor::default();
        let result = draw_window(window, props, terminal, &mut cursor);

        if cursor.show {
            _ = terminal.show_cursor();
            _ = terminal.set_cursor(cursor.x, cursor.y);
        } else {
            _ = terminal.hide_cursor();
        }

        result
    }

    pub fn traverse(&mut self, traversal: &Traversal, input: Vec<PathBuf>) -> Result<()> {
        let background_traversal =
            BackgroundTraversal::start(traversal.root_index, &self.walk_options, input, false)?;
        self.navigation_mut().view_root = traversal.root_index;
        self.active_traversal = Some(background_traversal);
        Ok(())
    }

    fn recompute_sizes_recursively(
        &mut self,
        traversal: &mut Traversal,
        node_index: TreeIndex)
    {
        let mut tree_view = self.tree_view(traversal);
        tree_view.recompute_sizes_recursively(node_index);
    }

    fn refresh_screen<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B: Backend,
    {
        let tree_view = self.tree_view(traversal);
        self.draw(window, &tree_view, *display, terminal)?;
        Ok(())
    }

    /// This method ends once the user quits the application or there are no more inputs to process.
    pub fn process_events<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: Receiver<Event>,
    ) -> Result<WalkResult>
    where
        B: Backend,
    {
        self.refresh_screen(window, traversal, display, terminal)?;

        loop {
            if let Some(result) =
                self.process_event(window, traversal, display, terminal, &events)?
            {
                return Ok(result);
            }
        }
    }

    pub fn process_event<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: &Receiver<Event>,
    ) -> Result<Option<WalkResult>>
    where
        B: Backend,
    {
        if let Some(active_traversal) = &mut self.active_traversal {
            crossbeam::select! {
                recv(events) -> event => {
                    let Ok(event) = event else {
                        return Ok(Some(WalkResult { num_errors: self.stats.io_errors }));
                    };
                    let res = self.process_terminal_event(
                        window,
                        traversal,
                        display,
                        terminal,
                        event)?;
                    if let Some(res) = res {
                        return Ok(Some(res));
                    }
                },
                recv(&active_traversal.event_rx) -> event => {
                    let Ok(event) = event else {
                        return Ok(None);
                    };

                    if let Some(is_finished) = active_traversal.integrate_traversal_event(traversal, event) {
                        if is_finished {
                            let root_index = active_traversal.root_idx;
                            self.recompute_sizes_recursively(traversal, root_index);
                            self.active_traversal = None;
                        }
                        self.update_state(traversal);
                        self.refresh_screen(window, traversal, display, terminal)?;
                    };
                }
            }
        } else {
            let Ok(event) = events.recv() else {
                return Ok(Some(WalkResult { num_errors: self.stats.io_errors }));
            };
            let result =
                self.process_terminal_event(window, traversal, display, terminal, event)?;
            if let Some(processing_result) = result {
                return Ok(Some(processing_result));
            }
        }
        Ok(None)
    }

    fn update_state(&mut self, traversal: &Traversal) {
        self.entries = sorted_entries(
            &traversal.tree,
            self.navigation().view_root,
            self.sorting,
            self.glob_root(),
        );
        if !self.received_events {
            self.navigation_mut().selected = self.entries.first().map(|b| b.index);
        }
        self.reset_message(); // force "scanning" to appear
    }

    fn process_terminal_event<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        event: Event,
    ) -> Result<Option<WalkResult>>
    where
        B: Backend,
    {
        use crosstermion::crossterm::event::KeyCode::*;
        use FocussedPane::*;

        let key = match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key != refresh_key() {
                    self.received_events = true;
                }
                key
            }
            Event::Resize(_, _) => refresh_key(),
            _ => return Ok(None),
        };

        self.reset_message();

        let glob_focussed = self.focussed == Glob;
        let mut tree_view = self.tree_view(traversal);
        let mut handled = true;
        match key.code {
            Esc => {
                if let Some(value) = self.handle_quit(&mut tree_view, window) {
                    return Ok(Some(value?));
                }
            }
            Tab => {
                self.cycle_focus(window);
            }
            Char('/') if !glob_focussed => {
                self.toggle_glob_search(window);
            }
            Char('?') if !glob_focussed => self.toggle_help_pane(window),
            Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) && !glob_focussed => {
                return Ok(Some(WalkResult {
                    num_errors: self.stats.io_errors,
                }))
            }
            Char('q') if !glob_focussed => {
                if let Some(result) = self.handle_quit(&mut tree_view, window) {
                    return Ok(Some(result?));
                }
            }
            _ => {
                handled = false;
            }
        }

        if !handled {
            match self.focussed {
                Mark => self.dispatch_to_mark_pane(key, window, &mut tree_view, *display, terminal),
                Help => {
                    window
                        .help_pane
                        .as_mut()
                        .expect("help pane")
                        .process_events(key);
                }
                Glob => {
                    let glob_pane = window.glob_pane.as_mut().expect("glob pane");
                    match key.code {
                        Enter => self.search_glob_pattern(&mut tree_view, &glob_pane.input),
                        _ => glob_pane.process_events(key),
                    }
                }
                Main => match key.code {
                    Char('O') => self.open_that(&tree_view),
                    Char(' ') => self.mark_entry(
                        CursorMode::KeepPosition,
                        MarkEntryMode::Toggle,
                        window,
                        &tree_view,
                    ),
                    Char('x') => self.mark_entry(
                        CursorMode::Advance,
                        MarkEntryMode::MarkForDeletion,
                        window,
                        &tree_view,
                    ),
                    Char('a') => self.mark_all_entries(MarkEntryMode::Toggle, window, &tree_view),
                    Char('o') | Char('l') | Enter | Right => {
                        self.enter_node_with_traversal(&tree_view)
                    }
                    Char('R') => self.refresh(&mut tree_view, Refresh::AllInView)?,
                    Char('r') => self.refresh(&mut tree_view, Refresh::Selected)?,
                    Char('H') | Home => self.change_entry_selection(CursorDirection::ToTop),
                    Char('G') | End => self.change_entry_selection(CursorDirection::ToBottom),
                    PageUp => self.change_entry_selection(CursorDirection::PageUp),
                    Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.change_entry_selection(CursorDirection::PageUp)
                    }
                    Char('k') | Up => self.change_entry_selection(CursorDirection::Up),
                    Char('j') | Down => self.change_entry_selection(CursorDirection::Down),
                    PageDown => self.change_entry_selection(CursorDirection::PageDown),
                    Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.change_entry_selection(CursorDirection::PageDown)
                    }
                    Char('s') => self.cycle_sorting(&tree_view),
                    Char('m') => self.cycle_mtime_sorting(&tree_view),
                    Char('M') => self.toggle_mtime_column(),
                    Char('c') => self.cycle_count_sorting(&tree_view),
                    Char('C') => self.toggle_count_column(),
                    Char('g') | Char('S') => display.byte_vis.cycle(),
                    Char('d') => self.mark_entry(
                        CursorMode::Advance,
                        MarkEntryMode::Toggle,
                        window,
                        &tree_view,
                    ),
                    Char('u') | Char('h') | Backspace | Left => {
                        self.exit_node_with_traversal(&tree_view)
                    }
                    _ => {}
                },
            };
        }
        self.draw(window, &tree_view, *display, terminal)?;

        Ok(None)
    }

    fn refresh(&mut self, tree: &mut TreeView<'_>, what: Refresh) -> anyhow::Result<()> {
        // TODO: we should refresh parent_idx not selected index
        match what {
            Refresh::Selected => {
                let mut path = tree.path_of(self.navigation().view_root);
                if path.to_str().unwrap() == "" {
                    path = PathBuf::from(".");
                }
                log::info!("Refreshing {:?}", path);

                let entries_deleted = tree.remove_entries(self.navigation().view_root, false);
                log::info!("Deleted {entries_deleted} entries");
                
                tree.recompute_sizes_recursively(self.navigation().view_root);
                self.entries = tree.sorted_entries(self.navigation().view_root, self.sorting);
                self.navigation_mut().selected = self.entries.first().map(|e| e.index);
                
                self.active_traversal = Some(BackgroundTraversal::start(
                    self.navigation().view_root,
                    &self.walk_options,
                    vec![path],
                    true,
                )?);

                self.received_events = false;
            }
            Refresh::AllInView => {
                log::info!("Not implemented")
            }
        }
        Ok(())
    }

    fn tree_view<'a>(&mut self, traversal: &'a mut Traversal) -> TreeView<'a> {
        TreeView {
            traversal,
            glob_tree_root: self.glob_navigation.as_ref().map(|n| n.tree_root),
        }
    }

    fn search_glob_pattern(&mut self, tree_view: &mut TreeView<'_>, glob_pattern: &str) {
        use FocussedPane::*;
        match glob_search(tree_view.tree(), self.navigation.view_root, glob_pattern) {
            Ok(matches) if matches.is_empty() => {
                self.message = Some("No match found".into());
            }
            Ok(matches) => {
                if let Some(glob_source) = &self.glob_navigation {
                    tree_view.tree_mut().remove_node(glob_source.tree_root);
                }

                let tree_root = tree_view.tree_mut().add_node(EntryData::default());
                let glob_source = Navigation {
                    tree_root,
                    view_root: tree_root,
                    selected: Some(tree_root),
                    ..Default::default()
                };
                self.glob_navigation = Some(glob_source);

                for idx in matches {
                    tree_view.tree_mut().add_edge(tree_root, idx, ());
                }

                let glob_tree_view = TreeView {
                    traversal: tree_view.traversal,
                    glob_tree_root: Some(tree_root),
                };
                let new_entries = glob_tree_view.sorted_entries(tree_root, self.sorting);

                let new_entries = self
                    .navigation_mut()
                    .selected
                    .map(|previously_selected| (previously_selected, new_entries));

                self.enter_node(new_entries);
                self.focussed = Main;
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    fn handle_quit(
        &mut self,
        tree_view: &mut TreeView<'_>,
        window: &mut MainWindow,
    ) -> Option<std::result::Result<WalkResult, anyhow::Error>> {
        use FocussedPane::*;
        match self.focussed {
            Main => {
                if self.glob_navigation.is_some() {
                    self.handle_glob_quit(tree_view, window);
                } else {
                    return Some(Ok(WalkResult {
                        num_errors: self.stats.io_errors,
                    }));
                }
            }
            Mark => self.focussed = Main,
            Help => {
                self.focussed = Main;
                window.help_pane = None
            }
            Glob => {
                self.handle_glob_quit(tree_view, window);
            }
        }
        None
    }

    fn handle_glob_quit(&mut self, tree_view: &mut TreeView<'_>, window: &mut MainWindow) {
        use FocussedPane::*;
        self.focussed = Main;
        if let Some(glob_source) = &self.glob_navigation {
            tree_view.tree_mut().remove_node(glob_source.tree_root);
        }
        self.glob_navigation = None;
        window.glob_pane = None;

        tree_view.glob_tree_root.take();
        self.entries = tree_view.sorted_entries(self.navigation().view_root, self.sorting);
    }
}

enum Refresh {
    /// Refresh the directory currently in view
    AllInView,
    /// Refresh only the selected item
    Selected,
}

pub fn draw_window<B>(
    window: &mut MainWindow,
    props: MainWindowProps<'_>,
    terminal: &mut Terminal<B>,
    cursor: &mut Cursor,
) -> Result<()>
where
    B: Backend,
{
    let area = terminal.pre_render()?;
    window.render(props, area, terminal, cursor);
    terminal.post_render()?;

    Ok(())
}

pub fn refresh_key() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::ALT)
}
