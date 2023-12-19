use crate::interactive::{
    app::navigation::NavigationState,
    sorted_entries,
    widgets::{glob_search, MainWindow, MainWindowProps},
    ByteVisualization, CursorDirection, CursorMode, DisplayOptions, EntryDataBundle, MarkEntryMode,
    SortMode,
};
use anyhow::Result;
use crosstermion::input::{input_channel, Event, Key};
use dua::{
    traverse::{EntryData, Traversal},
    WalkOptions, WalkResult,
};
use std::path::PathBuf;
use tui::backend::Backend;
use tui_react::Terminal;

use super::tree_view::{GlobTreeView, NormalTreeView, TreeView};

#[derive(Default, Copy, Clone, PartialEq)]
pub enum FocussedPane {
    #[default]
    Main,
    Help,
    Mark,
    Glob,
}

#[derive(Default)]
pub struct AppState {
    pub normal_mode: NavigationState,
    pub glob_mode: Option<NavigationState>,
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
    pub fn navigation_mut(&mut self) -> &mut NavigationState {
        self.glob_mode.as_mut().unwrap_or(&mut self.normal_mode)
    }

    pub fn navigation(&self) -> &NavigationState {
        self.glob_mode.as_ref().unwrap_or(&self.normal_mode)
    }

    pub fn draw<B>(
        &mut self,
        window: &mut MainWindow,
        tree_view: &dyn TreeView,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            current_path: tree_view.current_path(self.navigation().view_root),
            entries_traversed: tree_view.traversal().entries_traversed,
            total_bytes: tree_view.traversal().total_bytes,
            start: tree_view.traversal().start,
            elapsed: tree_view.traversal().elapsed,
            display,
            state: self,
        };
        draw_window(window, props, terminal)
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
        use crosstermion::input::Key::*;
        use FocussedPane::*;

        {
            let tree_view = self.tree_view(traversal);
            self.draw(window, tree_view.as_ref(), *display, terminal)?;
        }

