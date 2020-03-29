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
    ) -> Result<WalkResult, Error>
    where
        B: Backend,
    {
        use termion::event::Key::*;
        use FocussedPane::*;

        self.draw(window, traversal, display.clone(), terminal)?;
        for key in keys.filter_map(Result::ok) {
            self.reset_message();
            match key {
                Char('?') => self.toggle_help_pane(window),
                Char('\t') => {
                    self.cycle_focus(window);
                }
                Ctrl('c') => break,
                Char('q') | Esc => match self.focussed {
                    Main => break,
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
        Ok(WalkResult {
            num_errors: traversal.io_errors,
        })
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
        self.state.process_events(
            &mut self.window,
            &mut self.traversal,
            &mut self.display,
            terminal,
            keys,
        )
    }

    pub fn initialize<B>(
        terminal: &mut Terminal<B>,
        options: WalkOptions,
        input: Vec<PathBuf>,
        mode: Interaction,
    ) -> Result<TerminalApp, Error>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;
        let mut display_options: DisplayOptions = options.clone().into();
        display_options.byte_vis = ByteVisualization::Bar;
        let mut window = MainWindow::default();
        let (keys_tx, keys_rx) = crossbeam_channel::unbounded();
        match mode {
            Interaction::None => drop(keys_tx),
            Interaction::Full => drop(std::thread::spawn(move || {
                let keys = std::io::stdin().keys();
                for key in keys {
                    if let Err(_) = keys_tx.try_send(key) {
                        break;
                    }
                }
            })),
        }

        let fetch_buffered_key_events = || {
            let mut keys = Vec::new();
            while let Ok(key) = keys_rx.try_recv() {
                keys.push(key);
            }
            keys
        };

        let traversal = Traversal::from_walk(options, input, move |traversal| {
            let mut state = AppState {
                root: traversal.root_index,
                sorting: Default::default(),
                message: Some("-> scanning <-".into()),
                entries: sorted_entries(&traversal.tree, traversal.root_index, Default::default()),
                ..Default::default()
            };
            state.process_events(
                &mut window,
                traversal,
                &mut display_options,
                terminal,
                fetch_buffered_key_events().into_iter(),
            )?;
            Ok(())
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

pub enum Interaction {
    Full,
    #[allow(dead_code)]
    None,
}
