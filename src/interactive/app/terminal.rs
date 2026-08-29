use std::path::PathBuf;

#[cfg(unix)]
use std::io;

use crate::interactive::EntryCheck;
use anyhow::Result;
use crossbeam::channel::Receiver;
use crossterm::event::Event;
#[cfg(unix)]
use crossterm::{
    cursor::Show,
    event::{DisableFocusChange, EnableFocusChange},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use dua::Config;
#[cfg(test)]
use dua::traverse::TraversalStats;
use dua::{ByteFormat, WalkOptions, WalkResult, traverse::Traversal};
use tui::{Terminal, backend::Backend};

use crate::interactive::widgets::MainWindow;

use super::{DisplayOptions, sorted_entries, state::AppState};

/// Restores the user's terminal, suspends the process, and reinitializes the TUI after resume.
///
/// The previous frame is invalidated after resume so the caller's next draw repaints the complete
/// UI. This function does not draw by itself; the event loop draws normally after handling the
/// suspend key event.
#[cfg(unix)]
pub fn suspend_terminal<B>(terminal: &mut Terminal<B>, focus_change: bool) -> Result<()>
where
    B: Backend,
{
    let mut stderr = io::stderr();
    if focus_change {
        execute!(stderr, DisableFocusChange)?;
    }
    execute!(stderr, Show)?;
    disable_raw_mode()?;
    execute!(stderr, LeaveAlternateScreen)?;

    // This suspends the program, and anything that follows undoes the lines above.
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;

    enable_raw_mode()?;
    execute!(stderr, EnterAlternateScreen)?;
    if focus_change {
        execute!(stderr, EnableFocusChange)?;
    }
    // `Terminal::clear()` queries the cursor position, racing the input thread for its response.
    // This triggers a redraw as well without that issue.
    terminal.swap_buffers();
    Ok(())
}

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub config: Config,
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
        root_path: Option<PathBuf>,
        config: Config,
    ) -> Result<TerminalApp>
    where
        B: Backend,
    {
        terminal
            .hide_cursor()
            .map_err(|err| anyhow::Error::msg(err.to_string()))?;
        terminal
            .clear()
            .map_err(|err| anyhow::Error::msg(err.to_string()))?;

        let display = DisplayOptions::new(byte_format);
        let window = MainWindow::default();

        let mut state = AppState::new(walk_options, input, root_path);
        if config.gitignore == Some(false) {
            state.gitignored_entries = None;
        }
        if config.cleanup_heuristics == Some(false) {
            state.cleanup_candidates = None;
        }
        state.allow_entry_check = entry_check;
        let traversal = Traversal::new();
        #[cfg(test)]
        let traversal_stats = TraversalStats::default();

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
            config,
            traversal,
            display,
            state,
            #[cfg(test)]
            stats: traversal_stats,
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
            &self.config,
        )
    }

    pub fn process_events_once<B>(
        &mut self,
        terminal: &mut Terminal<B>,
        events: Receiver<Event>,
    ) -> Result<WalkResult>
    where
        B: Backend,
    {
        self.state.process_events_once(
            &mut self.window,
            &mut self.traversal,
            &mut self.display,
            terminal,
            events,
            &self.config,
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
                    &self.config,
                )?;
            }
            Ok(WalkResult {
                num_errors: self.stats.io_errors,
            })
        }
    }
}