        for event in events {
            let key = match event {
                Event::Key(key) => key,
                Event::Resize(_, _) => Alt('\r'),
            };

            let mut tree_view = self.tree_view(traversal);

            self.reset_message();
            let mut handled = true;
            match key {
                Char('?') if self.focussed != FocussedPane::Glob => self.toggle_help_pane(window),
                Char('/') if self.focussed != FocussedPane::Glob => {
                    self.toggle_glob_search(window);
                }
                Char('\t') => {
                    self.cycle_focus(window);
                }
                Ctrl('c') if self.focussed != FocussedPane::Glob => {
                    return Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: tree_view.traversal().io_errors,
                    }))
                }
                Char('q') if self.focussed != FocussedPane::Glob => {
                    if let Some(value) = self.handle_quit(tree_view.as_mut(), window) {
                        return value;
                    }
                }
                Esc => {
                    if let Some(value) = self.handle_quit(tree_view.as_mut(), window) {
                        return value;
                    }
                }
                _ => {
                    handled = false;
                }
            }

            if !handled {
                match self.focussed {
                    Mark => self.dispatch_to_mark_pane(
                        key,
                        window,
                        tree_view.as_mut(),
                        *display,
                        terminal,
                    ),
                    Help => {
                        window
                            .help_pane
                            .as_mut()
                            .expect("help pane")
                            .process_events(key);
                    }
                    Glob => {
                        let glob_pane = window.glob_pane.as_mut().expect("glob pane");
                        match key {
                            Char('\n') => {
                                self.search_glob_pattern(tree_view.as_mut(), &glob_pane.input)
                            }
                            _ => glob_pane.process_events(key),
                        }
                    }
                    Main => match key {
                        Char('O') => self.open_that(tree_view.as_ref()),
                        Char(' ') => self.mark_entry(
                            CursorMode::KeepPosition,
                            MarkEntryMode::Toggle,
                            window,
                            tree_view.as_ref(),
                        ),
                        Char('d') => self.mark_entry(
                            CursorMode::Advance,
                            MarkEntryMode::Toggle,
                            window,
                            tree_view.as_ref(),
                        ),
                        Char('x') => self.mark_entry(
                            CursorMode::Advance,
                            MarkEntryMode::MarkForDeletion,
                            window,
                            tree_view.as_ref(),
                        ),
                        Char('a') => {
                            self.mark_all_entries(MarkEntryMode::Toggle, window, tree_view.as_ref())
                        }
                        Char('u') | Char('h') | Backspace | Left => {
                            self.exit_node_with_traversal(tree_view.as_ref())
                        }
                        Char('o') | Char('l') | Char('\n') | Right => {
                            self.enter_node_with_traversal(tree_view.as_ref())
                        }
                        Char('H') | Home => self.change_entry_selection(CursorDirection::ToTop),
                        Char('G') | End => self.change_entry_selection(CursorDirection::ToBottom),
                        Ctrl('u') | PageUp => self.change_entry_selection(CursorDirection::PageUp),
                        Char('k') | Up => self.change_entry_selection(CursorDirection::Up),
                        Char('j') | Down => self.change_entry_selection(CursorDirection::Down),
                        Ctrl('d') | PageDown => {
                            self.change_entry_selection(CursorDirection::PageDown)
                        }
                        Char('s') => self.cycle_sorting(tree_view.as_ref()),
                        Char('m') => self.cycle_mtime_sorting(tree_view.as_ref()),
                        Char('c') => self.cycle_count_sorting(tree_view.as_ref()),
                        Char('g') => display.byte_vis.cycle(),
                        _ => {}
                    },
                };
            }
            self.draw(window, tree_view.as_ref(), *display, terminal)?;
        }
        Ok(ProcessingResult::Finished(WalkResult {
            num_errors: traversal.io_errors,
        }))
    }

    fn tree_view<'a>(&mut self, traversal: &'a mut Traversal) -> Box<dyn TreeView + 'a> {
        let tree_view: Box<dyn TreeView> = if let Some(glob_source) = &self.glob_mode {
            Box::new(GlobTreeView {
                traversal,
                glob_tree_root: glob_source.tree_root,
            })
        } else {
            Box::new(NormalTreeView { traversal })
        };
        tree_view
    }

    fn search_glob_pattern(&mut self, tree_view: &mut dyn TreeView, glob_pattern: &str) {
        use FocussedPane::*;
        let search_results =
            glob_search(tree_view.tree(), self.normal_mode.view_root, glob_pattern);
        match search_results {
            Ok(search_results) => {
                if let Some(glob_source) = &self.glob_mode {
                    tree_view.tree_as_mut().remove_node(glob_source.tree_root);
                }

                let tree_root = tree_view.tree_as_mut().add_node(EntryData::default());
                let glob_source = NavigationState {
                    tree_root,
                    view_root: tree_root,
                    selected: Some(tree_root),
                    ..Default::default()
                };
                self.glob_mode = Some(glob_source);

                for idx in search_results {
                    tree_view.tree_as_mut().add_edge(tree_root, idx, ());
                }

                let glob_tree_view = GlobTreeView {
                    traversal: tree_view.traversal_as_mut(),
                    glob_tree_root: tree_root
                };

                let new_entries =
                    glob_tree_view.sorted_entries(tree_root, self.sorting);

                let new_entries = self
                    .navigation_mut()
                    .selected
                    .map(|previously_selected| (previously_selected, new_entries));

                self.enter_node(new_entries);
                self.focussed = Main;
            }
            _ => self.message = Some("Glob search error!".into()),
        }
    }

    fn handle_quit(
        &mut self,
        tree_view: &mut dyn TreeView,
        window: &mut MainWindow,
    ) -> Option<std::result::Result<ProcessingResult, anyhow::Error>> {
        use FocussedPane::*;
        match self.focussed {
            Main => {
                if self.glob_mode.is_some() {
                    self.handle_glob_quit(tree_view, window);
                } else {
                    return Some(Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: tree_view.traversal().io_errors,
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

    fn handle_glob_quit(&mut self, tree_view: &mut dyn TreeView, window: &mut MainWindow) {
        use FocussedPane::*;
        self.focussed = Main;
        if let Some(glob_source) = &self.glob_mode {
            tree_view.tree_as_mut().remove_node(glob_source.tree_root);
        }
        self.glob_mode = None;
        window.glob_pane = None;

        let normal_tree_view = NormalTreeView {
            traversal: tree_view.traversal_as_mut()
        };

        let new_entries = self.navigation().selected.map(|previously_selected| {
            (
                previously_selected,
                normal_tree_view.sorted_entries(
                    self.navigation().view_root,
                    self.sorting),
            )
        });
        self.enter_node(new_entries);
    }
}

pub fn draw_window<B>(
    window: &mut MainWindow,
    props: MainWindowProps,
    terminal: &mut Terminal<B>,
) -> Result<()>
where
    B: Backend,
{
    let area = terminal.pre_render()?;
    window.render(props, area, terminal);
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

type KeyboardInputAndApp = (std::sync::mpsc::Receiver<Event>, TerminalApp);

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
                std::iter::once(Event::Key(Key::Alt('\r'))),
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
                let (_, keys_rx) = std::sync::mpsc::channel();
                keys_rx
            }
            Interaction::Full => input_channel(),
        };

        let fetch_buffered_key_events = || {
            let mut keys = Vec::new();
            while let Ok(key) = keys_rx.try_recv() {
                keys.push(key);
            }
            keys
        };

        let mut state = AppState {
            is_scanning: true,
            ..Default::default()
        };
        let mut received_events = false;
        let traversal = Traversal::from_walk(options, input_paths, |traversal| {
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

            let events = fetch_buffered_key_events();
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
