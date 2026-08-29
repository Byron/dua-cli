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
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};
use tui::{Terminal, backend::Backend};

use super::{
    notification,
    state::{
        AppState,
        FocussedPane::{Glob, Help, Main, Mark},
    },
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
        use CursorDirection::{Down, PageDown, PageUp, ToBottom, ToTop, Up};
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
            if self.read_only && !path.exists() {
                self.message = Some(format!("Snapshot path is unavailable: {}", path.display()));
                return;
            }
            if let Err(err) = open::that(&path) {
                self.message = Some(format!("Failed to open {}: {err}", path.display()));
            }
        }
    }

    pub fn exit_node_with_traversal(&mut self, tree_view: &TreeView<'_>, scan_parent_key: &str) {
        let entries = self.entries_for_exit_node(tree_view);
        self.exit_node(entries, tree_view, scan_parent_key);
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
        scan_parent_key: &str,
    ) {
        match entries {
            Some((parent_idx, entries)) => {
                self.navigation_mut().exit_node(parent_idx, &entries);
                self.entries = entries;
                self.update_entry_annotations(tree_view);
                self.reset_message();
            }
            None => {
                self.message = Some(if self.can_scan_parent(tree_view) {
                    format!(
                        "Top level reached. Press {scan_parent_key} to scan the parent directory"
                    )
                } else {
                    "Top level reached".into()
                });
            }
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
        self.enter_node(new_entries, tree_view);
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
        if self.read_only {
            self.message = Some("Gitignored entry detection is unavailable for snapshots".into());
            return;
        }
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
                window.glob = Some(GlobPane::default());
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
                window.help = Some(HelpPane::with_locale_from_env());
                Help
            }
            Help => {
                window.help = None;
                Main
            }
        }
    }
    pub fn cycle_focus(&mut self, window: &mut MainWindow) {
        if let Some(p) = window.mark.as_mut() {
            p.set_focus(false);
        }
        self.focussed = match (
            self.focussed,
            &window.help,
            &mut window.mark,
            &mut window.glob,
        ) {
            (Main, Some(_), _, _) => Help,
            (Help, _, Some(pane), _) | (Main, None, Some(pane), _) => {
                pane.set_focus(true);
                Mark
            }
            (Help | Mark, _, _, Some(_)) | (Main, None, None, Some(_)) => Glob,
            (Help, _, None, None) | (Mark | Glob, _, _, _) | (Main, None, None, None) => Main,
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
        let res = window
            .mark
            .take()
            .and_then(|p| p.process_events(key, &config.keys));
        window.mark = match res {
            Some((pane, Some(_))) if self.read_only => {
                self.message = Some("Snapshots are read-only".into());
                Some(pane)
            }
            Some((pane, mode)) => match mode {
                Some(MarkMode::Delete) => {
                    self.message = Some("Deleting items...".to_string());
                    let start = Instant::now();
                    let mut entries_deleted = 0;
                    let mut bytes_deleted = 0;
                    let mut errors = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_delete| {
                        window.mark = Some(pane);
                        self.draw(window, tree_view, display, terminal, config).ok();
                        pane = window.mark.take().expect("option to be filled");
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
                        window.mark = Some(pane);
                        self.draw(window, tree_view, display, terminal, config).ok();
                        pane = window.mark.take().expect("option to be filled");
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
        if window.mark.is_none() {
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
        let mut stats = delete_directory_recursively(path_to_delete, self.walk_options.threads);
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

        if tree_view.exists(self.navigation().view_root) {
            self.entries = tree_view.sorted_entries(
                self.navigation().view_root,
                self.sorting,
                self.entry_check(),
            );
        } else {
            self.go_to_root(tree_view);
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
        if let Some(pane) = window.mark.take() {
            window.mark = pane.toggle_index(index, tree_view, is_dir, should_toggle);
        } else {
            window.mark = MarkPane::default().toggle_index(index, tree_view, is_dir, should_toggle);
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
        }
        if let CursorMode::Advance = cursor {
            self.change_entry_selection(CursorDirection::Down);
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
        let already_marked = window.mark.as_ref().map(MarkPane::marked);
        let candidates = self
            .entries
            .iter()
            .filter_map(|entry| {
                let is_candidate = annotation_candidates.contains(&entry.index);
                let is_marked =
                    already_marked.is_some_and(|marked| marked.contains_key(&entry.index));
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
                self.cleanup_candidates = Some(BTreeSet::default());
            }
            if self.gitignored_entries.is_some() {
                self.gitignored_entries = Some(BTreeSet::default());
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
            Some(format!("{cleanup} {label}"))
        }
        (0, gitignored) => {
            let label = if gitignored == 1 {
                "gitignored entry"
            } else {
                "gitignored entries"
            };
            Some(format!("{gitignored} {label}"))
        }
        (cleanup, gitignored) => Some(format!("{cleanup} cleanup, {gitignored} gitignored")),
    }
}

fn io_err_to_usize(err: io::Error) -> usize {
    usize::from(err.kind() != io::ErrorKind::NotFound)
}

/// Remove `path` and everything beneath it, returning deletion statistics.
///
/// Uses the work-stealing walker for a parallel traversal that does **not** follow symlinks.
/// Files and symlinks are removed in parallel;
/// directories are collected and removed deepest-first so each `remove_dir`
/// sees an empty directory.
fn delete_directory_recursively(path: PathBuf, threads: usize) -> EntryDeletionStats {
    let mut stats = EntryDeletionStats::default();
    let mut dirs: Vec<(PathBuf, u128, usize)> = Vec::new();
    let mut files: Vec<(PathBuf, u128)> = Vec::new();

    for entry in dua_core::walk(
        &path,
        threads,
        dua_core::Order::Completion,
        dua_core::Options::default(),
        |_| true,
    ) {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                let bytes =
                    u128::from(entry.metadata.as_ref().map_or(0, |metadata| metadata.len()));
                if entry.file_type.is_dir() {
                    // Real directory (symlinks to dirs report is_symlink, not
                    // is_dir, when follow_links is false): remove after children.
                    dirs.push((entry_path, bytes, entry.depth));
                } else {
                    // Regular file or symlink — remove without following.
                    files.push((entry_path, bytes));
                }
            }
            Err(_) => stats.errors += 1,
        }
    }

    let next_file = AtomicUsize::new(0);
    let file_stats = thread::scope(|scope| {
        let handles = (0..threads.max(1).min(files.len()))
            .map(|_| {
                scope.spawn(|| {
                    let mut total = EntryDeletionStats::default();
                    while let Some((path, bytes)) =
                        files.get(next_file.fetch_add(1, Ordering::Relaxed))
                    {
                        record_removal(fs::remove_file(path), *bytes, &mut total);
                    }
                    total
                })
            })
            .collect::<Vec<_>>();

        handles
            .into_iter()
            .map(|handle| handle.join().expect("deletion worker does not panic"))
            .fold(EntryDeletionStats::default(), |mut total, stats| {
                total.entries += stats.entries;
                total.bytes += stats.bytes;
                total.errors += stats.errors;
                total
            })
    });
    stats.entries += file_stats.entries;
    stats.bytes += file_stats.bytes;
    stats.errors += file_stats.errors;

    // Remove directories deepest-first so parents are empty when removed.
    dirs.sort_by(|a, b| a.2.cmp(&b.2).reverse());
    for (dir, bytes, _) in dirs {
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

#[cfg(test)]
mod delete_directory_recursively_tests {
    use super::*;

    #[test]
    fn removes_a_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();

        let stats = delete_directory_recursively(file.clone(), 1);

        assert_eq!(stats.errors, 0);
        assert_eq!(stats.entries, 1);
        assert!(!file.exists());
    }

    #[test]
    fn removes_a_nested_tree() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("top.txt"), b"12345").unwrap();
        fs::write(nested.join("deep.txt"), b"abc").unwrap();

        let stats = delete_directory_recursively(root.clone(), 1);

        assert_eq!(stats.errors, 0);
        // top.txt + deep.txt + nested dir + root dir
        assert_eq!(stats.entries, 4);
        assert!(!root.exists());
    }

    #[cfg(unix)]
    #[test]
    fn removes_symlink_without_following_it() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep.txt"), b"keep").unwrap();

        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let stats = delete_directory_recursively(link.clone(), 1);

        assert_eq!(stats.errors, 0);
        assert!(!link.exists(), "the symlink itself should be gone");
        assert!(
            target.join("keep.txt").exists(),
            "the symlink target must not be deleted"
        );
    }

    #[test]
    fn reports_an_error_for_a_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");

        let stats = delete_directory_recursively(missing, 1);

        assert_eq!(stats.entries, 0);
        assert!(stats.errors > 0);
    }
}
