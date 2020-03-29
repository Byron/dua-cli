use crate::interactive::{
    sorted_entries,
    widgets::{MainWindow, MainWindowProps},
    ByteVisualization, CursorDirection, DisplayOptions, EntryDataBundle, SortMode,
};
use dua::{
    traverse::{Traversal, TreeIndex},
    WalkOptions, WalkResult,
};
use failure::Error;
use std::{collections::BTreeMap, io, path::PathBuf};
use termion::{event::Key, input::TermRead};
use tui::backend::Backend;
use tui_react::Terminal;

#[derive(Copy, Clone)]
pub enum FocussedPane {
    Main,
    Help,
    Mark,
}

impl Default for FocussedPane {
    fn default() -> Self {
        FocussedPane::Main
    }
}

#[derive(Default)]
pub struct AppState {
    pub root: TreeIndex,
    pub selected: Option<TreeIndex>,
    pub entries: Vec<EntryDataBundle>,
    pub sorting: SortMode,
    pub message: Option<String>,
    pub focussed: FocussedPane,
    pub bookmarks: BTreeMap<TreeIndex, TreeIndex>,
    pub is_scanning: bool,
}

pub enum ProcessingResult {
    Finished(WalkResult),
    ExitRequested(WalkResult),
}

impl AppState {
    pub fn draw<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &Traversal,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Error>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            traversal: &traversal,
            display,
            state: &self,
        };
        draw_window(window, props, terminal)
    }

    pub fn process_events<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        keys: impl Iterator<Item = Result<Key, io::Error>>,
    ) -> Result<ProcessingResult, Error>
    where
        B: Backend,
    {
        use termion::event::Key::*;
        use FocussedPane::*;

        self.reset_message();
        self.draw(window, traversal, display.clone(), terminal)?;
        for key in keys.filter_map(Result::ok) {
            self.reset_message();
            match key {
                Char('?') => self.toggle_help_pane(window),
                Char('\t') => {
                    self.cycle_focus(window);
                }
                Ctrl('c') => {
                    return Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: traversal.io_errors,
                    }))
                }
                Char('q') | Esc => match self.focussed {
                    Main => {
                        return Ok(ProcessingResult::ExitRequested(WalkResult {
                            num_errors: traversal.io_errors,
                        }))
                    }
                    Mark => self.focussed = Main,
                    Help => {
                        self.focussed = Main;
                        window.help_pane = None
                    }
                },
                _ => {}
            }

            match self.focussed {
                FocussedPane::Mark => {
                    self.dispatch_to_mark_pane(key, window, traversal, display.clone(), terminal)
                }
                FocussedPane::Help => {
                    window.help_pane.as_mut().expect("help pane").key(key);
                }
                FocussedPane::Main => match key {
                    Char('O') => self.open_that(traversal),
                    Char(' ') => self.mark_entry(false, window, traversal),
                    Char('d') => self.mark_entry(true, window, traversal),
                    Char('u') | Char('h') | Backspace | Left => {
                        self.exit_node_with_traversal(traversal)
                    }
                    Char('o') | Char('l') | Char('\n') | Right => {
                        self.enter_node_with_traversal(traversal)
                    }
                    Ctrl('u') | PageUp => self.change_entry_selection(CursorDirection::PageUp),
                    Char('k') | Up => self.change_entry_selection(CursorDirection::Up),
                    Char('j') | Down => self.change_entry_selection(CursorDirection::Down),
                    Ctrl('d') | PageDown => self.change_entry_selection(CursorDirection::PageDown),
                    Char('s') => self.cycle_sorting(traversal),
                    Char('g') => display.byte_vis.cycle(),
                    _ => {}
                },
            };
            self.draw(window, traversal, display.clone(), terminal)?;
        }
        Ok(ProcessingResult::Finished(WalkResult {
            num_errors: traversal.io_errors,
        }))
    }
}

pub fn draw_window<B>(
    window: &mut MainWindow,
    props: MainWindowProps,
    terminal: &mut Terminal<B>,
) -> Result<(), Error>
where
    B: Backend,
{
    let area = terminal.pre_render()?;
    window.render(props, area, terminal.current_buffer_mut());
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

impl TerminalApp {
    pub fn process_events<B>(
        &mut self,
        terminal: &mut Terminal<B>,
        keys: impl Iterator<Item = Result<Key, io::Error>>,
    ) -> Result<WalkResult, Error>
    where
        B: Backend,
    {
        match self.state.process_events(
            &mut self.window,
            &mut self.traversal,
            &mut self.display,
            terminal,
            keys,
        )? {
            ProcessingResult::Finished(res) | ProcessingResult::ExitRequested(res) => Ok(res),
        }
    }

    pub fn initialize<B>(
        terminal: &mut Terminal<B>,
        options: WalkOptions,
        input: Vec<PathBuf>,
        mode: Interaction,
    ) -> Result<Option<TerminalApp>, Error>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;
        let mut display: DisplayOptions = options.clone().into();
        display.byte_vis = ByteVisualization::PercentageAndBar;
        let mut window = MainWindow::default();
        let (keys_tx, keys_rx) = std::sync::mpsc::channel(); // unbounded
        match mode {
            Interaction::None => drop(keys_tx),
            Interaction::Full => drop(std::thread::spawn(move || {
                let keys = std::io::stdin().keys();
                for key in keys {
                    if let Err(_) = keys_tx.send(key) {
                        break;
                    }
                }
            })),
        }

        let fetch_buffered_key_events = move || {
            let mut keys = Vec::new();
            while let Ok(key) = keys_rx.try_recv() {
                keys.push(key);
            }
            keys
        };

        let mut state = None;
        let traversal = Traversal::from_walk(options, input, |traversal| {
            let s = match state.as_mut() {
                Some(s) => s,
                None => {
                    state = Some({
                        let sorting = Default::default();
                        let entries =
                            sorted_entries(&traversal.tree, traversal.root_index, sorting);
                        AppState {
                            root: traversal.root_index,
                            sorting,
                            selected: entries.get(0).map(|b| b.index),
                            entries,
                            is_scanning: true,
                            ..Default::default()
                        }
                    });
                    state.as_mut().expect("state to be present, we just set it")
                }
            };
            let should_exit = match s.process_events(
                &mut window,
                traversal,
                &mut display,
                terminal,
                fetch_buffered_key_events().into_iter(),
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
        drop(fetch_buffered_key_events); // shutdown input event handler early for good measure

        Ok(Some(TerminalApp {
            state: {
                let mut s = state.unwrap_or_else(|| {
                    let sorting = Default::default();
                    let root = traversal.root_index;
                    let entries = sorted_entries(&traversal.tree, root, sorting);
                    AppState {
                        root,
                        sorting,
                        entries,
                        ..Default::default()
                    }
                });
                s.is_scanning = false;
                s.entries = sorted_entries(&traversal.tree, s.root, s.sorting);
                s.selected = s.selected.or_else(|| s.entries.get(0).map(|b| b.index));
                s
            },
            display,
            traversal,
            window,
        }))
    }
}

pub enum Interaction {
    Full,
    #[allow(dead_code)]
    None,
}
