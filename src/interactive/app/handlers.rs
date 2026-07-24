use crate::interactive::{
    DisplayOptions, EntryDataBundle,
    app::tree_view::TreeView,
    widgets::{Column, GlobPane, HelpPane, MainWindow, MarkMode, MarkPane},
};
use crossterm::event::KeyEvent;
use dua::Config;
use dua::traverse::TreeIndex;
use std::{
    collections::BTreeSet,
    fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};
use tui::{Terminal, backend::Backend};

use super::{
    notification,
    state::{AppState, FocussedPane::*},
};

#[derive(Copy, Clone)]
pub enum CursorMode {
    Advance,
    KeepPosition,
}

#[derive(Copy, Clone)]
pub enum MarkEntryMode {
    Toggle,
    MarkForDeletion,
}

/// Aggregate outcome of an entire deletion or trash operation.
///
/// This combines the results for all selected entries and adds the operation's
/// wall-clock duration for the completion notification. In contrast,
/// [`EntryDeletionStats`] describes the lower-level removal of one selected entry.
struct DeletionStats {
    entries: usize,
    bytes: u128,
    errors: usize,
    elapsed: Duration,
}

/// Outcome of removing one selected entry from the filesystem and traversal.
#[derive(Default)]
struct EntryDeletionStats {
    entries: usize,
    bytes: u128,
    errors: usize,
}

pub enum CursorDirection {
    PageDown,
    Down,
    Up,
    PageUp,
    ToTop,
    ToBottom,
}

impl CursorDirection {
    pub fn move_cursor(&self, n: usize) -> usize {
        use CursorDirection::*;
        match self {
            ToTop => 0,
            ToBottom => usize::MAX,
            Down => n.saturating_add(1),
            Up => n.saturating_sub(1),
            PageDown => n.saturating_add(10),
            PageUp => n.saturating_sub(10),
        }
    }
}

impl AppState {
    pub fn open_that(&mut self, tree_view: &TreeView<'_>) {
        if let Some(idx) = self.navigation().selected {
            let path = tree_view.path_of(idx);
            if let Err(err) = open::that(&path) {
                self.message = Some(format!("Failed to open {}: {err}", path.display()));
            }
        }
    }

    pub fn exit_node_with_traversal(&mut self, tree_view: &TreeView<'_>) {
        let entries = self.entries_for_exit_node(tree_view);
        self.exit_node(entries, tree_view);
    }

    fn entries_for_exit_node(
        &self,
        tree_view: &TreeView<'_>,
    ) -> Option<(TreeIndex, Vec<EntryDataBundle>)> {
        tree_view
            .view_parent_of(self.navigation().view_root)
            .map(|parent_idx| {
                (
                    parent_idx,
                    tree_view.sorted_entries(parent_idx, self.sorting, self.entry_check()),
                )
            })
    }

    pub fn exit_node(
        &mut self,
        entries: Option<(TreeIndex, Vec<EntryDataBundle>)>,
        tree_view: &TreeView<'_>,
    ) {
        match entries {
            Some((parent_idx, entries)) => {
                self.navigation_mut().exit_node(parent_idx, &entries);
                self.entries = entries;
                self.update_entry_annotations(tree_view);
                self.reset_message();
            }
            None => self.message = Some("Top level reached".into()),
        }
    }

    fn entries_for_enter_node(
        &self,
        tree_view: &TreeView<'_>,
    ) -> Option<(TreeIndex, Vec<EntryDataBundle>)> {
        self.navigation().selected.map(|previously_selected| {
            (
                previously_selected,
                tree_view.sorted_entries(previously_selected, self.sorting, self.entry_check()),
            )
        })
    }

    pub fn enter_node_with_traversal(&mut self, tree_view: &TreeView<'_>) {
        let new_entries = self.entries_for_enter_node(tree_view);
        self.enter_node(new_entries, tree_view)
    }

    pub fn enter_node(
        &mut self,
        entries_at_selected: Option<(TreeIndex, Vec<EntryDataBundle>)>,
        tree_view: &TreeView<'_>,
    ) {
        if let Some((previously_selected, new_entries)) = entries_at_selected {
            match self
                .navigation()
                .previously_selected_index(previously_selected, &new_entries)
            {
                Some(selected) => {
                    self.navigation_mut()
                        .enter_node(previously_selected, selected);
                    self.entries = new_entries;
                    self.update_entry_annotations(tree_view);
                    self.reset_message();
                }
                None => self.message = Some("Entry is a file or an empty directory".into()),
            }
        }
    }

