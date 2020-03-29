use crate::interactive::widgets::MainWindow;
use crate::interactive::{
    app::FocussedPane::*,
    path_of, sorted_entries,
    widgets::MarkMode,
    widgets::{HelpPane, MarkPane},
    AppState, DisplayOptions, EntryDataBundle,
};
use dua::traverse::{Traversal, TreeIndex};
use itertools::Itertools;
use petgraph::{visit::Bfs, Direction};
use std::{fs, io, path::PathBuf};
use termion::event::Key;
use tui::backend::Backend;
use tui_react::Terminal;

pub enum CursorDirection {
    PageDown,
    Down,
    Up,
    PageUp,
}

impl CursorDirection {
    pub fn move_cursor(&self, n: usize) -> usize {
        use CursorDirection::*;
        match self {
            Down => n.saturating_add(1),
            Up => n.saturating_sub(1),
            PageDown => n.saturating_add(10),
            PageUp => n.saturating_sub(10),
        }
    }
}

impl AppState {
    pub fn open_that(&self, traversal: &Traversal) {
        if let Some(idx) = self.selected {
            open::that(path_of(&traversal.tree, idx)).ok();
        }
    }

    pub fn exit_node_with_traversal(&mut self, traversal: &Traversal) {
        let entries = self.entries_for_exit_node(traversal);
        self.exit_node(entries);
    }

    fn entries_for_exit_node(
        &self,
        traversal: &Traversal,
    ) -> Option<(TreeIndex, Vec<EntryDataBundle>)> {
        traversal
            .tree
            .neighbors_directed(self.root, Direction::Incoming)
            .next()
            .map(|parent_idx| {
                (
                    parent_idx,
                    sorted_entries(&traversal.tree, parent_idx, self.sorting),
                )
            })
    }

    pub fn exit_node(&mut self, entries: Option<(TreeIndex, Vec<EntryDataBundle>)>) {
        match entries {
            Some((parent_idx, entries)) => {
                self.root = parent_idx;
                self.entries = entries;
                self.selected = self
                    .bookmarks
                    .get(&parent_idx)
                    .copied()
                    .or_else(|| self.entries.get(0).map(|b| b.index));
            }
            None => self.message = Some("Top level reached".into()),
        }
    }

    fn entries_for_enter_node(
        &self,
        traversal: &Traversal,
    ) -> Option<(TreeIndex, Vec<EntryDataBundle>)> {
        self.selected.map(|previously_selected| {
            (
                previously_selected,
                sorted_entries(&traversal.tree, previously_selected, self.sorting),
            )
        })
    }

    pub fn enter_node_with_traversal(&mut self, traversal: &Traversal) {
        let new_entries = self.entries_for_enter_node(traversal);
        self.enter_node(new_entries)
    }

    pub fn enter_node(&mut self, entries_at_selected: Option<(TreeIndex, Vec<EntryDataBundle>)>) {
        if let Some((previously_selected, new_entries)) = entries_at_selected {
            match new_entries.get(
                self.bookmarks
                    .get(&previously_selected)
                    .and_then(|selected| {
                        new_entries
                            .iter()
                            .find_position(|b| b.index == *selected)
                            .map(|(pos, _)| pos)
                    })
                    .unwrap_or(0),
            ) {
                Some(b) => {
                    self.bookmarks.insert(self.root, previously_selected);
                    self.root = previously_selected;
                    self.selected = Some(b.index);
                    self.entries = new_entries;
                }
                None => self.message = Some("Entry is a file or an empty directory".into()),
            }
        }
    }

    pub fn change_entry_selection(&mut self, direction: CursorDirection) {
        let entries = &self.entries;
        let next_selected_pos = match self.selected {
            Some(ref selected) => entries
                .iter()
                .find_position(|b| b.index == *selected)
                .map(|(idx, _)| direction.move_cursor(idx))
                .unwrap_or(0),
            None => 0,
        };
        self.selected = entries
            .get(next_selected_pos)
            .or_else(|| entries.last())
            .map(|b| b.index)
            .or(self.selected);
        if let Some(selected) = self.selected {
            self.bookmarks.insert(self.root, selected);
        }
    }

    pub fn cycle_sorting(&mut self, traversal: &Traversal) {
        self.sorting.toggle_size();
        self.entries = sorted_entries(&traversal.tree, self.root, self.sorting);
    }

    pub fn reset_message(&mut self) {
        if self.is_scanning {
            self.message = Some("-> scanning <-".into());
        } else {
            self.message = None;
        }
    }

