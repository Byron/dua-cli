use std::path::PathBuf;

use crossbeam::channel::Receiver;
use crosstermion::input::Event;
use dua::{traverse::{Traversal, Tree, EntryData}, WalkResult, WalkOptions};
use tui::prelude::Backend;
use tui_react::Terminal;
use anyhow::Result;

use crate::interactive::widgets::MainWindow;

use super::{DisplayOptions, ByteVisualization, sorted_entries, refresh_key, app_state::{ProcessingResult, AppState}};


/// State and methods representing the interactive disk usage analyser for the terminal
pub struct TerminalApp {
    pub traversal: Traversal,
    pub display: DisplayOptions,
    pub state: AppState,
    pub window: MainWindow,
}

type KeyboardInputAndApp = (crossbeam::channel::Receiver<Event>, TerminalApp);

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
                std::iter::once(Event::Key(refresh_key())),
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
        keys_rx: Receiver<Event>,
    ) -> Result<Option<KeyboardInputAndApp>>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;
        
        let mut display: DisplayOptions = options.clone().into();
        display.byte_vis = ByteVisualization::PercentageAndBar;
        
        let mut window = MainWindow::default();

        // #[inline]
        // fn fetch_buffered_key_events(keys_rx: &Receiver<Event>) -> Vec<Event> {
        //     let mut keys = Vec::new();
        //     while let Ok(key) = keys_rx.try_recv() {
        //         keys.push(key);
        //     }
        //     keys
        // }

        let mut state = AppState {
            is_scanning: false,
            ..Default::default()
        };

        // let mut received_events = false;
        // let traversal =
        //     Traversal::from_walk(options, input_paths, &keys_rx, |traversal, event| {
        //         if !received_events {
        //             state.navigation_mut().view_root = traversal.root_index;
        //         }
        //         state.entries = sorted_entries(
        //             &traversal.tree,
        //             state.navigation().view_root,
        //             state.sorting,
        //             state.glob_root(),
        //         );
        //         if !received_events {
        //             state.navigation_mut().selected = state.entries.first().map(|b| b.index);
        //         }
        //         state.reset_message(); // force "scanning" to appear

        //         let mut events = fetch_buffered_key_events(&keys_rx);
        //         if let Some(event) = event {
        //             // This update is triggered by a user event, insert it
        //             // before any events fetched later.
        //             events.insert(0, event);
        //         }
        //         received_events |= !events.is_empty();

        //         let should_exit = match state.process_events(
        //             &mut window,
        //             traversal,
        //             &mut display,
        //             terminal,
        //             events.into_iter(),
        //         )? {
        //             ProcessingResult::ExitRequested(_) => true,
        //             ProcessingResult::Finished(_) => false,
        //         };

        //         Ok(should_exit)
        //     })?;

        // let traversal = match traversal {
        //     Some(t) => t,
        //     None => return Ok(None),
        // };

        // state.is_scanning = false;
        // if !received_events {
            // }

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
