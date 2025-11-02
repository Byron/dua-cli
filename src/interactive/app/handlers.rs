use crate::interactive::{
    DisplayOptions, EntryDataBundle,
    app::tree_view::TreeView,
    widgets::{Column, GlobPane, HelpPane, MainWindow, MarkMode, MarkPane},
};
use crosstermion::input::Key;
use dua::traverse::TreeIndex;
use std::{fs, io, path::PathBuf};
use tui::{Terminal, backend::Backend};

use super::state::{AppState, FocussedPane::*};

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
    pub fn open_that(&self, tree_view: &TreeView<'_>) {
        if let Some(idx) = self.navigation().selected {
            open::that(tree_view.path_of(idx)).ok();
        }
    }

    pub fn exit_node_with_traversal(&mut self, tree_view: &TreeView<'_>) {
        let entries = self.entries_for_exit_node(tree_view);
        self.exit_node(entries);
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

    pub fn exit_node(&mut self, entries: Option<(TreeIndex, Vec<EntryDataBundle>)>) {
        match entries {
            Some((parent_idx, entries)) => {
                self.navigation_mut().exit_node(parent_idx, &entries);
                self.entries = entries;
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
        self.enter_node(new_entries)
    }

    pub fn enter_node(&mut self, entries_at_selected: Option<(TreeIndex, Vec<EntryDataBundle>)>) {
        if let Some((previously_selected, new_entries)) = entries_at_selected {
            match self
                .navigation()
                .previously_selected_index(previously_selected, &new_entries)
            {
                Some(selected) => {
                    self.navigation_mut()
                        .enter_node(previously_selected, selected);
                    self.entries = new_entries;
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
    }

    pub fn cycle_mtime_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_mtime();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
    }

    pub fn cycle_count_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_count();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
    }

    pub fn cycle_name_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_name();
        self.entries = tree_view.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
    }

    pub fn toggle_mtime_column(&mut self) {
        self.toggle_column(Column::MTime);
    }

    pub fn toggle_count_column(&mut self) {
        self.toggle_column(Column::Count);
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
            self.message = None;
        }
    }

    pub fn toggle_help_pane(&mut self, window: &mut MainWindow) {
        self.focussed = match self.focussed {
            Main | Mark | Glob => {
                window.help_pane = Some(HelpPane::default());
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
        key: Key,
        window: &mut MainWindow,
        tree_view: &mut TreeView<'_>,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) where
        B: Backend,
    {
        let res = window.mark_pane.take().and_then(|p| p.process_events(key));
        window.mark_pane = match res {
            Some((pane, mode)) => match mode {
                Some(MarkMode::Delete) => {
                    self.message = Some("Deleting items...".to_string());
                    let mut entries_deleted = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_delete| {
                        window.mark_pane = Some(pane);
                        self.draw(window, tree_view, display, terminal).ok();
                        pane = window.mark_pane.take().expect("option to be filled");
                        match self.delete_entry(entry_to_delete, tree_view) {
                            Ok(ed) => {
                                entries_deleted += ed;
                                self.message = Some(format!("Deleted {entries_deleted} items..."));
                                Ok(pane)
                            }
                            Err(c) => Err((pane, c)),
                        }
                    });
                    self.message = None;
                    res
                }
                #[cfg(feature = "trash-move")]
                Some(MarkMode::Trash) => {
                    self.message = Some("Trashing items...".to_string());
                    let mut entries_trashed = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_trash| {
                        window.mark_pane = Some(pane);
                        self.draw(window, tree_view, display, terminal).ok();
                        pane = window.mark_pane.take().expect("option to be filled");
                        match self.trash_entry(entry_to_trash, tree_view) {
                            Ok(ed) => {
                                entries_trashed += ed;
                                self.message = Some(format!("Trashed {entries_trashed} items..."));
                                Ok(pane)
                            }
                            Err(c) => Err((pane, c)),
                        }
                    });
                    self.message = None;
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

    pub fn delete_entry(
        &mut self,
        index: TreeIndex,
        tree_view: &mut TreeView<'_>,
    ) -> Result<usize, usize> {
        let mut entries_deleted = 0;
        if tree_view.exists(index) {
            let path_to_delete = tree_view.path_of(index);
            delete_directory_recursively(path_to_delete)?;
            entries_deleted = self.delete_entries_in_traversal(index, tree_view);
        }
        Ok(entries_deleted)
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
}

fn into_error_count(res: Result<(), io::Error>) -> usize {
    match res.map_err(io_err_to_usize) {
        Ok(_) => 0,
        Err(c) => c,
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
fn delete_directory_recursively(path: PathBuf) -> Result<(), usize> {
    let mut files_or_dirs = vec![path];
    let mut dirs = Vec::new();
    let mut num_errors = 0;
    while let Some(path) = files_or_dirs.pop() {
        let assume_symlink_to_try_deletion = true;
        let is_symlink = path
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(assume_symlink_to_try_deletion);
        if is_symlink {
            // do not follow symlinks
            num_errors += into_error_count(fs::remove_file(&path));
            continue;
        }
        match fs::read_dir(&path) {
            Ok(iterator) => {
                dirs.push(path);
                for entry in iterator {
                    match entry.map_err(io_err_to_usize) {
                        Ok(entry) => files_or_dirs.push(entry.path()),
                        Err(c) => num_errors += c,
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotADirectory => {
                // try again with file deletion instead.
                num_errors += into_error_count(fs::remove_file(path));
                continue;
            }
            Err(_) => {
                num_errors += 1;
                continue;
            }
        };
    }

    for dir in dirs.into_iter().rev() {
        num_errors += into_error_count(fs::remove_dir(&dir).or_else(|_| fs::remove_file(dir)));
    }

    if num_errors == 0 {
        Ok(())
    } else {
        Err(num_errors)
    }
}
