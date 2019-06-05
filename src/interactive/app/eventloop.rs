use crate::{
    interactive::{
        sorted_entries,
        widgets::{DrawState, HelpPaneState, MainWindow},
        SortMode,
    },
    ByteFormat,
};
use dua::{
    path_of,
    traverse::{Traversal, TreeIndex},
    WalkOptions, WalkResult,
};
use failure::Error;
use itertools::Itertools;
use petgraph::Direction;
use std::{fmt, io, path::PathBuf};
use termion::input::{Keys, TermReadEventsAndRaw};
use tui::{backend::Backend, widgets::Widget, Terminal};

#[derive(Clone, Copy)]
pub enum ByteVisualization {
    Percentage,
    Bar,
    LongBar,
    PercentageAndBar,
}

pub struct DisplayByteVisualization {
    format: ByteVisualization,
    percentage: f32,
}

impl Default for ByteVisualization {
    fn default() -> Self {
        ByteVisualization::PercentageAndBar
    }
}

impl ByteVisualization {
    pub fn cycle(&mut self) {
        use ByteVisualization::*;
        *self = match self {
            Bar => LongBar,
            LongBar => PercentageAndBar,
            PercentageAndBar => Percentage,
            Percentage => Bar,
        }
    }
    pub fn display(&self, percentage: f32) -> DisplayByteVisualization {
        DisplayByteVisualization {
            format: *self,
            percentage,
        }
    }
}

impl fmt::Display for DisplayByteVisualization {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use ByteVisualization::*;
        let Self { format, percentage } = self;

        const BAR_SIZE: usize = 10;
        match format {
            Percentage => Self::make_percentage(f, percentage),
            PercentageAndBar => {
                Self::make_percentage(f, percentage)?;
                f.write_str(" ")?;
                Self::make_bar(f, percentage, BAR_SIZE)
            }
            Bar => Self::make_bar(f, percentage, BAR_SIZE),
            LongBar => Self::make_bar(f, percentage, 20),
        }
    }
}

impl DisplayByteVisualization {
    fn make_bar(f: &mut fmt::Formatter, percentage: &f32, length: usize) -> Result<(), fmt::Error> {
        let block_length = (length as f32 * percentage).round() as usize;
        for _ in 0..block_length {
            f.write_str(tui::symbols::block::FULL)?;
        }
        for _ in 0..length - block_length {
            f.write_str(" ")?;
        }
        Ok(())
    }
    fn make_percentage(f: &mut fmt::Formatter, percentage: &f32) -> Result<(), fmt::Error> {
        write!(f, " {:>5.02}% ", percentage * 100.0)
    }
}

/// Options to configure how we display things
#[derive(Clone, Copy)]
pub struct DisplayOptions {
    pub byte_format: ByteFormat,
    pub byte_vis: ByteVisualization,
}

