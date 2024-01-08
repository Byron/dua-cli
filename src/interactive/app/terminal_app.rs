use std::path::PathBuf;

use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::input::Event;
use dua::{
    traverse::{EntryData, Traversal, Tree},
    ByteFormat, WalkOptions, WalkResult,
};
use tui::prelude::Backend;
use tui_react::Terminal;

use crate::interactive::widgets::MainWindow;

use super::{
    app_state::{AppState, ProcessingResult},
    sorted_entries, DisplayOptions,
};

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: AppState,
    pub window: MainWindow,
    pub walk_options: WalkOptions,
}

impl TerminalApp {
    pub fn initialize<B>(
        terminal: &mut Terminal<B>,
        walk_options: WalkOptions,
        byte_format: ByteFormat,
    ) -> Result<TerminalApp>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;

        let display = DisplayOptions::new(byte_format);
        let window = MainWindow::default();

        let mut state = AppState {
            is_scanning: false,
            ..Default::default()
        };

        let traversal = {
            let mut tree = Tree::new();
            let root_index = tree.add_node(EntryData::default());
            Traversal {
                tree,
                root_index,
                entries_traversed: 0,
                start: std::time::Instant::now(),
                elapsed: None,
                io_errors: 0,
                total_bytes: None,
            }
        };

        state.navigation_mut().view_root = traversal.root_index;
        state.entries = sorted_entries(
            &traversal.tree,
            state.navigation().view_root,
            state.sorting,
            state.glob_root(),
        );
        state.navigation_mut().selected = state.entries.first().map(|b| b.index);

        let app = TerminalApp {
            state,
            display,
            traversal,
            window,
            walk_options,
        };
        Ok(app)
    }

    pub fn traverse(&mut self, input: Vec<PathBuf>) -> Result<()> {
        self.state
            .traverse(&self.traversal, &self.walk_options, input)?;
        Ok(())
    }

    pub fn process_events<B>(
        &mut self,
        terminal: &mut Terminal<B>,
        events: Receiver<Event>,
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
            ProcessingResult::ExitRequested(res) => Ok(res),
        }
    }
}
