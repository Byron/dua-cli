use crate::interactive::state::FilesystemScan;
use crate::interactive::{
    CursorDirection, CursorMode, DisplayOptions, EntryCheck, MarkEntryMode,
    app::navigation::Navigation,
    state::FocussedPane,
    widgets::{MainWindow, MainWindowProps, glob_search},
};
use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crosstermion::input::Event;
use dua::{
    WalkResult,
    traverse::{BackgroundTraversal, EntryData, Traversal, TreeIndex},
};
use std::path::PathBuf;
use tui::{Terminal, backend::Backend, buffer::Buffer, layout::Rect, widgets::Widget};

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
            total_bytes: tree_view.total_size(),
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

    pub fn traverse(&mut self, traversal: &Traversal) -> Result<()> {
        let bg_traversal = BackgroundTraversal::start(
            traversal.root_index,
            &self.walk_options,
            self.root_paths.clone(),
            false,
            true,
        )?;
        self.navigation_mut().view_root = traversal.root_index;
        self.scan = Some(FilesystemScan {
            active_traversal: bg_traversal,
            previous_selection: None,
        });
        Ok(())
    }

    fn recompute_sizes_recursively(&mut self, traversal: &mut Traversal, node_index: TreeIndex) {
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
        if let Some(FilesystemScan {
            active_traversal,
            previous_selection,
        }) = self.scan.as_mut()
        {
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
                        self.stats = active_traversal.stats;
                        let previous_selection = previous_selection.clone();
                        if is_finished {
                            let root_index = active_traversal.root_idx;
                            self.recompute_sizes_recursively(traversal, root_index);
                            self.scan = None;
                            traversal.cost = Some(traversal.start_time.elapsed());
                        }
                        self.update_state_during_traversal(traversal, previous_selection.as_ref(), is_finished);
                        self.refresh_screen(window, traversal, display, terminal)?;
                    };
                }
            }
        } else {
            let Ok(event) = events.recv() else {
                return Ok(Some(WalkResult {
                    num_errors: self.stats.io_errors,
                }));
            };
            let result =
                self.process_terminal_event(window, traversal, display, terminal, event)?;
            if let Some(processing_result) = result {
                return Ok(Some(processing_result));
            }
        }
        Ok(None)
    }

    fn update_state_during_traversal(
        &mut self,
        traversal: &mut Traversal,
        previous_selection: Option<&(PathBuf, usize)>,
        is_finished: bool,
    ) {
        let tree_view = self.tree_view(traversal);
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );

        if !self.received_events {
            let previously_selected_entry =
                previous_selection.and_then(|(selected_name, selected_idx)| {
                    self.entries
                        .iter()
                        .find(|e| e.name == *selected_name)
                        .or_else(|| self.entries.get(*selected_idx))
                });
            if let Some(selected_entry) = previously_selected_entry {
                self.navigation_mut().selected = Some(selected_entry.index);
            } else if is_finished {
                self.navigation_mut().selected = self.entries.first().map(|b| b.index);
            }
        }
        self.reset_message(); // force "scanning" to appear
    }

    pub(crate) fn entry_check(&self) -> EntryCheck {
        EntryCheck::new(self.scan.is_some(), self.allow_entry_check)
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
        use FocussedPane::*;
        use crosstermion::crossterm::event::KeyCode::*;

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

        match (key.code, glob_focussed) {
            (Esc, _) | (Char('q'), false) => {
                if let Some(result) = self.handle_quit(&mut tree_view, window) {
                    return Ok(Some(result?));
                }
            }
            _ => {
                // Reset pending exit state when other keys are pressed.
                self.pending_exit = false;
            }
        }

        let mut handled = true;
        match key.code {
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
                }));
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
                        Enter => self.search_glob_pattern(
                            &mut tree_view,
                            &glob_pane.input,
                            glob_pane.case,
                        ),
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
                    Char('r') => self.refresh(&mut tree_view, window, Refresh::Selected)?,
                    Char('R') => self.refresh(&mut tree_view, window, Refresh::AllInView)?,
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
                    Char('n') => self.cycle_name_sorting(&tree_view),
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

    fn refresh(
        &mut self,
        tree: &mut TreeView<'_>,
        window: &mut MainWindow,
        what: Refresh,
    ) -> anyhow::Result<()> {
        // If another traversal is already running do not do anything.
        if self.scan.is_some() {
            self.message = Some("Traversal already running".into());
            return Ok(());
        }

        let previous_selection = self.navigation().selected.and_then(|sel_index| {
            tree.tree().node_weight(sel_index).map(|w| {
                (
                    w.name.clone(),
                    self.entries
                        .iter()
                        .enumerate()
                        .find_map(|(idx, e)| (e.index == sel_index).then_some(idx))
                        .expect("selected item is always in entries"),
                )
            })
        });

        // If we are displaying the root of the glob search results then cancel the search.
        if let Some(glob_tree_root) = tree.glob_tree_root
            && glob_tree_root == self.navigation().view_root
        {
            self.quit_glob_mode(tree, window)
        }

        let (paths, remove_root_node, skip_root, use_root_path, index, parent_index) = match what {
            Refresh::Selected => {
                let Some(selected) = self.navigation().selected else {
                    return Ok(());
                };
                let parent_index = tree
                    .fs_parent_of(selected)
                    .expect("there is always a parent to a selection");

                let mut path = tree.path_of(selected);
                if path.to_str() == Some("") {
                    path = PathBuf::from(".");
                }

                let (paths, use_root_path, skip_root) = if self.navigation().view_root
                    == tree.traversal.root_index
                    && self.root_paths.len() > 1
                {
                    (vec![path], true, false)
                } else {
                    (vec![path], false, false)
                };

                (
                    paths,
                    true,
                    skip_root,
                    use_root_path,
                    selected,
                    parent_index,
                )
            }
            Refresh::AllInView => {
                let (paths, use_root_path, skip_root) = if self.navigation().view_root
                    == tree.traversal.root_index
                    && self.root_paths.len() > 1
                {
                    (self.root_paths.clone(), true, false)
                } else {
                    let mut path = tree.path_of(self.navigation().view_root);
                    if path.to_str() == Some("") {
                        path = PathBuf::from(".");
                    }
                    (vec![path], false, true)
                };

                (
                    paths,
                    false,
                    skip_root,
                    use_root_path,
                    self.navigation().view_root,
                    self.navigation().view_root,
                )
            }
        };

        tree.remove_entries(index, remove_root_node);
        tree.recompute_sizes_recursively(parent_index);

        self.entries = tree.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
        self.navigation_mut().selected = self.entries.first().map(|e| e.index);

        self.scan = Some(FilesystemScan {
            active_traversal: BackgroundTraversal::start(
                parent_index,
                &self.walk_options,
                paths,
                skip_root,
                use_root_path,
            )?,
            previous_selection,
        });

        self.received_events = false;
        Ok(())
    }

    fn tree_view<'a>(&mut self, traversal: &'a mut Traversal) -> TreeView<'a> {
        TreeView {
            traversal,
            glob_tree_root: self.glob_navigation.as_ref().map(|n| n.tree_root),
        }
    }

    fn search_glob_pattern(
        &mut self,
        tree_view: &mut TreeView<'_>,
        glob_pattern: &str,
        case: gix_glob::pattern::Case,
    ) {
        use FocussedPane::*;
        match glob_search(
            tree_view.tree(),
            self.navigation.view_root,
            glob_pattern,
            case,
        ) {
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
                let new_entries =
                    glob_tree_view.sorted_entries(tree_root, self.sorting, self.entry_check());

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
                    self.quit_glob_mode(tree_view, window);
                } else if window.mark_pane.is_none() && !tree_view.traversal.is_costly() {
                    // If nothing is selected for deletion, quit instantly
                    return Some(Ok(WalkResult {
                        num_errors: self.stats.io_errors,
                    }));
                } else if !self.pending_exit {
                    self.pending_exit = true;
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
                self.quit_glob_mode(tree_view, window);
            }
        }
        None
    }

    fn quit_glob_mode(&mut self, tree_view: &mut TreeView<'_>, window: &mut MainWindow) {
        use FocussedPane::*;
        self.focussed = Main;
        if let Some(glob_source) = &self.glob_navigation {
            tree_view.tree_mut().remove_node(glob_source.tree_root);
        }
        self.glob_navigation = None;
        window.glob_pane = None;

        tree_view.glob_tree_root.take();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
    }
}

