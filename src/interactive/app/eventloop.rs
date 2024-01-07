use crate::{
    crossdev,
    interactive::{
        app::navigation::Navigation,
        app_state::FocussedPane,
        sorted_entries,
        widgets::{glob_search, MainWindow, MainWindowProps},
        ByteVisualization, CursorDirection, CursorMode, DisplayOptions, EntryDataBundle,
        MarkEntryMode, SortMode,
    },
};
use anyhow::Result;
use crossbeam::channel::Receiver;
use crosstermion::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crosstermion::input::Event;
use dua::{
    traverse::{size_on_disk, EntryData, Traversal, Tree},
    WalkOptions, WalkResult,
};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tui::backend::Backend;
use tui_react::Terminal;

use super::{
    app_state::{parent_or_panic, pop_or_panic, set_entry_info_or_panic, EntryInfo},
    terminal_app::TraversalEvent,
    tree_view::TreeView,
};
use super::{
    app_state::{AppState, Cursor, ProcessingResult},
    input::input_channel,
};

impl AppState {
    pub fn navigation_mut(&mut self) -> &mut Navigation {
        self.glob_navigation
            .as_mut()
            .unwrap_or(&mut self.navigation)
    }

    pub fn navigation(&self) -> &Navigation {
        self.glob_navigation.as_ref().unwrap_or(&self.navigation)
    }

