use super::widgets::{DisplayState, MainWindow};
use crate::{interactive::Traversal, ByteFormat, WalkOptions, WalkResult};
use failure::Error;
use std::{io, path::PathBuf};
use termion::input::{Keys, TermReadEventsAndRaw};
use tui::widgets::Widget;
use tui::{backend::Backend, Terminal};

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

/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: DisplayState,
}

impl TerminalApp {
    fn draw_to_terminal<B>(&self, terminal: &mut Terminal<B>) -> Result<(), Error>
    where
        B: Backend,
    {
        let Self {
            traversal,
            display,
            state,
        } = self;
        terminal.draw(|mut f| {
            let full_screen = f.size();
            MainWindow {
                traversal,
                display: *display,
                state: &state,
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

        self.draw_to_terminal(terminal)?;
        for key in keys.filter_map(Result::ok) {
            match key {
                Ctrl('c') | Char('\n') | Char('q') => break,
                _ => {}
            };
            self.draw_to_terminal(terminal)?;
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
        let display_options: DisplayOptions = options.clone().into();
        let traversal = Traversal::from_walk(options, input, move |traversal| {
            terminal.draw(|mut f| {
                let full_screen = f.size();
                let state = DisplayState {
                    root: traversal.root_index,
                    selected: None,
                };
                MainWindow {
                    traversal,
                    display: display_options,
                    state: &state,
                }
                .render(&mut f, full_screen)
            })?;
            Ok(())
        })?;
        Ok(TerminalApp {
            state: DisplayState {
                root: traversal.root_index,
                selected: None,
            },
            display: display_options,
            traversal: traversal,
        })
    }
}