    pub fn toggle_help_pane(&mut self, window: &mut MainWindow) {
        self.focussed = match self.focussed {
            Main | Mark => {
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
        self.focussed = match (self.focussed, &window.help_pane, &mut window.mark_pane) {
            (Main, Some(_), _) => Help,
            (Help, _, Some(ref mut pane)) => {
                pane.set_focus(true);
                Mark
            }
            (Help, _, None) => Main,
            (Mark, _, _) => Main,
            (Main, None, None) => Main,
            (Main, None, Some(ref mut pane)) => {
                pane.set_focus(true);
                Mark
            }
        };
    }

    pub fn dispatch_to_mark_pane<B>(
        &mut self,
        key: Key,
        window: &mut MainWindow,
        traversal: &mut Traversal,
        display: DisplayOptions,
        terminal: &mut Terminal<B>,
    ) where
        B: Backend,
    {
        let res = window.mark_pane.take().and_then(|p| p.key(key));
        window.mark_pane = match res {
            Some((pane, mode)) => match mode {
                Some(MarkMode::Delete) => {
                    self.message = Some("Deleting entries...".to_string());
                    let mut entries_deleted = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_delete| {
                        window.mark_pane = Some(pane);
                        self.draw(window, traversal, display, terminal).ok();
                        pane = window.mark_pane.take().expect("option to be filled");
                        match self.delete_entry(entry_to_delete, traversal) {
                            Ok(ed) => {
                                entries_deleted += ed;
                                self.message =
                                    Some(format!("Deleted {} entries...", entries_deleted));
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
        traversal: &mut Traversal,
    ) -> Result<usize, usize> {
        let mut entries_deleted = 0;
        if let Some(_entry) = traversal.tree.node_weight(index) {
            let path_to_delete = path_of(&traversal.tree, index);
            delete_directory_recursively(path_to_delete)?;
            let parent_idx = traversal
                .tree
                .neighbors_directed(index, Direction::Incoming)
                .next()
                .expect("us being unable to delete the root index");
            let mut bfs = Bfs::new(&traversal.tree, index);
            while let Some(nx) = bfs.next(&traversal.tree) {
                traversal.tree.remove_node(nx);
                traversal.entries_traversed -= 1;
                entries_deleted += 1;
            }
            self.entries = sorted_entries(&traversal.tree, self.root, self.sorting);
            if traversal.tree.node_weight(self.root).is_none() {
                self.set_root(traversal.root_index, traversal);
            }
            if self
                .selected
                .and_then(|selected| self.entries.iter().find(|e| e.index == selected))
                .is_none()
            {
                self.selected = self.entries.get(0).map(|e| e.index);
            }
            self.recompute_sizes_recursively(parent_idx, traversal);
        }
        Ok(entries_deleted)
    }

    fn set_root(&mut self, root: TreeIndex, traversal: &Traversal) {
        self.root = root;
        self.entries = sorted_entries(&traversal.tree, root, self.sorting);
    }

    fn recompute_sizes_recursively(&mut self, mut index: TreeIndex, traversal: &mut Traversal) {
        loop {
            traversal
                .tree
                .node_weight_mut(index)
                .expect("valid index")
                .size = traversal
                .tree
                .neighbors_directed(index, Direction::Outgoing)
                .filter_map(|idx| traversal.tree.node_weight(idx).map(|w| w.size))
                .sum();
            match traversal
                .tree
                .neighbors_directed(index, Direction::Incoming)
                .next()
            {
                None => break,
                Some(parent) => index = parent,
            }
        }
        traversal.total_bytes = traversal
            .tree
            .node_weight(traversal.root_index)
            .map(|w| w.size);
    }

    pub fn mark_entry(
        &mut self,
        advance_cursor: bool,
        window: &mut MainWindow,
        traversal: &Traversal,
    ) {
        if let Some(index) = self.selected {
            let is_dir = self
                .entries
                .iter()
                .find(|e| e.index == index)
                .unwrap()
                .is_dir;
            if let Some(pane) = window.mark_pane.take() {
                window.mark_pane = pane.toggle_index(index, &traversal.tree, is_dir);
            } else {
                window.mark_pane = MarkPane::default().toggle_index(index, &traversal.tree, is_dir)
            }
        };
        if advance_cursor {
            self.change_entry_selection(CursorDirection::Down)
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
            Err(ref e) if e.kind() == io::ErrorKind::Other => {
                // assume file, save IOps
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
