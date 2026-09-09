use crate::interactive::state::FilesystemScan;
use crate::interactive::{
    CursorDirection, CursorMode, DisplayOptions, EntryCheck, MarkEntryMode,
    app::navigation::Navigation,
    state::FocussedPane,
    widgets::{MainWindow, MainWindowProps, glob_search},
};
use anyhow::{Context, Result, bail};
use crossbeam::channel::Receiver;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Colored,
};
use dua::{
    Config, WalkResult,
    traverse::{BackgroundTraversal, EntryData, Traversal, TreeIndex},
};
use std::{path::PathBuf, sync::Arc};
use tui::{
    Terminal, backend::Backend, buffer::Buffer, layout::Rect, style::Color, widgets::Widget,
};

use super::deletion::DeletionEvent;
use super::notification;
use super::state::{AppState, Cursor};
#[cfg(unix)]
use super::terminal::suspend_terminal;
use super::terminal::write_snapshot_atomically;
use super::tree_view::TreeView;

/// Information needed to extend the traversal one directory upward:
///
/// 0. The parent directory to scan.
/// 1. Existing subtree roots to preserve as `(filesystem path, tree node)` pairs.
/// 2. Whether the current root represents a complete directory and must become a child of a new
///    root. If false, it is a synthetic container that can be repurposed as the parent.
type ParentScan = (PathBuf, Vec<(PathBuf, TreeIndex)>, bool);

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
        config: &Config,
    ) -> Result<()>
    where
        B: Backend,
    {
        let props = MainWindowProps {
            current_path: self.display_path(tree_view),
            entries_traversed: self.stats.entries_traversed,
            total_bytes: tree_view.total_size(),
            start: self.stats.start,
            elapsed: self.stats.elapsed,
            display,
            state: self,
            config,
        };

        let mut cursor = Cursor::default();
        let result = draw_window(window, props, terminal, &mut cursor);

        if cursor.show {
            _ = terminal.show_cursor();
            _ = terminal.set_cursor_position((cursor.x, cursor.y));
        } else {
            _ = terminal.hide_cursor();
        }

        result
    }

    pub fn traverse(
        &mut self,
        traversal: &Traversal,
        snapshot_export: Option<(PathBuf, Option<i32>)>,
    ) -> Result<()> {
        if self.read_only {
            bail!(self.language.ui_text().snapshots_read_only);
        }
        if self.is_deleting() {
            bail!(self.language.ui_text().deletion_running);
        }
        let bg_traversal = if let Some(hub) = &self.clean_hub {
            BackgroundTraversal::start_clean(
                traversal.root_index,
                &self.walk_options,
                hub.inputs.clone(),
                hub.depth,
                None,
            )?
        } else {
            BackgroundTraversal::start(
                traversal.root_index,
                &self.walk_options,
                self.root_paths.clone(),
                self.walk_options
                    .ignore_patterns
                    .as_ref()
                    .map(|_| self.root_paths.as_slice()),
                false,
                true,
            )?
        };
        self.navigation_mut().view_root = traversal.root_index;
        self.scan = Some(FilesystemScan {
            active_traversal: bg_traversal,
            previous_selection: None,
            previous_cleanup_view: None,
            snapshot_export,
        });
        Ok(())
    }

    pub(super) fn can_scan_parent(&self, tree: &TreeView<'_>) -> bool {
        self.parent_scan_target(tree).is_some()
    }

    /// Return the path displayed for the current view without resolving snapshot paths on disk.
    pub(super) fn display_path(&self, tree_view: &TreeView<'_>) -> PathBuf {
        if self.read_only {
            let path = tree_view.path_of(self.navigation().view_root);
            if path.as_os_str().is_empty() {
                PathBuf::from(self.language.ui_text().snapshot_label)
            } else {
                path
            }
        } else {
            tree_view.current_path(self.navigation().view_root)
        }
    }

    fn parent_scan_target(&self, tree: &TreeView<'_>) -> Option<ParentScan> {
        if self.read_only
            || self.clean_hub.is_some()
            || self.scan.is_some()
            || self.is_deleting()
            || self.glob_navigation.is_some()
            || self.navigation.view_root != tree.traversal.root_index
        {
            return None;
        }

        if let Some(current_root) = &self.root_path {
            let parent = current_root.parent()?;
            return (parent != current_root.as_path()).then(|| {
                (
                    parent.to_owned(),
                    vec![(current_root.clone(), tree.traversal.root_index)],
                    true,
                )
            });
        }

        let cwd = std::env::current_dir().ok()?;
        let mut common_parent = None;
        let mut roots = Vec::new();
        for index in tree.tree().children(tree.traversal.root_index) {
            let path = tree.path_of(index);
            let path = if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            };
            let name = path.file_name()?.to_owned();
            let parent = path.parent()?.canonicalize().ok()?;
            if common_parent
                .as_ref()
                .is_some_and(|common| common != &parent)
            {
                return None;
            }
            roots.push((parent.join(name), index));
            common_parent = Some(parent);
        }
        common_parent.map(|parent| (parent, roots, false))
    }

    fn scan_parent(&mut self, tree: &mut TreeView<'_>) -> Result<()> {
        if self.block_deletion_changes() {
            return Ok(());
        }
        if self.read_only {
            self.message = Some(self.language.ui_text().snapshots_read_only.into());
            return Ok(());
        }
        if self.scan.is_some() {
            self.message = Some(self.language.ui_text().traversal_running.into());
            return Ok(());
        }
        let Some((parent, preexisting, wrap_root)) = self.parent_scan_target(tree) else {
            self.message = Some(self.language.ui_text().top_level.into());
            return Ok(());
        };

        let old_root = tree.traversal.root_index;
        let new_root = if wrap_root {
            tree.tree_mut().add_root(
                &parent,
                EntryData {
                    is_dir: true,
                    ..EntryData::default()
                },
            )
        } else {
            old_root
        };
        let pattern_roots = self
            .walk_options
            .ignore_patterns
            .as_ref()
            .map(|_| std::slice::from_ref(&parent));
        let active_traversal = match BackgroundTraversal::start_incremental(
            new_root,
            &self.walk_options,
            vec![parent.clone()],
            pattern_roots,
            true,
            false,
            preexisting
                .iter()
                .map(|(path, index)| (path.clone(), *index, wrap_root))
                .collect(),
        ) {
            Ok(traversal) => traversal,
            Err(err) => {
                if wrap_root {
                    tree.tree_mut().remove_subtree(new_root);
                }
                return Err(err);
            }
        };

        let previous_selection = self.navigation.selected;
        if wrap_root {
            let current_root = self.root_path.as_ref().expect("complete root is set");
            tree.tree_mut()
                .rename(
                    old_root,
                    current_root
                        .file_name()
                        .expect("filesystem roots have no parent"),
                )
                .expect("root exists");
            tree.tree_mut()
                .update(old_root, |entry| entry.is_dir = true);
            let children = tree.tree().children(old_root).collect::<Vec<_>>();
            for child in children {
                let name = tree
                    .tree()
                    .name(child)
                    .expect("child exists")
                    .file_name()
                    .expect("children have a file name")
                    .to_owned();
                tree.tree_mut().rename(child, name).expect("child exists");
            }
            tree.tree_mut()
                .attach(new_root, old_root)
                .expect("old root is detached");
        } else {
            tree.tree_mut()
                .rename(old_root, &parent)
                .expect("root exists");
            tree.tree_mut()
                .update(old_root, |entry| entry.is_dir = true);
            for (path, index) in &preexisting {
                tree.tree_mut()
                    .rename(
                        *index,
                        path.file_name()
                            .expect("paths with a parent have a file name"),
                    )
                    .expect("preexisting node exists");
            }
        }

        tree.recompute_sizes_recursively(new_root);
        tree.traversal.root_index = new_root;
        self.navigation.tree_root = new_root;
        self.navigation.view_root = new_root;
        self.entries = tree.sorted_entries(new_root, self.sorting, self.entry_check());
        let selected = if wrap_root {
            Some(old_root)
        } else {
            previous_selection
                .filter(|selected| preexisting.iter().any(|(_, index)| index == selected))
                .or_else(|| self.entries.first().map(|entry| entry.index))
        };
        self.navigation.select(selected);
        self.update_entry_annotations(tree);

        let previous_selection = selected.and_then(|selected| {
            self.entries
                .iter()
                .position(|entry| entry.index == selected)
                .map(|position| {
                    (
                        tree.tree()
                            .name(selected)
                            .expect("selected node exists")
                            .into_owned(),
                        position,
                    )
                })
        });
        self.root_path = Some(parent.clone());
        self.root_paths = vec![parent];
        self.received_events = false;
        self.scan = Some(FilesystemScan {
            active_traversal,
            previous_selection,
            previous_cleanup_view: None,
            snapshot_export: None,
        });
        self.reset_message();
        Ok(())
    }

    fn recompute_sizes_recursively(&mut self, traversal: &mut Traversal, node_index: TreeIndex) {
        let mut tree_view = self.tree_view(traversal);
        tree_view.recompute_sizes_recursively(node_index);
    }

    fn refresh_screen<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        config: &Config,
    ) -> Result<()>
    where
        B: Backend,
    {
        let tree_view = self.tree_view(traversal);
        self.draw(window, &tree_view, *display, terminal, config)?;
        Ok(())
    }

    /// This method ends once the user quits the application or there are no more inputs to process.
    pub fn process_events<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: Receiver<Event>,
        config: &Config,
    ) -> Result<WalkResult>
    where
        B: Backend,
    {
        self.refresh_screen(window, traversal, display, terminal, config)?;

        loop {
            if let Some(result) =
                self.process_event(window, traversal, display, terminal, &events, config)?
            {
                return Ok(result);
            }
        }
    }

    pub fn process_events_once<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: Receiver<Event>,
        config: &Config,
    ) -> Result<WalkResult>
    where
        B: Backend,
    {
        self.refresh_screen(window, traversal, display, terminal, config)?;

        if let Some(result) =
            self.process_events_until_traversed(window, traversal, display, terminal, config)?
        {
            return Ok(result);
        }

        while let Ok(event) = events.try_recv() {
            if let Some(result) =
                self.process_terminal_event(window, traversal, display, terminal, event, config)?
            {
                return Ok(result);
            }
        }

        if let Some(result) =
            self.process_events_until_traversed(window, traversal, display, terminal, config)?
        {
            return Ok(result);
        }

        Ok(WalkResult {
            num_errors: self.stats.io_errors,
        })
    }

    fn process_events_until_traversed<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        config: &Config,
    ) -> Result<Option<WalkResult>>
    where
        B: Backend,
    {
        let (_keep_alive, no_events) = crossbeam::channel::bounded(0);
        while self.scan.is_some() || self.is_deleting() {
            if let Some(result) =
                self.process_event(window, traversal, display, terminal, &no_events, config)?
            {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    pub fn process_event<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        events: &Receiver<Event>,
        config: &Config,
    ) -> Result<Option<WalkResult>>
    where
        B: Backend,
    {
        if let Some(deletion) = &self.deletion {
            let input = if deletion.input_closed {
                crossbeam::channel::never()
            } else {
                events.clone()
            };
            let removals = deletion.task.events.clone();
            let tick = deletion.tick.clone();
            crossbeam::select! {
                recv(input) -> event => {
                    match event {
                        Ok(event) => {
                            if let Some(result) = self.flush_deletion(traversal, window, config)? {
                                return Ok(Some(result));
                            }
                            return self.process_terminal_event(window, traversal, display, terminal, event, config);
                        }
                        Err(_) => {
                            self.deletion.as_mut().expect("active deletion").input_closed = true;
                        }
                    }
                },
                recv(removals) -> event => {
                    let event = event.context("Deletion worker stopped unexpectedly")?;
                    let deletion = self.deletion.as_mut().expect("active deletion");
                    deletion.pending.push(event);
                    deletion.pending.extend(removals.try_iter().take(1023));
                    // Bound pending path storage as well as the worker channel. The timer is
                    // independent of traffic, so a busy worker cannot starve progress redraws.
                    if deletion.pending.len() >= 16_384
                        || deletion.pending.iter().any(|event| matches!(event, DeletionEvent::Finished { .. })) {
                        let result = self.flush_deletion(traversal, window, config)?;
                        self.refresh_screen(window, traversal, display, terminal, config)?;
                        return Ok(result);
                    }
                },
                recv(tick) -> _ => {
                    let result = self.flush_deletion(traversal, window, config)?;
                    self.refresh_screen(window, traversal, display, terminal, config)?;
                    return Ok(result);
                }
            }
        } else if let Some(FilesystemScan {
            active_traversal,
            previous_selection,
            previous_cleanup_view,
            snapshot_export,
        }) = self.scan.as_mut()
        {
            crossbeam::select! {
                recv(events) -> event => {
                    let Ok(event) = event else {
                        return Ok(Some(WalkResult { num_errors: self.stats.io_errors }));
                    };
                    let res = self.process_terminal_event(
                        window,
                        traversal,
                        display,
                        terminal,
                        event,
                        config,
                    )?;
                    if let Some(res) = res {
                        return Ok(Some(res));
                    }
                },
                recv(&active_traversal.event_rx) -> event => {
                    let event = event.context("Filesystem traversal stopped unexpectedly")?;

                    if let Some(is_finished) = active_traversal.integrate_traversal_event(traversal, event) {
                        self.stats = active_traversal.stats;
                        let previous_selection = previous_selection.clone();
                        let previous_cleanup_view = previous_cleanup_view.clone();
                        if is_finished {
                            let root_index = active_traversal.root_idx;
                            let export = snapshot_export
                                .take()
                                .map(|(path, compression_level)| {
                                    active_traversal
                                        .root_nodes()
                                        .map(|roots| (path, roots, compression_level))
                                        .context(
                                            "traversal did not produce a node for every root",
                                        )
                                })
                                .transpose()?;
                            self.recompute_sizes_recursively(traversal, root_index);
                            if let Some((path, roots, compression_level)) = export {
                                write_snapshot_atomically(
                                    &path,
                                    traversal,
                                    &roots,
                                    compression_level,
                                    self.language,
                                )?;
                            }
                            self.scan = None;
                            traversal.cost = Some(traversal.start_time.elapsed());
                        }
                        if is_finished && !self.received_events
                            && let Some((source_path, view_path)) = &previous_cleanup_view
                        {
                            let mut tree = self.tree_view(traversal);
                            if let Some(source) = tree.index_at_path(source_path) {
                                self.navigation.view_root = source;
                                if self.glob_navigation.is_some() && let Some(glob) = &window.glob {
                                    // Rebuild replaced IDs within the original search directory.
                                    self.search_glob_pattern(&mut tree, &glob.input, glob.case);
                                }
                            }
                            if let Some(view) = tree.index_at_path(view_path) {
                                self.navigation_mut().view_root = view;
                            }
                        }
                        self.update_state_during_traversal(traversal, previous_selection.as_ref(), is_finished);
                        self.refresh_screen(window, traversal, display, terminal, config)?;
                        if is_finished {
                            let message = notification::scan_finished(
                                self.language,
                                self.stats.entries_traversed,
                                self.stats.total_bytes.unwrap_or_default(),
                                self.stats.elapsed.unwrap_or_else(|| self.stats.start.elapsed()),
                                self.stats.io_errors,
                                display.byte_format,
                            );
                            if let Err(err) = notification::emit_if_unfocused(
                                config.notifications.scan_finished,
                                self.terminal_focus.is_focussed(),
                                &message,
                            ) {
                                log::debug!("Could not emit terminal notification: {err}");
                            }
                        }
                    }
                },
                default(std::time::Duration::from_secs(1)) => {
                    // Keep progress visible while discovery or filesystem I/O is waiting.
                    self.stats = active_traversal.stats;
                    let previous_selection = previous_selection.clone();
                    self.update_state_during_traversal(traversal, previous_selection.as_ref(), false);
                    self.refresh_screen(window, traversal, display, terminal, config)?;
                }
            }
        } else {
            let Ok(event) = events.recv() else {
                return Ok(Some(WalkResult {
                    num_errors: self.stats.io_errors,
                }));
            };
            let result =
                self.process_terminal_event(window, traversal, display, terminal, event, config)?;
            if let Some(processing_result) = result {
                return Ok(Some(processing_result));
            }
        }
        Ok(None)
    }

    fn update_state_during_traversal(
        &mut self,
        traversal: &mut Traversal,
        previous_selection: Option<&(PathBuf, usize)>,
        is_finished: bool,
    ) {
        let tree_view = self.tree_view(traversal);
        self.update_entries(&tree_view);

        if !self.received_events {
            if let Some(hub) = &mut self.clean_hub
                && hub.root.is_none()
                && let Some((path, _)) = previous_selection
            {
                hub.select_path(path);
            }
            let previously_selected_entry =
                previous_selection.and_then(|(selected_name, selected_idx)| {
                    self.entries
                        .iter()
                        .find(|e| e.name == *selected_name)
                        .or_else(|| self.entries.get(*selected_idx))
                });
            if let Some(selected_entry) = previously_selected_entry {
                self.navigation_mut().selected = Some(selected_entry.index);
            } else if is_finished {
                self.navigation_mut().selected = self.entries.first().map(|b| b.index);
            }
        }
        self.reset_message(); // force "scanning" to appear
    }

    pub(crate) fn entry_check(&self) -> EntryCheck {
        EntryCheck::new(
            self.scan.is_some() || self.is_deleting(),
            self.allow_entry_check && !self.read_only,
        )
    }

    fn process_terminal_event<B>(
        &mut self,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: &mut DisplayOptions,
        terminal: &mut Terminal<B>,
        event: Event,
        config: &Config,
    ) -> Result<Option<WalkResult>>
    where
        B: Backend,
    {
        use FocussedPane::{Glob, Help, Main, Mark};

        let key = match event {
            Event::FocusGained => {
                self.terminal_focus.observe(&Event::FocusGained);
                return Ok(None);
            }
            Event::FocusLost => {
                self.terminal_focus.observe(&Event::FocusLost);
                return Ok(None);
            }
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key != refresh_key() && !config.keys.repaint.matches(key) {
                    self.received_events = true;
                }
                key
            }
            Event::Resize(_, _) => refresh_key(),
            _ => return Ok(None),
        };

        self.reset_message();

        let glob_focussed = self.focussed == Glob;
        let mut tree_view = self.tree_view(traversal);
        let keys = &config.keys;

        let close_pane = keys.close_pane.matches(key);
        let quit = !glob_focussed && keys.quit.matches(key);
        let mut handled = true;
        if self.process_clean_key(key, window, &mut tree_view, config)? {
            self.pending_exit = false;
        } else if keys.esc_navigates_back && close_pane && self.focussed == Main {
            self.pending_exit = false;
            self.exit_node_with_traversal(&tree_view, &keys.scan_parent.primary());
        } else if close_pane || quit {
            if let Some(result) = self.handle_quit(&mut tree_view, window) {
                result?;
                if let Some(result) = self.exit_after_deletion() {
                    return Ok(Some(result));
                }
            }
        } else {
            self.pending_exit = false;
            match key {
                #[cfg(unix)]
                _ if keys.suspend.matches(key) => {
                    suspend_terminal(terminal, config.notifications.any_enabled())?;
                }
                _ if keys.repaint.matches(key) => {
                    terminal
                        .backend_mut()
                        .clear()
                        .map_err(|err| anyhow::Error::msg(err.to_string()))?;
                    terminal.swap_buffers();
                }
                _ if keys.cycle_panes.matches(key) => {
                    self.cycle_focus(window);
                }
                _ if !glob_focussed && keys.toggle_right_panes.matches(key) => {
                    self.toggle_right_panes(window);
                }
                _ if !glob_focussed && keys.open_search.matches(key) => {
                    self.toggle_glob_search(window);
                }
                _ if !glob_focussed && keys.toggle_help.matches(key) => {
                    self.toggle_help_pane(window);
                }
                _ if !glob_focussed && keys.quit_immediately.matches(key) => {
                    if let Some(result) = self.exit_after_deletion() {
                        return Ok(Some(result));
                    }
                }
                _ => {
                    handled = false;
                }
            }
        }

        if !handled {
            match self.focussed {
                Mark => self.dispatch_to_mark_pane(key, window, &mut tree_view, *display, config),
                Help => {
                    window
                        .help
                        .as_mut()
                        .expect("help pane")
                        .process_events(key, keys);
                }
                Glob => {
                    let glob_pane = window.glob.as_mut().expect("glob pane");
                    if keys.search_confirm.matches(key) {
                        self.search_glob_pattern(&mut tree_view, &glob_pane.input, glob_pane.case);
                    } else {
                        glob_pane.process_events(key, keys);
                    }
                }
                Main => {
                    if keys.open_entry.matches(key) {
                        self.open_path(
                            self.navigation()
                                .selected
                                .map(|index| tree_view.path_of(index)),
                        );
                    } else if keys.toggle_mark.matches(key) {
                        self.mark_entry(
                            CursorMode::KeepPosition,
                            MarkEntryMode::Toggle,
                            window,
                            &tree_view,
                        );
                    } else if keys.mark_for_deletion.matches(key) {
                        self.mark_entry(
                            CursorMode::Advance,
                            MarkEntryMode::MarkForDeletion,
                            window,
                            &tree_view,
                        );
                    } else if keys.toggle_all.matches(key) {
                        self.mark_all_entries(MarkEntryMode::Toggle, window, &tree_view);
                    } else if keys.toggle_cleanup.matches(key) {
                        self.toggle_cleanup_candidates(&tree_view);
                    } else if keys.mark_cleanup.matches(key) {
                        self.mark_cleanup_candidates(window, &tree_view);
                    } else if keys.toggle_gitignore.matches(key) {
                        self.toggle_gitignored_entries(&tree_view);
                    } else if keys.mark_gitignore.matches(key) {
                        self.mark_gitignored_entries(window, &tree_view);
                    } else if keys.descend.matches(key) {
                        self.enter_node_with_traversal(&tree_view);
                    } else if keys.refresh_selected.matches(key) {
                        self.refresh(&mut tree_view, window, Refresh::Selected)?;
                    } else if keys.refresh_all.matches(key) {
                        self.refresh(&mut tree_view, window, Refresh::AllInView)?;
                    } else if let Some(direction) = CursorDirection::from_key(key, keys) {
                        self.change_entry_selection(direction);
                    } else if keys.sort_by_size.matches(key) {
                        self.cycle_sorting(&tree_view);
                    } else if keys.sort_by_mtime.matches(key) {
                        self.cycle_mtime_sorting(&tree_view);
                    } else if keys.cycle_mtime_mode.matches(key) {
                        self.cycle_mtime_sort_mode(&tree_view);
                    } else if keys.sort_by_count.matches(key) {
                        self.cycle_count_sorting(&tree_view);
                    } else if keys.toggle_count_column.matches(key) {
                        self.toggle_count_column();
                    } else if keys.sort_by_name.matches(key) {
                        self.cycle_name_sorting(&tree_view);
                    } else if keys.cycle_visualization.matches(key) {
                        display.byte_vis.cycle();
                    } else if keys.toggle_mark_and_move_down.matches(key) {
                        self.mark_entry(
                            CursorMode::Advance,
                            MarkEntryMode::Toggle,
                            window,
                            &tree_view,
                        );
                    } else if keys.scan_parent.matches(key) {
                        self.scan_parent(&mut tree_view)?;
                    } else if keys.ascend.matches(key) {
                        self.exit_node_with_traversal(&tree_view, &keys.scan_parent.primary());
                    }
                }
            }
        }
        self.draw(window, &tree_view, *display, terminal, config)?;

        Ok(None)
    }

    pub(super) fn refresh(
        &mut self,
        tree: &mut TreeView<'_>,
        window: &mut MainWindow,
        what: Refresh,
    ) -> anyhow::Result<()> {
        if self.block_deletion_changes() {
            return Ok(());
        }
        if self.read_only {
            self.message = Some(self.language.ui_text().snapshots_read_only.into());
            return Ok(());
        }
        // If another traversal is already running do not do anything.
        if self.scan.is_some() {
            self.message = Some(self.language.ui_text().traversal_running.into());
            return Ok(());
        }

        let previous_selection = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, entry)| Some(entry.index) == self.navigation().selected)
            .map(|(position, entry)| (entry.name.clone(), position))
            .or_else(|| {
                self.clean_hub
                    .as_ref()?
                    .selected_path()
                    .map(|path| (path.to_owned(), 0))
            });

        // If we are displaying the root of the glob search results then cancel the search.
        if let Some(glob_tree_root) = tree.glob_tree_root
            && glob_tree_root == self.navigation().view_root
        {
            self.quit_glob_mode(tree, window);
        }

        let previous_cleanup_view = (self.clean_hub.is_some()
            && self.navigation().view_root != tree.traversal.root_index)
            .then(|| {
                (
                    tree.path_of(self.navigation.view_root),
                    tree.path_of(self.navigation().view_root),
                )
            });
        let remove_root_node = matches!(what, Refresh::Selected);
        let index = if remove_root_node {
            let Some(selected) = self
                .clean_hub
                .as_ref()
                .filter(|hub| hub.root.is_none())
                .and_then(|hub| hub.selected_members().first().copied())
                .or(self.navigation().selected)
            else {
                return Ok(());
            };
            selected
        } else {
            self.navigation().view_root
        };
        let (indices, parent_index, active_traversal) = if self.clean_hub.is_some() {
            let (indices, traversal) = self.clean_refresh(tree, index)?;
            (indices, tree.traversal.root_index, traversal)
        } else {
            let parent = if remove_root_node {
                tree.fs_parent_of(index).expect("selection has a parent")
            } else {
                index
            };
            let multi_root = self.navigation().view_root == tree.traversal.root_index
                && self.root_paths.len() > 1;
            let paths = if multi_root && !remove_root_node {
                self.root_paths.clone()
            } else {
                let path = tree.path_of(index);
                vec![if path.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    path
                }]
            };
            (
                if remove_root_node {
                    vec![index]
                } else {
                    tree.tree().children(index).collect()
                },
                parent,
                BackgroundTraversal::start(
                    parent,
                    &self.walk_options,
                    paths,
                    self.walk_options
                        .ignore_patterns
                        .as_ref()
                        .map(|_| self.root_paths.as_slice()),
                    !remove_root_node && !multi_root,
                    multi_root,
                )?,
            )
        };

        // Tree slots are reused: old marks and bookmarks must not target replacement entries.
        window.mark = None;
        self.navigation.bookmarks.clear();
        tree.traversal
            .remove_children(parent_index, &indices.into_iter().collect());
        tree.recompute_sizes_recursively(parent_index);
        if let Some(navigation) = self.glob_navigation.as_mut() {
            navigation.bookmarks.clear();
            navigation.matches = navigation
                .matches
                .iter()
                .copied()
                .filter(|index| tree.exists(*index))
                .collect();
            tree.glob_matches = Some(Arc::clone(&navigation.matches));
        }
        for navigation in
            std::iter::once(&mut self.navigation).chain(self.glob_navigation.iter_mut())
        {
            if !tree.exists(navigation.view_root) {
                navigation.view_root = navigation.tree_root;
            }
            navigation.selected = navigation.selected.filter(|index| tree.exists(*index));
        }

        self.sync_clean_hub(tree);
        self.update_entries(tree);

        self.scan = Some(FilesystemScan {
            active_traversal,
            previous_selection,
            previous_cleanup_view,
            snapshot_export: None,
        });

        self.received_events = false;
        Ok(())
    }

    pub(super) fn tree_view<'a>(&mut self, traversal: &'a mut Traversal) -> TreeView<'a> {
        let mut tree = TreeView {
            traversal,
            scope: None,
            glob_tree_root: self.glob_navigation.as_ref().map(|n| n.tree_root),
            glob_matches: self
                .glob_navigation
                .as_ref()
                .map(|navigation| Arc::clone(&navigation.matches)),
        };
        self.sync_clean_hub(&mut tree);
        tree
    }

    fn search_glob_pattern(
        &mut self,
        tree_view: &mut TreeView<'_>,
        glob_pattern: &str,
        case: gix::glob::pattern::Case,
    ) {
        use FocussedPane::Main;
        let entries = tree_view
            .children(self.navigation.view_root)
            .into_iter()
            .filter_map(|index| {
                tree_view
                    .name_in(self.navigation.view_root, index)
                    .map(|name| (index, name))
            });
        match glob_search(tree_view.tree(), entries, glob_pattern, case, self.language) {
            Ok(matches) if matches.is_empty() => {
                self.message = Some(self.language.ui_text().no_match.into());
            }
            Ok(mut matches) => {
                if let Some(glob_source) = &self.glob_navigation {
                    tree_view.tree_mut().remove_subtree(glob_source.tree_root);
                }

                matches.sort_unstable();
                let matches: Arc<[TreeIndex]> = matches.into();
                let tree_root = tree_view.tree_mut().add_detached("", EntryData::default());
                let glob_source = Navigation {
                    tree_root,
                    view_root: tree_root,
                    selected: Some(tree_root),
                    matches: Arc::clone(&matches),
                    ..Default::default()
                };
                self.glob_navigation = Some(glob_source);

                let glob_tree_view = TreeView {
                    traversal: tree_view.traversal,
                    scope: tree_view.scope.clone(),
                    glob_tree_root: Some(tree_root),
                    glob_matches: Some(matches),
                };
                let new_entries =
                    glob_tree_view.sorted_entries(tree_root, self.sorting, self.entry_check());

                let new_entries = self
                    .navigation_mut()
                    .selected
                    .map(|previously_selected| (previously_selected, new_entries));

                self.enter_node(new_entries, &glob_tree_view);
                self.focussed = Main;
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    fn handle_quit(
        &mut self,
        tree_view: &mut TreeView<'_>,
        window: &mut MainWindow,
    ) -> Option<std::result::Result<WalkResult, anyhow::Error>> {
        use FocussedPane::{Glob, Help, Main, Mark};
        match self.focussed {
            Main => {
                if self.glob_navigation.is_some() {
                    self.quit_glob_mode(tree_view, window);
                } else if window.mark.is_none() && !tree_view.traversal.is_costly() {
                    // If nothing is selected for deletion, quit instantly
                    return Some(Ok(WalkResult {
                        num_errors: self.stats.io_errors,
                    }));
                } else if !self.pending_exit {
                    self.pending_exit = true;
                } else {
                    return Some(Ok(WalkResult {
                        num_errors: self.stats.io_errors,
                    }));
                }
            }
            Mark => self.focussed = Main,
            Help => {
                self.focussed = Main;
                window.help = None;
            }
            Glob => {
                self.quit_glob_mode(tree_view, window);
            }
        }
        None
    }

    fn quit_glob_mode(&mut self, tree_view: &mut TreeView<'_>, window: &mut MainWindow) {
        use FocussedPane::Main;
        self.focussed = Main;
        if let Some(glob_source) = &self.glob_navigation {
            tree_view.tree_mut().remove_subtree(glob_source.tree_root);
        }
        self.glob_navigation = None;
        window.glob = None;

        tree_view.glob_tree_root.take();
        self.update_entries(tree_view);
    }
}

pub(super) enum Refresh {
    /// Refresh the directory currently in view
    AllInView,
    /// Refresh only the selected item
    Selected,
}

/// A [`Widget`] that renders by calling a function.
///
/// The `FunctionWidget` struct holds a function that renders into a portion of
/// a [`Buffer`] designated by a [`Rect`].
///
/// This widget can be used to create custom UI elements that are defined by a
/// rendering function. and allows for rendering functions that do not implement
/// the [`Widget`] trait.
struct FunctionWidget<F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    render: F,
}

impl<F> FunctionWidget<F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    /// Creates a new [`FunctionWidget`] with the given rendering function.
    ///
    /// The rendering function must have the signature `FnOnce(Rect, &mut
    /// Buffer)`, where:
    /// - [`Rect`] represents the available space for rendering.
    /// - [`Buffer`] is the buffer to write the rendered content to.
    ///
    /// The `FunctionWidget` can then be used to render the provided function in
    /// a user interface.
    fn new(function: F) -> FunctionWidget<F>
    where
        F: FnOnce(Rect, &mut Buffer),
    {
        FunctionWidget { render: function }
    }
}

/// Implements the [`Widget`] trait for [`FunctionWidget`].
///
/// The implementation simply calls the provided render function with the given
/// `Rect` and `Buffer`.
impl<F> Widget for FunctionWidget<F>
where
    F: FnOnce(Rect, &mut Buffer),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        (self.render)(area, buf);
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
    terminal
        .draw(|frame| {
            frame.render_widget(
                FunctionWidget::new(|area, buf| {
                    window.render(props, area, buf, cursor);
                }),
                frame.area(),
            );
            // Disabled Crossterm color commands reset attributes such as reverse
            // video, so remove colors before they reach the backend.
            if Colored::ansi_color_disabled_memoized() {
                strip_colors(frame.buffer_mut());
            }
        })
        .map_err(|err| anyhow::Error::msg(err.to_string()))?;
    Ok(())
}

fn strip_colors(buffer: &mut Buffer) {
    for cell in &mut buffer.content {
        cell.set_fg(Color::Reset).set_bg(Color::Reset);
    }
}

pub fn refresh_key() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::ALT)
}