impl From<WalkOptions> for DisplayOptions {
    fn from(WalkOptions { byte_format, .. }: WalkOptions) -> Self {
        DisplayOptions {
            byte_format,
            byte_vis: ByteVisualization::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum FocussedPane {
    Main,
    Help,
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
    pub sorting: SortMode,
    pub message: Option<String>,
    pub help_pane: Option<HelpPaneState>,
    pub focussed: FocussedPane,
}

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: AppState,
    pub draw_state: DrawState,
}

enum CursorDirection {
    PageDown,
    Down,
    Up,
    PageUp,
}

impl TerminalApp {
    fn draw<B>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Error>
    where
        B: Backend,
    {
        let Self {
            traversal,
            display,
            state,
            ref mut draw_state,
        } = self;

        terminal.draw(|mut f| {
            let full_screen = f.size();
            MainWindow {
                traversal,
                display: *display,
                state: &state,
                draw_state,
            }
            .render(&mut f, full_screen)
        })?;

        Ok(())
    }
    pub fn process_events<B, R>(
        &mut self,
        terminal: &mut Terminal<B>,
        keys: Keys<R>,
    ) -> Result<WalkResult, Error>
    where
        B: Backend,
        R: io::Read + TermReadEventsAndRaw,
    {
        use termion::event::Key::{Char, Ctrl};
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
                Char('q') => match self.state.focussed {
                    Main => break,
                    Help => {
                        self.state.focussed = Main;
                        self.state.help_pane = None
                    }
                },
                _ => {}
            }

            match self.state.focussed {
                FocussedPane::Help => match key {
                    Ctrl('u') => self.scroll_help(CursorDirection::PageUp),
                    Char('k') => self.scroll_help(CursorDirection::Up),
                    Char('j') => self.scroll_help(CursorDirection::Down),
                    Ctrl('d') => self.scroll_help(CursorDirection::PageDown),
                    _ => {}
                },
                FocussedPane::Main => match key {
                    Char('O') => self.open_that(),
                    Char('u') => self.exit_node(),
                    Char('o') => self.enter_node(),
                    Ctrl('u') => self.change_entry_selection(CursorDirection::PageUp),
                    Char('k') => self.change_entry_selection(CursorDirection::Up),
                    Char('j') => self.change_entry_selection(CursorDirection::Down),
                    Ctrl('d') => self.change_entry_selection(CursorDirection::PageDown),
                    Char('s') => self.state.sorting.toggle_size(),
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

    fn cycle_focus(&mut self) {
        use FocussedPane::*;
        self.state.focussed = match (self.state.focussed, self.state.help_pane) {
            (Main, Some(_)) => Help,
            (Help, _) => Main,
            _ => Main,
        };
    }

    fn toggle_help_pane(&mut self) {
        use FocussedPane::*;
        self.state.focussed = match self.state.focussed {
            Main => {
                self.state.help_pane = Some(HelpPaneState::default());
                Help
            }
            Help => {
                self.state.help_pane = None;
                Main
            }
        }
    }

    fn update_message(&mut self) {
        self.state.message = None;
    }

    fn exit_node(&mut self) {
        match self
            .traversal
            .tree
            .neighbors_directed(self.state.root, Direction::Incoming)
            .next()
        {
            Some(parent_idx) => {
                self.state.root = parent_idx;
                self.state.selected =
                    sorted_entries(&self.traversal.tree, parent_idx, self.state.sorting)
                        .get(0)
                        .map(|(idx, _)| *idx);
            }
            None => self.state.message = Some("Top level reached".into()),
        }
    }

    fn open_that(&mut self) {
        match self.state.selected {
            Some(ref idx) => {
                open::that(path_of(&self.traversal.tree, *idx)).ok();
            }
            None => {}
        }
    }

    fn enter_node(&mut self) {
        if let Some(new_root) = self.state.selected {
            let entries = sorted_entries(&self.traversal.tree, new_root, self.state.sorting);
            match entries.get(0) {
                Some((next_selection, _)) => {
                    self.state.root = new_root;
                    self.state.selected = Some(*next_selection);
                }
                None => self.state.message = Some("Entry is a file or an empty directory".into()),
            }
        }
    }

    fn scroll_help(&mut self, direction: CursorDirection) {
        use CursorDirection::*;
        let scroll = self.draw_state.help_scroll;
        self.draw_state.help_scroll = match direction {
            Down => scroll.saturating_add(1),
            Up => scroll.saturating_sub(1),
            PageDown => scroll.saturating_add(10),
            PageUp => scroll.saturating_sub(10),
        };
    }

    fn change_entry_selection(&mut self, direction: CursorDirection) {
        let entries = sorted_entries(&self.traversal.tree, self.state.root, self.state.sorting);
        let next_selected_pos = match self.state.selected {
            Some(ref selected) => entries
                .iter()
                .find_position(|(idx, _)| *idx == *selected)
                .map(|(idx, _)| match direction {
                    CursorDirection::PageDown => idx.saturating_add(10),
                    CursorDirection::Down => idx.saturating_add(1),
                    CursorDirection::Up => idx.saturating_sub(1),
                    CursorDirection::PageUp => idx.saturating_sub(10),
                })
                .unwrap_or(0),
            None => 0,
        };
        self.state.selected = entries
            .get(next_selected_pos)
            .or(entries.last())
            .map(|(idx, _)| *idx)
            .or(self.state.selected)
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
        let mut display_options: DisplayOptions = options.clone().into();
        display_options.byte_vis = ByteVisualization::Bar;
        let traversal = Traversal::from_walk(options, input, move |traversal| {
            terminal.draw(|mut f| {
                let full_screen = f.size();
                let state = AppState {
                    root: traversal.root_index,
                    sorting: Default::default(),
                    message: Some("-> scanning <-".into()),
                    ..Default::default()
                };
                MainWindow {
                    traversal,
                    display: display_options,
                    state: &state,
                    draw_state: &mut Default::default(),
                }
                .render(&mut f, full_screen)
            })?;
            Ok(())
        })?;

        let sorting = Default::default();
        let root = traversal.root_index;
        let selected = sorted_entries(&traversal.tree, root, sorting)
            .get(0)
            .map(|(idx, _)| *idx);
        display_options.byte_vis = ByteVisualization::PercentageAndBar;
        Ok(TerminalApp {
            state: AppState {
                root,
                sorting,
                selected,
                ..Default::default()
            },
            display: display_options,
            traversal,
            draw_state: Default::default(),
        })
    }
}
