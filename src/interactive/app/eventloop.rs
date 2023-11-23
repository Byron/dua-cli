use crate::interactive::{
    sorted_entries,
    widgets::{MainWindow, MainWindowProps},
    ByteVisualization, CursorDirection, CursorMode, DisplayOptions, EntryDataBundle, MarkEntryMode,
    SortMode,
};
use anyhow::Result;
use crosstermion::input::{input_channel, Event, Key};
use dua::{
    traverse::{Traversal, TreeIndex},
    WalkOptions, WalkResult,
};
use std::{collections::BTreeMap, path::PathBuf};
use tui::backend::Backend;
use tui_react::Terminal;

#[derive(Default, Copy, Clone)]
pub enum FocussedPane {
    #[default]
    Main,
    Help,
    Mark,
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
    pub is_scanning: bool,
}

pub enum ProcessingResult {
    Finished(WalkResult),
    ExitRequested(WalkResult),
}

impl AppState {
    pub fn draw<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &Traversal,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            traversal,
            display,
            state: self,
        };
        draw_window(window, props, terminal)
    }

    pub fn process_events<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: impl Iterator<Item = Event>,
    ) -> Result<ProcessingResult>
    where
        B: Backend,
    {
        use crosstermion::input::Key::*;
        use FocussedPane::*;

        self.draw(window, traversal, *display, terminal)?;
        for event in events {
            let key = match event {
                Event::Key(key) => key,
                Event::Resize(_, _) => Alt('\r'),
            };

            self.reset_message();
            match key {
                Char('?') => self.toggle_help_pane(window),
                Char('\t') => {
                    self.cycle_focus(window);
                }
                Ctrl('c') => {
                    return Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: traversal.io_errors,
                    }))
                }
                Char('q') | Esc => match self.focussed {
                    Main => {
                        return Ok(ProcessingResult::ExitRequested(WalkResult {
                            num_errors: traversal.io_errors,
                        }))
                    }
                    Mark => self.focussed = Main,
                    Help => {
                        self.focussed = Main;
                        window.help_pane = None
                    }
                },
                _ => {}
            }

            match self.focussed {
                Mark => self.dispatch_to_mark_pane(key, window, traversal, *display, terminal),
                Help => {
                    window
                        .help_pane
                        .as_mut()
                        .expect("help pane")
                        .process_events(key);
                }
                Main => match key {
                    Char('O') => self.open_that(traversal),
                    Char(' ') => self.mark_entry(
                        CursorMode::KeepPosition,
                        MarkEntryMode::Toggle,
                        window,
                        traversal,
                    ),
                    Char('d') => self.mark_entry(
                        CursorMode::Advance,
                        MarkEntryMode::Toggle,
                        window,
                        traversal,
                    ),
                    Char('x') => self.mark_entry(
                        CursorMode::Advance,
                        MarkEntryMode::MarkForDeletion,
                        window,
                        traversal,
                    ),
                    Char('a') => self.mark_all_entries(MarkEntryMode::Toggle, window, traversal),
                    Char('u') | Char('h') | Backspace | Left => {
                        self.exit_node_with_traversal(traversal)
                    }
                    Char('o') | Char('l') | Char('\n') | Right => {
                        self.enter_node_with_traversal(traversal)
                    }
                    Char('H') | Home => self.change_entry_selection(CursorDirection::ToTop),
                    Char('G') | End => self.change_entry_selection(CursorDirection::ToBottom),
                    Ctrl('u') | PageUp => self.change_entry_selection(CursorDirection::PageUp),
                    Char('k') | Up => self.change_entry_selection(CursorDirection::Up),
                    Char('j') | Down => self.change_entry_selection(CursorDirection::Down),
                    Ctrl('d') | PageDown => self.change_entry_selection(CursorDirection::PageDown),
                    Char('s') => self.cycle_sorting(traversal),
                    Char('m') => self.cycle_mtime_sorting(traversal),
                    Char('g') => display.byte_vis.cycle(),
                    _ => {}
                },
            };
            self.draw(window, traversal, *display, terminal)?;
        }
        Ok(ProcessingResult::Finished(WalkResult {
            num_errors: traversal.io_errors,
        }))
    }
}

pub fn draw_window<B>(
    window: &mut MainWindow,
    props: MainWindowProps,
    terminal: &mut Terminal<B>,
) -> Result<()>
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

type KeyboardInputAndApp = (std::sync::mpsc::Receiver<Event>, TerminalApp);

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
                std::iter::once(Event::Key(Key::Alt('\r'))),
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
        mode: Interaction,
    ) -> Result<Option<KeyboardInputAndApp>>
    where
        B: Backend,
    {
        terminal.hide_cursor()?;
        terminal.clear()?;
        let mut display: DisplayOptions = options.clone().into();
        display.byte_vis = ByteVisualization::PercentageAndBar;
        let mut window = MainWindow::default();
        let keys_rx = match mode {
            Interaction::None => {
                let (_, keys_rx) = std::sync::mpsc::channel();
                keys_rx
            }
            Interaction::Full => input_channel(),
        };

        let fetch_buffered_key_events = || {
            let mut keys = Vec::new();
            while let Ok(key) = keys_rx.try_recv() {
                keys.push(key);
            }
            keys
        };

        let mut state = None::<AppState>;
        let mut received_events = false;
        let traversal = Traversal::from_walk(options, input_paths, |traversal| {
            let s = match state.as_mut() {
                Some(s) => {
                    s.entries = sorted_entries(&traversal.tree, s.root, s.sorting);
                    if !received_events {
                        s.selected = s.entries.get(0).map(|b| b.index);
                    }
                    s
                }
                None => {
                    state = Some({
                        let sorting = Default::default();
                        let entries =
                            sorted_entries(&traversal.tree, traversal.root_index, sorting);
                        AppState {
                            root: traversal.root_index,
                            sorting,
                            selected: entries.get(0).map(|b| b.index),
                            entries,
                            is_scanning: true,
                            ..Default::default()
                        }
                    });
                    state.as_mut().expect("state to be present, we just set it")
                }
            };
            s.reset_message(); // force "scanning" to appear
            let events = fetch_buffered_key_events();
            received_events |= !events.is_empty();

            let should_exit = match s.process_events(
                &mut window,
                traversal,
                &mut display,
                terminal,
                events.into_iter(),
            )? {
                ProcessingResult::ExitRequested(_) => true,
                ProcessingResult::Finished(_) => false,
            };
            Ok(should_exit)
        })?;
        let traversal = match traversal {
            Some(t) => t,
            None => return Ok(None),
        };

        Ok(Some((keys_rx, {
            let mut app = TerminalApp {
                state: {
                    let mut s = state.unwrap_or_else(|| {
                        let sorting = Default::default();
                        let root = traversal.root_index;
                        let entries = sorted_entries(&traversal.tree, root, sorting);
                        AppState {
                            root,
                            entries,
                            sorting,
                            ..Default::default()
                        }
                    });
                    s.is_scanning = false;
                    s.entries = sorted_entries(&traversal.tree, s.root, s.sorting);
                    s.selected = if received_events {
                        s.selected.or_else(|| s.entries.get(0).map(|b| b.index))
                    } else {
                        s.entries.get(0).map(|b| b.index)
                    };
                    s
                },
                display,
                traversal,
                window,
            };
            app.refresh_view(terminal);
            app
        })))
    }
}

pub enum Interaction {
    Full,
    #[allow(dead_code)]
    None,
}