    pub fn change_entry_selection(&mut self, direction: CursorDirection) {
        let next_index = self.navigation().next_index(direction, &self.entries);
        self.navigation_mut().select(next_index);
    }

    pub fn cycle_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_size();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
        self.update_entry_annotations(tree_view);
    }

    pub fn cycle_mtime_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_mtime();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
        self.update_entry_annotations(tree_view);
    }

    pub fn cycle_count_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_count();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
        self.update_entry_annotations(tree_view);
    }

    pub fn cycle_name_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_name();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
        self.update_entry_annotations(tree_view);
    }

    pub fn cycle_mtime_sort_mode(&mut self, tree_view: &TreeView<'_>) {
        if self.sorting.mtime_sort().is_some() {
            self.sorting.cycle_mtime_sort();
            self.entries = tree_view.sorted_entries(
                self.navigation().view_root,
                self.sorting,
                self.entry_check(),
            );
        } else {
            self.toggle_column(Column::MTime);
        }
    }

    pub fn toggle_count_column(&mut self) {
        self.toggle_column(Column::Count);
    }

    pub fn toggle_cleanup_candidates(&mut self, tree_view: &TreeView<'_>) {
        self.cleanup_candidates = self.cleanup_candidates.is_none().then(BTreeSet::new);
        self.update_entry_annotations(tree_view);
        self.reset_message();
    }

    pub fn toggle_gitignored_entries(&mut self, tree_view: &TreeView<'_>) {
        self.gitignored_entries = self.gitignored_entries.is_none().then(BTreeSet::new);
        self.update_entry_annotations(tree_view);
        self.reset_message();
    }

    fn toggle_column(&mut self, column: Column) {
        if self.show_columns.contains(&column) {
            self.show_columns.remove(&column);
        } else {
            self.show_columns.insert(column);
        }
    }

    pub fn toggle_glob_search(&mut self, window: &mut MainWindow) {
        self.focussed = match self.focussed {
            Main | Mark | Help => {
                window.glob_pane = Some(GlobPane::default());
                Glob
            }
            Glob => unreachable!("BUG: glob pane must catch the input leading here"),
        }
    }

    pub fn reset_message(&mut self) {
        if self.scan.is_some() {
            self.message = Some("-> scanning <-".into());
        } else {
            self.message = annotation_message(
                self.cleanup_candidates.as_ref().map_or(0, BTreeSet::len),
                self.gitignored_entries.as_ref().map_or(0, BTreeSet::len),
            );
        }
    }

    pub fn toggle_help_pane(&mut self, window: &mut MainWindow) {
        self.focussed = match self.focussed {
            Main | Mark | Glob => {
                window.help_pane = Some(HelpPane::with_locale_from_env());
                Help
            }
            Help => {
                window.help_pane = None;
                Main
            }
        }
    }
    pub fn cycle_focus(&mut self, window: &mut MainWindow) {
        if let Some(p) = window.mark_pane.as_mut() {
            p.set_focus(false)
        };
        self.focussed = match (
            self.focussed,
            &window.help_pane,
            &mut window.mark_pane,
            &mut window.glob_pane,
        ) {
            (Main, Some(_), _, _) => Help,
            (Help, _, Some(pane), _) => {
                pane.set_focus(true);
                Mark
            }
            (Help, _, _, Some(_)) => Glob,
            (Help, _, None, None) => Main,
            (Mark, _, _, Some(_)) => Glob,
            (Mark, _, _, _) => Main,
            (Main, None, None, None) => Main,
            (Main, None, Some(pane), _) => {
                pane.set_focus(true);
                Mark
            }
            (Main, None, None, Some(_)) => Glob,
            (Glob, _, _, _) => Main,
        };
    }

    pub fn dispatch_to_mark_pane<B>(
        &mut self,
        key: KeyEvent,
        window: &mut MainWindow,
        tree_view: &mut TreeView<'_>,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
        config: &Config,
    ) where
        B: Backend,
    {
        let res = window.mark_pane.take().and_then(|p| p.process_events(key));
        window.mark_pane = match res {
            Some((pane, mode)) => match mode {
                Some(MarkMode::Delete) => {
                    self.message = Some("Deleting items...".to_string());
                    let start = Instant::now();
                    let mut entries_deleted = 0;
                    let mut bytes_deleted = 0;
                    let mut errors = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_delete| {
                        window.mark_pane = Some(pane);
                        self.draw(window, tree_view, display, terminal, config).ok();
                        pane = window.mark_pane.take().expect("option to be filled");
                        match self.delete_entry(entry_to_delete, tree_view) {
                            Ok(stats) => {
                                entries_deleted += stats.entries;
                                bytes_deleted += stats.bytes;
                                self.message = Some(format!("Deleted {entries_deleted} items..."));
                                Ok(pane)
                            }
                            Err(stats) => {
                                entries_deleted += stats.entries;
                                bytes_deleted += stats.bytes;
                                errors += stats.errors;
                                Err((pane, stats.errors))
                            }
                        }
                    });
                    self.message = None;
                    self.notify_deletion_finished(
                        "Deletion",
                        DeletionStats {
                            entries: entries_deleted,
                            bytes: bytes_deleted,
                            elapsed: start.elapsed(),
                            errors,
                        },
                        display,
                        config,
                    );
                    res
                }
                #[cfg(feature = "trash-move")]
                Some(MarkMode::Trash) => {
                    self.message = Some("Trashing items...".to_string());
                    let start = Instant::now();
                    let mut entries_trashed = 0;
                    let mut bytes_trashed = 0;
                    let mut errors = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_trash| {
                        window.mark_pane = Some(pane);
                        self.draw(window, tree_view, display, terminal, config).ok();
                        pane = window.mark_pane.take().expect("option to be filled");
                        let entry_size = tree_view
                            .tree()
                            .node_weight(entry_to_trash)
                            .map_or(0, |entry| entry.size);
                        match self.trash_entry(entry_to_trash, tree_view) {
                            Ok(ed) => {
                                entries_trashed += ed;
                                bytes_trashed += entry_size;
                                self.message = Some(format!("Trashed {entries_trashed} items..."));
                                Ok(pane)
                            }
                            Err(c) => {
                                errors += c;
                                Err((pane, c))
                            }
                        }
                    });
                    self.message = None;
                    self.notify_deletion_finished(
                        "Trash",
                        DeletionStats {
                            entries: entries_trashed,
                            bytes: bytes_trashed,
                            elapsed: start.elapsed(),
                            errors,
                        },
                        display,
                        config,
                    );
                    res
                }
                None => Some(pane),
            },
            None => None,
        };
        if window.mark_pane.is_none() {
            self.focussed = Main;
        }
    }

    fn notify_deletion_finished(
        &self,
        action: &str,
        stats: DeletionStats,
        display: DisplayOptions,
        config: &Config,
    ) {
        let message = notification::deletion_finished(
            action,
            stats.entries,
            stats.bytes,
            stats.elapsed,
            stats.errors,
            display.byte_format,
        );
        if let Err(err) = notification::emit_if_unfocused(
            config.notifications.delete_finished,
            self.terminal_focus.is_focussed(),
            &message,
        ) {
            log::debug!("Could not emit terminal notification: {err}");
        }
    }

    fn delete_entry(
        &mut self,
        index: TreeIndex,
        tree_view: &mut TreeView<'_>,
    ) -> Result<EntryDeletionStats, EntryDeletionStats> {
        if !tree_view.exists(index) {
            return Ok(EntryDeletionStats::default());
        }
        let path_to_delete = tree_view.path_of(index);
        let bytes = tree_view
            .tree()
            .node_weight(index)
            .map_or(0, |entry| entry.size);
        let mut stats = delete_directory_recursively(path_to_delete);
        if stats.errors == 0 {
            stats.entries = self.delete_entries_in_traversal(index, tree_view);
            stats.bytes = bytes;
            Ok(stats)
        } else {
            Err(stats)
        }
    }

    #[cfg(feature = "trash-move")]
    pub fn trash_entry(
        &mut self,
        index: TreeIndex,
        tree_view: &mut TreeView<'_>,
    ) -> Result<usize, usize> {
        let mut entries_deleted = 0;
        if tree_view.exists(index) {
            let path_to_delete = tree_view.path_of(index);
            if trash::delete(path_to_delete).is_err() {
                return Err(1);
            }
            entries_deleted = self.delete_entries_in_traversal(index, tree_view);
        }
        Ok(entries_deleted)
    }

    pub fn delete_entries_in_traversal(
        &mut self,
        index: TreeIndex,
        tree_view: &mut TreeView<'_>,
    ) -> usize {
        let parent_idx = tree_view
            .fs_parent_of(index)
            .expect("us being unable to delete the root index");
        let entries_deleted =
            tree_view.remove_entries(index, true /* remove node at `index` */);

        if !tree_view.exists(self.navigation().view_root) {
            self.go_to_root(tree_view);
        } else {
            self.entries = tree_view.sorted_entries(
                self.navigation().view_root,
                self.sorting,
                self.entry_check(),
            );
        }
        self.update_entry_annotations(tree_view);

        if self
            .navigation()
            .selected
            .and_then(|selected| self.entries.iter().find(|e| e.index == selected))
            .is_none()
        {
            let idx = self.entries.first().map(|e| e.index);
            self.navigation_mut().select(idx);
        }
        tree_view.recompute_sizes_recursively(parent_idx);

        entries_deleted
    }

    pub fn go_to_root(&mut self, tree_view: &TreeView<'_>) {
        let root = self.navigation().tree_root;
        let entries = tree_view.sorted_entries(root, self.sorting, self.entry_check());
        self.navigation_mut().exit_node(root, &entries);
        self.entries = entries;
        self.update_entry_annotations(tree_view);
    }

    pub fn glob_root(&self) -> Option<TreeIndex> {
        self.glob_navigation.as_ref().map(|e| e.tree_root)
    }

    fn mark_entry_by_index(
        &mut self,
        index: TreeIndex,
        mode: MarkEntryMode,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
    ) {
        let is_dir = self
            .entries
            .iter()
            .find(|e| e.index == index)
            .unwrap()
            .is_dir;
        let should_toggle = match mode {
            MarkEntryMode::Toggle => true,
            MarkEntryMode::MarkForDeletion => false,
        };
        if let Some(pane) = window.mark_pane.take() {
            window.mark_pane = pane.toggle_index(index, tree_view, is_dir, should_toggle);
        } else {
            window.mark_pane =
                MarkPane::default().toggle_index(index, tree_view, is_dir, should_toggle)
        }
    }

    pub fn mark_entry(
        &mut self,
        cursor: CursorMode,
        mode: MarkEntryMode,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
    ) {
        if let Some(index) = self.navigation().selected {
            self.mark_entry_by_index(index, mode, window, tree_view);
        };
        if let CursorMode::Advance = cursor {
            self.change_entry_selection(CursorDirection::Down)
        }
    }

    pub fn mark_all_entries(
        &mut self,
        mode: MarkEntryMode,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
    ) {
        for index in self.entries.iter().map(|e| e.index).collect::<Vec<_>>() {
            self.mark_entry_by_index(index, mode, window, tree_view);
        }
    }

    pub fn mark_cleanup_candidates(&mut self, window: &mut MainWindow, tree_view: &TreeView<'_>) {
        match self.cleanup_candidates.clone() {
            Some(cleanup_candidates) => self.mark_annotation_candidates(
                cleanup_candidates,
                "No cleanup candidates in view",
                "Cleanup candidates are already marked",
                "cleanup candidates",
                window,
                tree_view,
            ),
            None => self.message = Some("Cleanup candidate detection is disabled".into()),
        }
    }

    pub fn mark_gitignored_entries(&mut self, window: &mut MainWindow, tree_view: &TreeView<'_>) {
        match self.gitignored_entries.clone() {
            Some(gitignored_entries) => self.mark_annotation_candidates(
                gitignored_entries,
                "No gitignored entries in view",
                "Gitignored entries are already marked",
                "gitignored entries",
                window,
                tree_view,
            ),
            None => self.message = Some("Gitignored entry detection is disabled".into()),
        }
    }

    fn mark_annotation_candidates(
        &mut self,
        annotation_candidates: BTreeSet<TreeIndex>,
        none_in_view_message: &str,
        already_marked_message: &str,
        marked_label: &str,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
    ) {
        let already_marked = window.mark_pane.as_ref().map(|pane| pane.marked());
        let candidates = self
            .entries
            .iter()
            .filter_map(|entry| {
                let is_candidate = annotation_candidates.contains(&entry.index);
                let is_marked = already_marked
                    .map(|marked| marked.contains_key(&entry.index))
                    .unwrap_or(false);
                (is_candidate && !is_marked).then_some(entry.index)
            })
            .collect::<Vec<_>>();

        for index in &candidates {
            self.mark_entry_by_index(*index, MarkEntryMode::MarkForDeletion, window, tree_view);
        }

        if candidates.is_empty() {
            self.message = Some(if annotation_candidates.is_empty() {
                none_in_view_message.into()
            } else {
                already_marked_message.into()
            });
        } else {
            self.message = Some(format!("Marked {} {marked_label}", candidates.len()));
        }
    }

    pub fn update_entry_annotations(&mut self, tree_view: &TreeView<'_>) {
        if self.glob_navigation.is_some() {
            if self.cleanup_candidates.is_some() {
                self.cleanup_candidates = Some(Default::default());
            }
            if self.gitignored_entries.is_some() {
                self.gitignored_entries = Some(Default::default());
            }
        } else {
            if self.cleanup_candidates.is_some() {
                self.cleanup_candidates = Some(super::cleanup::cleanup_candidates(&self.entries));
            }
            if self.gitignored_entries.is_some() {
                self.gitignored_entries = Some(super::gitignore::gitignored_entries(
                    tree_view,
                    self.navigation().view_root,
                    &self.entries,
                ));
            }
        }
    }
}

