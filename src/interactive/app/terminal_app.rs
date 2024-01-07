use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::input::Event;
use dua::{
    traverse::{EntryData, Traversal, Tree},
    ByteFormat, WalkOptions, WalkResult,
};
use tui::prelude::Backend;
use tui_react::Terminal;

use crate::{crossdev, interactive::widgets::MainWindow};

use super::{
    app_state::{AppState, ProcessingResult, TraversalState},
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

pub type TraversalEntry =
    Result<jwalk::DirEntry<((), Option<Result<std::fs::Metadata, jwalk::Error>>)>, jwalk::Error>;

pub enum TraversalEvent {
    Entry(TraversalEntry, Arc<PathBuf>, u64),
    Finished(u64),
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

    pub fn scan(&mut self, input: Vec<PathBuf>) -> Result<Receiver<TraversalEvent>> {
        self.state.traversal_state = TraversalState::new(self.traversal.root_index);
        self.state.is_scanning = true;

        let (entry_tx, entry_rx) = crossbeam::channel::bounded(100);
        std::thread::Builder::new()
            .name("dua-fs-walk-dispatcher".to_string())
            .spawn({
                let walk_options = self.walk_options.clone();
                let mut io_errors: u64 = 0;
                move || {
                    for root_path in input.into_iter() {
                        let device_id = match crossdev::init(root_path.as_ref()) {
                            Ok(id) => id,
                            Err(_) => {
                                io_errors += 1;
                                continue;
                            }
                        };

                        let root_path = Arc::new(root_path);
                        for entry in walk_options
                            .iter_from_path(root_path.as_ref(), device_id)
                            .into_iter()
                        {
                            if entry_tx
                                .send(TraversalEvent::Entry(
                                    entry,
                                    Arc::clone(&root_path),
                                    device_id,
                                ))
                                .is_err()
                            {
                                // The channel is closed, this means the user has
                                // requested to quit the app. Abort the walking.
                                return;
                            }
                        }
                    }
                    if entry_tx.send(TraversalEvent::Finished(io_errors)).is_err() {
                        log::error!("Failed to send TraversalEvents::Finished event");
                    }
                }
            })?;

        Ok(entry_rx)
    }

    pub fn process_events<B>(
        &mut self,
        terminal: &mut Terminal<B>,
        events: Receiver<Event>,
        traversal: Receiver<TraversalEvent>,
    ) -> Result<WalkResult>
    where
        B: Backend,
    {
        match self.state.process_events(
            &mut self.window,
            &mut self.traversal,
            &mut self.display,
            terminal,
            &self.walk_options,
            events,
            traversal,
        )? {
            ProcessingResult::ExitRequested(res) => Ok(res),
        }
    }
}
