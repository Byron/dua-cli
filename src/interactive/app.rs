use crate::{
    interactive::widgets::{DrawState, MainWindow},
    path_of, sorted_entries,
    traverse::{Traversal, TreeIndex},
    ByteFormat, WalkOptions, WalkResult,
};
use failure::Error;
use itertools::Itertools;
use petgraph::Direction;
use std::{io, path::PathBuf};
use termion::input::{Keys, TermReadEventsAndRaw};
use tui::{backend::Backend, widgets::Widget, Terminal};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    SizeDescending,
    SizeAscending,
}

impl SortMode {
    pub fn toggle_size(&mut self) {
        use SortMode::*;
        *self = match self {
            SizeAscending => SizeDescending,
            SizeDescending => SizeAscending,
        }
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::SizeDescending
    }
}

/// Options to configure how we display things
#[derive(Clone, Copy)]
pub struct DisplayOptions {
    pub byte_format: ByteFormat,
}

impl From<WalkOptions> for DisplayOptions {
    fn from(WalkOptions { byte_format, .. }: WalkOptions) -> Self {
        DisplayOptions { byte_format }
    }
}

pub struct AppState {
    pub root: TreeIndex,
    pub selected: Option<TreeIndex>,
    pub sorting: SortMode,
    pub message: Option<String>,
}

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: AppState,
    pub widgets: DrawState,
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
            ref mut widgets,
        } = self;

        terminal.draw(|mut f| {
            let full_screen = f.size();
            MainWindow {
                traversal,
                display: *display,
                state: &state,
                widgets,
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

        self.draw(terminal)?;
        for key in keys.filter_map(Result::ok) {
            match key {
                Char('O') => self.open_that(),
                Char('u') => self.exit_node(),
                Char('o') => self.enter_node(),
                Ctrl('u') => self.change_vertical_index(CursorDirection::PageUp),
                Char('k') => self.change_vertical_index(CursorDirection::Up),
                Char('j') => self.change_vertical_index(CursorDirection::Down),
                Ctrl('d') => self.change_vertical_index(CursorDirection::PageDown),
                Char('s') => self.state.sorting.toggle_size(),
                Ctrl('c') | Char('q') => break,
                _ => {}
            };
            self.draw(terminal)?;
        }
        Ok(WalkResult {
            num_errors: self.traversal.io_errors,
        })
    }

    fn exit_node(&mut self) -> () {
        if let Some(parent_idx) = self
            .traversal
            .tree
            .neighbors_directed(self.state.root, Direction::Incoming)
            .next()
        {
            self.state.root = parent_idx;
            self.state.selected =
                sorted_entries(&self.traversal.tree, parent_idx, self.state.sorting)
                    .get(0)
                    .map(|(idx, _)| *idx);
        }
    }

    fn open_that(&mut self) -> () {
        match self.state.selected {
            Some(ref idx) => {
                open::that(path_of(&self.traversal.tree, *idx)).ok();
            }
            None => {}
        }
    }

    fn enter_node(&mut self) -> () {
        if let Some(idx) = self.state.selected {
            let entries = sorted_entries(&self.traversal.tree, idx, self.state.sorting);
            if let Some((next_selection, _)) = entries.get(0) {
                self.state.root = idx;
                self.state.selected = Some(*next_selection);
            }
        }
    }

    fn change_vertical_index(&mut self, direction: CursorDirection) -> () {
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
        let display_options: DisplayOptions = options.clone().into();
        let traversal = Traversal::from_walk(options, input, move |traversal| {
            terminal.draw(|mut f| {
                let full_screen = f.size();
                let state = AppState {
                    root: traversal.root_index,
                    sorting: Default::default(),
                    message: Some("-> scanning <-".into()),
                    selected: None,
                };
                MainWindow {
                    traversal,
                    display: display_options,
                    state: &state,
                    widgets: &mut Default::default(),
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
        Ok(TerminalApp {
            state: AppState {
                root,
                sorting,
                message: None,
                selected,
            },
            display: display_options,
            traversal,
            widgets: Default::default(),
        })
    }
}