    pub fn draw<B>(
        &mut self,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) -> Result<()>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            current_path: tree_view.current_path(self.navigation().view_root),
            entries_traversed: tree_view.traversal.entries_traversed,
            total_bytes: tree_view.traversal.total_bytes,
            start: tree_view.traversal.start,
            elapsed: tree_view.traversal.elapsed,
            display,
            state: self,
        };

        let mut cursor = Cursor::default();
        let result = draw_window(window, props, terminal, &mut cursor);

        if cursor.show {
            _ = terminal.show_cursor();
            _ = terminal.set_cursor(cursor.x, cursor.y);
        } else {
            _ = terminal.hide_cursor();
        }

        result
    }

    pub fn process_events<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        walk_options: &WalkOptions,
        events: Receiver<Event>,
        traversal_events: Receiver<TraversalEvent>,
    ) -> Result<ProcessingResult>
    where
        B: Backend,
    {
        {
            let tree_view = self.tree_view(traversal);
            self.draw(window, &tree_view, *display, terminal)?;
        }

        loop {
            crossbeam::select! {
                recv(events) -> event => {
                    let Ok(event) = event else {
                        continue;
                    };
                    let result = self.process_event(
                        window,
                        traversal,
                        display,
                        terminal,
                        event)?;
                    if let Some(processing_result) = result {
                        return Ok(processing_result);
                    }
                },
                recv(traversal_events) -> event => {
                    let Ok(event) = event else {
                        continue;
                    };
                    if self.process_traversal_event(traversal, walk_options, event) {
                        self.update_state(traversal);
                        let result = self.process_event(
                            window,
                            traversal,
                            display,
                            terminal,
                            Event::Key(refresh_key()))?;
                        if let Some(processing_result) = result {
                            return Ok(processing_result);
                        }
                    }
                }
            }
        }
        // TODO: do we need this?
        // Ok(ProcessingResult::Finished(WalkResult {
        //     num_errors: traversal.io_errors,
        // }))
    }

    // TODO: do we need this?
    //         default(Duration::from_millis(250)) => {
    //             // No events or new entries received, but we still need
    //             // to keep updating the status message regularly.
    //             if update(&mut t, None)? {
    //                 return Ok(None);
    //             }
    //         }
    //     }
    // }

    fn process_traversal_event<'a>(
        &mut self,
        t: &'a mut Traversal,
        walk_options: &'a WalkOptions,
        event: TraversalEvent,
    ) -> bool {
        match event {
            TraversalEvent::Entry(entry, root_path, device_id) => {
                t.entries_traversed += 1;
                let mut data = EntryData::default();
                match entry {
                    Ok(entry) => {
                        data.name = if entry.depth < 1 {
                            (*root_path).clone()
                        } else {
                            entry.file_name.into()
                        };

                        let mut file_size = 0u128;
                        let mut mtime: SystemTime = UNIX_EPOCH;
                        match &entry.client_state {
                            Some(Ok(ref m)) => {
                                if !m.is_dir()
                                    && (walk_options.count_hard_links
                                        || self.traversal_state.inodes.add(m))
                                    && (walk_options.cross_filesystems
                                        || crossdev::is_same_device(device_id, m))
                                {
                                    if walk_options.apparent_size {
                                        file_size = m.len() as u128;
                                    } else {
                                        file_size = size_on_disk(&entry.parent_path, &data.name, m)
                                            .unwrap_or_else(|_| {
                                                t.io_errors += 1;
                                                data.metadata_io_error = true;
                                                0
                                            })
                                            as u128;
                                    }
                                } else {
                                    data.entry_count = Some(0);
                                    data.is_dir = true;
                                }

                                match m.modified() {
                                    Ok(modified) => {
                                        mtime = modified;
                                    }
                                    Err(_) => {
                                        t.io_errors += 1;
                                        data.metadata_io_error = true;
                                    }
                                }
                            }
                            Some(Err(_)) => {
                                t.io_errors += 1;
                                data.metadata_io_error = true;
                            }
                            None => {}
                        }

                        match (entry.depth, self.traversal_state.previous_depth) {
                            (n, p) if n > p => {
                                self.traversal_state
                                    .directory_info_per_depth_level
                                    .push(self.traversal_state.current_directory_at_depth);
                                self.traversal_state.current_directory_at_depth = EntryInfo {
                                    size: file_size,
                                    entries_count: Some(1),
                                };
                                self.traversal_state.parent_node_idx =
                                    self.traversal_state.previous_node_idx;
                            }
                            (n, p) if n < p => {
                                for _ in n..p {
                                    set_entry_info_or_panic(
                                        &mut t.tree,
                                        self.traversal_state.parent_node_idx,
                                        self.traversal_state.current_directory_at_depth,
                                    );
                                    let dir_info = pop_or_panic(
                                        &mut self.traversal_state.directory_info_per_depth_level,
                                    );

                                    self.traversal_state.current_directory_at_depth.size +=
                                        dir_info.size;
                                    self.traversal_state
                                        .current_directory_at_depth
                                        .add_count(&dir_info);

                                    self.traversal_state.parent_node_idx = parent_or_panic(
                                        &mut t.tree,
                                        self.traversal_state.parent_node_idx,
                                    );
                                }
                                self.traversal_state.current_directory_at_depth.size += file_size;
                                *self
                                    .traversal_state
                                    .current_directory_at_depth
                                    .entries_count
                                    .get_or_insert(0) += 1;
                                set_entry_info_or_panic(
                                    &mut t.tree,
                                    self.traversal_state.parent_node_idx,
                                    self.traversal_state.current_directory_at_depth,
                                );
                            }
                            _ => {
                                self.traversal_state.current_directory_at_depth.size += file_size;
                                *self
                                    .traversal_state
                                    .current_directory_at_depth
                                    .entries_count
                                    .get_or_insert(0) += 1;
                            }
                        };

                        data.mtime = mtime;
                        data.size = file_size;
                        let entry_index = t.tree.add_node(data);

                        t.tree
                            .add_edge(self.traversal_state.parent_node_idx, entry_index, ());
                        self.traversal_state.previous_node_idx = entry_index;
                        self.traversal_state.previous_depth = entry.depth;
                    }
                    Err(_) => {
                        if self.traversal_state.previous_depth == 0 {
                            data.name = (*root_path).clone();
                            let entry_index = t.tree.add_node(data);
                            t.tree
                                .add_edge(self.traversal_state.parent_node_idx, entry_index, ());
                        }

                        t.io_errors += 1
                    }
                }

                if let Some(throttle) = &self.traversal_state.throttle {
                    if throttle.can_update() {
                        return true;
                    }
                }
            }
            TraversalEvent::Finished(io_errors) => {
                t.io_errors += io_errors;

                self.traversal_state.throttle = None;
                self.traversal_state
                    .directory_info_per_depth_level
                    .push(self.traversal_state.current_directory_at_depth);
                self.traversal_state.current_directory_at_depth = EntryInfo::default();
                for _ in 0..self.traversal_state.previous_depth {
                    let dir_info =
                        pop_or_panic(&mut self.traversal_state.directory_info_per_depth_level);
                    self.traversal_state.current_directory_at_depth.size += dir_info.size;
                    self.traversal_state
                        .current_directory_at_depth
                        .add_count(&dir_info);

                    set_entry_info_or_panic(
                        &mut t.tree,
                        self.traversal_state.parent_node_idx,
                        self.traversal_state.current_directory_at_depth,
                    );
                    self.traversal_state.parent_node_idx =
                        parent_or_panic(&mut t.tree, self.traversal_state.parent_node_idx);
                }
                let root_size = t.recompute_root_size();
                set_entry_info_or_panic(
                    &mut t.tree,
                    t.root_index,
                    EntryInfo {
                        size: root_size,
                        entries_count: (t.entries_traversed > 0).then_some(t.entries_traversed),
                    },
                );
                t.total_bytes = Some(root_size);
                t.elapsed = Some(t.start.elapsed());

                self.is_scanning = false;

                return true;
            }
        }
        return false;
    }

    fn update_state<'a>(&mut self, traversal: &'a Traversal) {
        let received_events = self.traversal_state.received_event;
        if !received_events {
            self.navigation_mut().view_root = traversal.root_index;
        }
        self.entries = sorted_entries(
            &traversal.tree,
            self.navigation().view_root,
            self.sorting,
            self.glob_root(),
        );
        if !received_events {
            self.navigation_mut().selected = self.entries.first().map(|b| b.index);
        }
        self.reset_message(); // force "scanning" to appear
    }

    fn process_event<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        event: Event,
    ) -> Result<Option<ProcessingResult>>
    where
        B: Backend,
    {
        use crosstermion::crossterm::event::KeyCode::*;
        use FocussedPane::*;

        let key = match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key.code != KeyCode::Char('\r') {
                    self.traversal_state.received_event = true;
                }
                key
            },
            Event::Resize(_, _) => refresh_key(),
            _ => return Ok(None),
        };

        self.reset_message();

        let glob_focussed = self.focussed == Glob;
        let mut tree_view = self.tree_view(traversal);
        let mut handled = true;
        match key.code {
            Esc => {
                if let Some(value) = self.handle_quit(&mut tree_view, window) {
                    return Ok(Some(value?));
                }
            }
            Tab => {
                self.cycle_focus(window);
            }
            Char('/') if !glob_focussed => {
                self.toggle_glob_search(window);
            }
            Char('?') if !glob_focussed => self.toggle_help_pane(window),
            Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) && !glob_focussed => {
                return Ok(Some(ProcessingResult::ExitRequested(WalkResult {
                    num_errors: tree_view.traversal.io_errors,
                })))
            }
            Char('q') if !glob_focussed => {
                if let Some(result) = self.handle_quit(&mut tree_view, window) {
                    return Ok(Some(result?));
                }
            }
            _ => {
                handled = false;
            }
        }

        if !handled {
            match self.focussed {
                Mark => self.dispatch_to_mark_pane(key, window, &mut tree_view, *display, terminal),
                Help => {
                    window
                        .help_pane
                        .as_mut()
                        .expect("help pane")
                        .process_events(key);
                }
                Glob => {
                    let glob_pane = window.glob_pane.as_mut().expect("glob pane");
                    match key.code {
                        Enter => self.search_glob_pattern(&mut tree_view, &glob_pane.input),
                        _ => glob_pane.process_events(key),
                    }
                }
                Main => match key.code {
                    Char('O') => self.open_that(&tree_view),
                    Char(' ') => self.mark_entry(
                        CursorMode::KeepPosition,
                        MarkEntryMode::Toggle,
                        window,
                        &tree_view,
                    ),
                    Char('x') => self.mark_entry(
                        CursorMode::Advance,
                        MarkEntryMode::MarkForDeletion,
                        window,
                        &tree_view,
                    ),
                    Char('a') => self.mark_all_entries(MarkEntryMode::Toggle, window, &tree_view),
                    Char('o') | Char('l') | Enter | Right => {
                        self.enter_node_with_traversal(&tree_view)
                    }
                    Char('H') | Home => self.change_entry_selection(CursorDirection::ToTop),
                    Char('G') | End => self.change_entry_selection(CursorDirection::ToBottom),
                    PageUp => self.change_entry_selection(CursorDirection::PageUp),
                    Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.change_entry_selection(CursorDirection::PageUp)
                    }
                    Char('k') | Up => self.change_entry_selection(CursorDirection::Up),
                    Char('j') | Down => self.change_entry_selection(CursorDirection::Down),
                    PageDown => self.change_entry_selection(CursorDirection::PageDown),
                    Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.change_entry_selection(CursorDirection::PageDown)
                    }
                    Char('s') => self.cycle_sorting(&tree_view),
                    Char('m') => self.cycle_mtime_sorting(&tree_view),
                    Char('c') => self.cycle_count_sorting(&tree_view),
                    Char('g') => display.byte_vis.cycle(),
                    Char('d') => self.mark_entry(
                        CursorMode::Advance,
                        MarkEntryMode::Toggle,
                        window,
                        &tree_view,
                    ),
                    Char('u') | Char('h') | Backspace | Left => {
                        self.exit_node_with_traversal(&tree_view)
                    }
                    _ => {}
                },
            };
        }
        self.draw(window, &tree_view, *display, terminal)?;

        Ok(None)
    }

    fn tree_view<'a>(&mut self, traversal: &'a mut Traversal) -> TreeView<'a> {
        TreeView {
            traversal,
            glob_tree_root: self.glob_navigation.as_ref().map(|n| n.tree_root),
        }
    }

    fn search_glob_pattern(&mut self, tree_view: &mut TreeView<'_>, glob_pattern: &str) {
        use FocussedPane::*;
        match glob_search(tree_view.tree(), self.navigation.view_root, glob_pattern) {
            Ok(matches) if matches.is_empty() => {
                self.message = Some("No match found".into());
            }
            Ok(matches) => {
                if let Some(glob_source) = &self.glob_navigation {
                    tree_view.tree_mut().remove_node(glob_source.tree_root);
                }

                let tree_root = tree_view.tree_mut().add_node(EntryData::default());
                let glob_source = Navigation {
                    tree_root,
                    view_root: tree_root,
                    selected: Some(tree_root),
                    ..Default::default()
                };
                self.glob_navigation = Some(glob_source);

                for idx in matches {
                    tree_view.tree_mut().add_edge(tree_root, idx, ());
                }

                let glob_tree_view = TreeView {
                    traversal: tree_view.traversal,
                    glob_tree_root: Some(tree_root),
                };
                let new_entries = glob_tree_view.sorted_entries(tree_root, self.sorting);

                let new_entries = self
                    .navigation_mut()
                    .selected
                    .map(|previously_selected| (previously_selected, new_entries));

                self.enter_node(new_entries);
                self.focussed = Main;
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    fn handle_quit(
        &mut self,
        tree_view: &mut TreeView<'_>,
        window: &mut MainWindow,
    ) -> Option<std::result::Result<ProcessingResult, anyhow::Error>> {
        use FocussedPane::*;
        match self.focussed {
            Main => {
                if self.glob_navigation.is_some() {
                    self.handle_glob_quit(tree_view, window);
                } else {
                    return Some(Ok(ProcessingResult::ExitRequested(WalkResult {
                        num_errors: tree_view.traversal.io_errors,
                    })));
                }
            }
            Mark => self.focussed = Main,
            Help => {
                self.focussed = Main;
                window.help_pane = None
            }
            Glob => {
                self.handle_glob_quit(tree_view, window);
            }
        }
        None
    }

    fn handle_glob_quit(&mut self, tree_view: &mut TreeView<'_>, window: &mut MainWindow) {
        use FocussedPane::*;
        self.focussed = Main;
        if let Some(glob_source) = &self.glob_navigation {
            tree_view.tree_mut().remove_node(glob_source.tree_root);
        }
        self.glob_navigation = None;
        window.glob_pane = None;

        tree_view.glob_tree_root.take();
        self.entries = tree_view.sorted_entries(self.navigation().view_root, self.sorting);
    }
}

pub fn draw_window<B>(
    window: &mut MainWindow,
    props: MainWindowProps<'_>,
    terminal: &mut Terminal<B>,
    cursor: &mut Cursor,
) -> Result<()>
where
    B: Backend,
{
    let area = terminal.pre_render()?;
    window.render(props, area, terminal, cursor);
    terminal.post_render()?;

    Ok(())
}

pub fn refresh_key() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::ALT)
}
