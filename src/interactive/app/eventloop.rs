use crate::interactive::{
    app::navigation::Navigation,
    sorted_entries,
    widgets::{glob_search, MainWindow, MainWindowProps},
    ByteVisualization, CursorDirection, CursorMode, DisplayOptions, EntryDataBundle, MarkEntryMode,
    SortMode,
};
use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crosstermion::input::Event;
use dua::{
    traverse::{EntryData, Traversal},
    WalkOptions, WalkResult,
};
use std::path::PathBuf;
use tui::backend::Backend;
use tui_react::Terminal;

use super::input::input_channel;
use super::tree_view::TreeView;

#[derive(Default, Copy, Clone, PartialEq)]
pub enum FocussedPane {
    #[default]
    Main,
    Help,
    Mark,
    Glob,
}

#[derive(Default)]
pub struct Cursor {
    pub show: bool,
    pub x: u16,
    pub y: u16,
}

#[derive(Default)]
pub struct AppState {
    pub navigation: Navigation,
    pub glob_navigation: Option<Navigation>,
    pub entries: Vec<EntryDataBundle>,
    pub sorting: SortMode,
    pub message: Option<String>,
    pub focussed: FocussedPane,
    pub is_scanning: bool,
}

pub enum ProcessingResult {
    Finished(WalkResult),
    ExitRequested(WalkResult),
}

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
            entries_traversed: tree_view.traversal.entries_traversed,
            total_bytes: tree_view.traversal.total_bytes,
            start: tree_view.traversal.start,
            elapsed: tree_view.traversal.elapsed,
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

    pub fn process_events<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: impl Iterator<Item = Event>,
    ) -> Result<ProcessingResult>
    where
        B: Backend,
    {
        use crosstermion::crossterm::event::KeyCode::*;
        use FocussedPane::*;

        {
            let tree_view = self.tree_view(traversal);
            self.draw(window, &tree_view, *display, terminal)?;
        }

        for event in events {
            let key = match event {
                Event::Key(key) if key.kind != KeyEventKind::Release => key,
                Event::Resize(_, _) => refresh_key(),
                _ => continue,
            };

            self.reset_message();

            let glob_focussed = self.focussed == Glob;
            let mut tree_view = self.tree_view(traversal);
            let mut handled = true;
            match key.code {
                Esc => {
                    if let Some(value) = self.handle_quit(&mut tree_view, window) {
                        return value;
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
                    return Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: tree_view.traversal.io_errors,
                    }))
                }
                Char('q') if !glob_focussed => {
                    if let Some(value) = self.handle_quit(&mut tree_view, window) {
                        return value;
                    }
                }
                _ => {
                    handled = false;
                }
            }

            if !handled {
                match self.focussed {
                    Mark => {
                        self.dispatch_to_mark_pane(key, window, &mut tree_view, *display, terminal)
                    }
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
                        Char('a') => {
                            self.mark_all_entries(MarkEntryMode::Toggle, window, &tree_view)
                        }
                        Char('o') | Char('l') | Enter | Right => {
                            self.enter_node_with_traversal(&tree_view)
                        }
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
                        Char('c') => self.cycle_count_sorting(&tree_view),
                        Char('g') => display.byte_vis.cycle(),
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
        }
        Ok(ProcessingResult::Finished(WalkResult {
            num_errors: traversal.io_errors,
        }))
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
    ) -> Option<std::result::Result<ProcessingResult, anyhow::Error>> {
        use FocussedPane::*;
        match self.focussed {
            Main => {
                if self.glob_navigation.is_some() {
                    self.handle_glob_quit(tree_view, window);
                } else {
                    return Some(Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: tree_view.traversal.io_errors,
                    })));
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

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: AppState,
    pub window: MainWindow,
}

type KeyboardInputAndApp = (crossbeam::channel::Receiver<Event>, TerminalApp);

impl TerminalApp {
    pub fn refresh_view<B>(&mut self, terminal: &mut Terminal<B>)
    where
        B: Backend,
    {
        // Use an event that does nothing to trigger a refresh
        self.state
            .process_events(
                &mut self.window,
                &mut self.traversal,
                &mut self.display,
                terminal,
                std::iter::once(Event::Key(refresh_key())),
            )
            .ok();
    }

    pub fn process_events<B>(
        &mut self,
        terminal: &mut Terminal<B>,
        events: impl Iterator<Item = Event>,
    ) -> Result<WalkResult>
    where
        B: Backend,
    {
        match self.state.process_events(
            &mut self.window,
            &mut self.traversal,
            &mut self.display,
            terminal,
            events,
        )? {
            ProcessingResult::Finished(res) | ProcessingResult::ExitRequested(res) => Ok(res),
        }
    }

    pub fn initialize<B>(
        terminal: &mut Terminal<B>,
        options: WalkOptions,
        input_paths: Vec<PathBuf>,
        mode: Interaction,
    ) -> Result<Option<KeyboardInputAndApp>>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;
        let mut display: DisplayOptions = options.clone().into();
        display.byte_vis = ByteVisualization::PercentageAndBar;
        let mut window = MainWindow::default();
        let keys_rx = match mode {
            Interaction::None => {
                let (_, keys_rx) = crossbeam::channel::unbounded();
                keys_rx
            }
            Interaction::Full => input_channel(),
        };

        #[inline]
        fn fetch_buffered_key_events(keys_rx: &Receiver<Event>) -> Vec<Event> {
            let mut keys = Vec::new();
            while let Ok(key) = keys_rx.try_recv() {
                keys.push(key);
            }
            keys
        }

        let mut state = AppState {
            is_scanning: true,
            ..Default::default()
        };
        let mut received_events = false;
        let traversal =
            Traversal::from_walk(options, input_paths, &keys_rx, |traversal, event| {
                if !received_events {
                    state.navigation_mut().view_root = traversal.root_index;
                }
                state.entries = sorted_entries(
                    &traversal.tree,
                    state.navigation().view_root,
                    state.sorting,
                    state.glob_root(),
                );
                if !received_events {
                    state.navigation_mut().selected = state.entries.first().map(|b| b.index);
                }
                state.reset_message(); // force "scanning" to appear

                let mut events = fetch_buffered_key_events(&keys_rx);
                if let Some(event) = event {
                    // Updater is triggered by an event, insert it
                    // before any events fetched later.
                    events.insert(0, event);
                }
                received_events |= !events.is_empty();

                let should_exit = match state.process_events(
                    &mut window,
                    traversal,
                    &mut display,
                    terminal,
                    events.into_iter(),
                )? {
                    ProcessingResult::ExitRequested(_) => true,
                    ProcessingResult::Finished(_) => false,
                };

                Ok(should_exit)
            })?;

        let traversal = match traversal {
            Some(t) => t,
            None => return Ok(None),
        };

        state.is_scanning = false;
        if !received_events {
            state.navigation_mut().view_root = traversal.root_index;
        }
        state.entries = sorted_entries(
            &traversal.tree,
            state.navigation().view_root,
            state.sorting,
            state.glob_root(),
        );
        state.navigation_mut().selected = state
            .navigation()
            .selected
            .filter(|_| received_events)
            .or_else(|| state.entries.first().map(|b| b.index));

        let mut app = TerminalApp {
            state,
            display,
            traversal,
            window,
        };
        app.refresh_view(terminal);

        Ok(Some((keys_rx, app)))
    }
}

pub enum Interaction {
    Full,
    #[allow(dead_code)]
    None,
}

fn refresh_key() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::ALT)
}