fn annotation_message(cleanup_count: usize, gitignored_count: usize) -> Option<String> {
    match (cleanup_count, gitignored_count) {
        (0, 0) => None,
        (cleanup, 0) => {
            let label = if cleanup == 1 {
                "cleanup candidate"
            } else {
                "cleanup candidates"
            };
            Some(format!("{cleanup} {label} (X)"))
        }
        (0, gitignored) => {
            let label = if gitignored == 1 {
                "gitignored entry"
            } else {
                "gitignored entries"
            };
            Some(format!("{gitignored} {label} (I)"))
        }
        (cleanup, gitignored) => Some(format!("{cleanup} cleanup, {gitignored} gitignored (X|I)")),
    }
}

fn io_err_to_usize(err: io::Error) -> usize {
    if err.kind() == io::ErrorKind::NotFound {
        0
    } else {
        1
    }
}

// TODO: could use jwalk for this
// see https://github.com/Byron/dua-cli/issues/43
fn delete_directory_recursively(path: PathBuf) -> EntryDeletionStats {
    let mut files_or_dirs = vec![path];
    let mut dirs = Vec::new();
    let mut stats = EntryDeletionStats::default();
    while let Some(path) = files_or_dirs.pop() {
        let assume_symlink_to_try_deletion = true;
        let metadata = path.symlink_metadata();
        let bytes = metadata.as_ref().map_or(0, |metadata| metadata.len()) as u128;
        let is_symlink = metadata
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(assume_symlink_to_try_deletion);
        if is_symlink {
            // do not follow symlinks
            record_removal(fs::remove_file(&path), bytes, &mut stats);
            continue;
        }
        match fs::read_dir(&path) {
            Ok(iterator) => {
                dirs.push((path, bytes));
                for entry in iterator {
                    match entry.map_err(io_err_to_usize) {
                        Ok(entry) => files_or_dirs.push(entry.path()),
                        Err(c) => stats.errors += c,
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotADirectory => {
                // try again with file deletion instead.
                record_removal(fs::remove_file(path), bytes, &mut stats);
                continue;
            }
            Err(_) => {
                stats.errors += 1;
                continue;
            }
        };
    }

    for (dir, bytes) in dirs.into_iter().rev() {
        record_removal(
            fs::remove_dir(&dir).or_else(|_| fs::remove_file(dir)),
            bytes,
            &mut stats,
        );
    }

    stats
}

fn record_removal(result: io::Result<()>, bytes: u128, stats: &mut EntryDeletionStats) {
    match result {
        Ok(()) => {
            stats.entries += 1;
            stats.bytes += bytes;
        }
        Err(err) => stats.errors += io_err_to_usize(err),
    }
}

#[cfg(test)]
mod deletion_notification_tests {
    use super::*;

    #[test]
    fn retains_partial_success_statistics_alongside_errors() {
        let mut stats = EntryDeletionStats::default();
        record_removal(Ok(()), 42, &mut stats);
        record_removal(
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
            100,
            &mut stats,
        );

        assert_eq!(stats.entries, 1);
        assert_eq!(stats.bytes, 42);
        assert_eq!(stats.errors, 1);
    }
}