enum Refresh {
    /// Refresh the directory currently in view
    AllInView,
    /// Refresh only the selected item
    Selected,
}

/// A [`Widget`] that renders by calling a function.
///
/// The `FunctionWidget` struct holds a function that renders into a portion of
/// a [`Buffer`] designated by a [`Rect`].
///
/// This widget can be used to create custom UI elements that are defined by a
/// rendering function. and allows for rendering functions that do not implement
/// the [`Widget`] trait.
struct FunctionWidget<F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    render: F,
}

impl<F> FunctionWidget<F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    /// Creates a new [`FunctionWidget`] with the given rendering function.
    ///
    /// The rendering function must have the signature `FnOnce(Rect, &mut
    /// Buffer)`, where:
    /// - [`Rect`] represents the available space for rendering.
    /// - [`Buffer`] is the buffer to write the rendered content to.
    ///
    /// The `FunctionWidget` can then be used to render the provided function in
    /// a user interface.
    fn new(function: F) -> FunctionWidget<F>
    where
        F: FnOnce(Rect, &mut Buffer),
    {
        FunctionWidget { render: function }
    }
}

/// Implements the [`Widget`] trait for [`FunctionWidget`].
///
/// The implementation simply calls the provided render function with the given
/// `Rect` and `Buffer`.
impl<F> Widget for FunctionWidget<F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        (self.render)(area, buf);
    }
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
    terminal.draw(|frame| {
        frame.render_widget(
            FunctionWidget::new(|area, buf| {
                window.render(props, area, buf, cursor);
            }),
            frame.size(),
        );
    })?;
    Ok(())
}

pub fn refresh_key() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::ALT)
}
