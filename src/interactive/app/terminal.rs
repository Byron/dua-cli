use std::path::PathBuf;

use crate::interactive::EntryCheck;
use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::input::Event;
#[cfg(test)]
use dua::traverse::TraversalStats;
use dua::{ByteFormat, WalkOptions, WalkResult, traverse::Traversal};
use tui::{Terminal, backend::Backend};

use crate::interactive::widgets::MainWindow;

use super::{DisplayOptions, sorted_entries, state::AppState};

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    #[cfg(test)]
    pub stats: TraversalStats,
    pub display: DisplayOptions,
    pub state: AppState,
    pub window: MainWindow,
}

impl TerminalApp {
    pub fn initialize<B>(
        terminal: &mut Terminal<B>,
        walk_options: WalkOptions,
        byte_format: ByteFormat,
        entry_check: bool,
        input: Vec<PathBuf>,
    ) -> Result<TerminalApp>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;

        let display = DisplayOptions::new(byte_format);
        let window = MainWindow::default();

        let mut state = AppState::new(walk_options, input);
        state.allow_entry_check = entry_check;
        let traversal = Traversal::new();
        #[cfg(test)]
        let stats = TraversalStats::default();

        state.navigation_mut().view_root = traversal.root_index;
        state.entries = sorted_entries(
            &traversal.tree,
            state.navigation().view_root,
            state.sorting,
            state.glob_root(),
            EntryCheck::new(state.scan.is_some(), state.allow_entry_check),
        );
        state.navigation_mut().selected = state.entries.first().map(|b| b.index);

        let app = TerminalApp {
            state,
            display,
            traversal,
            #[cfg(test)]
            stats,
            window,
        };
        Ok(app)
    }

    pub fn traverse(&mut self) -> Result<()> {
        self.state.traverse(&self.traversal)?;
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
        self.state.process_events(
            &mut self.window,
            &mut self.traversal,
            &mut self.display,
            terminal,
            events,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::TerminalApp;

    impl TerminalApp {
        pub fn run_until_traversed<B>(
            &mut self,
            terminal: &mut Terminal<B>,
            events: Receiver<Event>,
        ) -> Result<WalkResult>
        where
            B: Backend,
        {
            while self.state.scan.is_some() {
                self.state.process_event(
                    &mut self.window,
                    &mut self.traversal,
                    &mut self.display,
                    terminal,
                    &events,
                )?;
            }
            Ok(WalkResult {
                num_errors: self.stats.io_errors,
            })
        }
    }
}
