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
use std::{io, path::PathBuf};
use termion::event::Key;
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
}

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: AppState,
    pub window: MainWindow,
}

impl TerminalApp {
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
    pub fn draw<B>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Error>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            traversal: &self.traversal,
            display: self.display,
            state: &self.state,
        };
        Self::draw_window(&mut self.window, props, terminal)
    }
    pub fn process_events<B>(
        &mut self,
        terminal: &mut Terminal<B>,
        keys: impl Iterator<Item = Result<Key, io::Error>>,
    ) -> Result<WalkResult, Error>
    where
        B: Backend,
    {
        use termion::event::Key::*;
        use FocussedPane::*;

        self.draw(terminal)?;
        for key in keys.filter_map(Result::ok) {
            self.update_message();
            match key {
                Char('?') => self.toggle_help_pane(),
                Char('\t') => {
                    self.cycle_focus();
                }
                Ctrl('c') => break,
                Char('q') | Esc => match self.state.focussed {
                    Main => break,
                    Mark => self.state.focussed = Main,
                    Help => {
                        self.state.focussed = Main;
                        self.window.help_pane = None
                    }
                },
                _ => {}
            }

            match self.state.focussed {
                FocussedPane::Mark => self.dispatch_to_mark_pane(key, terminal),
                FocussedPane::Help => {
                    self.window.help_pane.as_mut().expect("help pane").key(key);
                }
                FocussedPane::Main => match key {
                    Char('O') => self.open_that(),
                    Char(' ') => self.mark_entry(false),
                    Char('d') => self.mark_entry(true),
                    Char('u') | Backspace | Left => self.exit_node(),
                    Char('o') | Char('\n') | Right => self.enter_node(),
                    Ctrl('u') => self.change_entry_selection(CursorDirection::PageUp),
                    Char('k') | Up => self.change_entry_selection(CursorDirection::Up),
                    Char('j') | Down => self.change_entry_selection(CursorDirection::Down),
                    Ctrl('d') => self.change_entry_selection(CursorDirection::PageDown),
                    Char('s') => self.cycle_sorting(),
                    Char('g') => self.display.byte_vis.cycle(),
                    _ => {}
                },
            };
            self.draw(terminal)?;
        }
        Ok(WalkResult {
            num_errors: self.traversal.io_errors,
        })
    }

    pub fn initialize<B>(
        terminal: &mut Terminal<B>,
        options: WalkOptions,
        input: Vec<PathBuf>,
    ) -> Result<TerminalApp, Error>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;
        let mut display_options: DisplayOptions = options.clone().into();
        display_options.byte_vis = ByteVisualization::Bar;
        let mut window = MainWindow::default();

        let traversal = Traversal::from_walk(options, input, move |traversal| {
            let state = AppState {
                root: traversal.root_index,
                sorting: Default::default(),
                message: Some("-> scanning <-".into()),
                entries: sorted_entries(&traversal.tree, traversal.root_index, Default::default()),
                ..Default::default()
            };
            let props = MainWindowProps {
                traversal,
                display: display_options,
                state: &state,
            };
            Self::draw_window(&mut window, props, terminal)
        })?;

        let sorting = Default::default();
        let root = traversal.root_index;
        let entries = sorted_entries(&traversal.tree, root, sorting);
        let selected = entries.get(0).map(|b| b.index);
        display_options.byte_vis = ByteVisualization::PercentageAndBar;
        Ok(TerminalApp {
            state: AppState {
                root,
                sorting,
                selected,
                entries,
                ..Default::default()
            },
            display: display_options,
            traversal,
            window: Default::default(),
        })
    }
}
