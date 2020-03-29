use crate::interactive::{
    app::{FocussedPane::*, TerminalApp},
    path_of, sorted_entries,
    widgets::MarkMode,
    widgets::{HelpPane, MarkPane},
    AppState, EntryDataBundle,
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
}

impl TerminalApp {
    pub fn cycle_focus(&mut self) {
        if let Some(p) = self.window.mark_pane.as_mut() {
            p.set_focus(false)
        };
        self.state.focussed = match (
            self.state.focussed,
            &self.window.help_pane,
            &mut self.window.mark_pane,
        ) {
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

    pub fn toggle_help_pane(&mut self) {
        self.state.focussed = match self.state.focussed {
            Main | Mark => {
                self.window.help_pane = Some(HelpPane::default());
                Help
            }
            Help => {
                self.window.help_pane = None;
                Main
            }
        }
    }

    pub fn reset_message(&mut self) {
        self.state.message = None;
    }

    pub fn open_that(&self) {
        self.state.open_that(&self.traversal)
    }

    pub fn exit_node(&mut self) {
        let entries = self.state.entries_for_exit_node(&self.traversal);
        self.state.exit_node(entries);
    }

    pub fn enter_node(&mut self) {
        let new_entries = self.state.entries_for_enter_node(&self.traversal);
        self.state.enter_node(new_entries)
    }

    pub fn change_entry_selection(&mut self, direction: CursorDirection) {
        self.state.change_entry_selection(direction)
    }

    pub fn cycle_sorting(&mut self) {
        self.state.cycle_sorting(&self.traversal)
    }

    pub fn mark_entry(&mut self, advance_cursor: bool) {
        if let Some(index) = self.state.selected {
            let is_dir = self
                .state
                .entries
                .iter()
                .find(|e| e.index == index)
                .unwrap()
                .is_dir;
            if let Some(pane) = self.window.mark_pane.take() {
                self.window.mark_pane = pane.toggle_index(index, &self.traversal.tree, is_dir);
            } else {
                self.window.mark_pane =
                    MarkPane::default().toggle_index(index, &self.traversal.tree, is_dir)
            }
        };
        if advance_cursor {
            self.change_entry_selection(CursorDirection::Down)
        }
    }

    fn set_root(&mut self, root: TreeIndex) {
        self.state.root = root;
        self.state.entries = sorted_entries(&self.traversal.tree, root, self.state.sorting);
    }

    pub fn delete_entry(&mut self, index: TreeIndex) -> Result<usize, usize> {
        let mut entries_deleted = 0;
        if let Some(_entry) = self.traversal.tree.node_weight(index) {
            let path_to_delete = path_of(&self.traversal.tree, index);
            delete_directory_recursively(path_to_delete)?;
            let parent_idx = self
                .traversal
                .tree
                .neighbors_directed(index, Direction::Incoming)
                .next()
                .expect("us being unable to delete the root index");
            let mut bfs = Bfs::new(&self.traversal.tree, index);
            while let Some(nx) = bfs.next(&self.traversal.tree) {
                self.traversal.tree.remove_node(nx);
                self.traversal.entries_traversed -= 1;
                entries_deleted += 1;
            }
            self.state.entries =
                sorted_entries(&self.traversal.tree, self.state.root, self.state.sorting);
            if self.traversal.tree.node_weight(self.state.root).is_none() {
                self.set_root(self.traversal.root_index);
            }
            if self
                .state
                .selected
                .and_then(|selected| self.state.entries.iter().find(|e| e.index == selected))
                .is_none()
            {
                self.state.selected = self.state.entries.get(0).map(|e| e.index);
            }
            self.recompute_sizes_recursively(parent_idx);
        }
        Ok(entries_deleted)
    }

    fn recompute_sizes_recursively(&mut self, mut index: TreeIndex) {
        loop {
            self.traversal
                .tree
                .node_weight_mut(index)
                .expect("valid index")
                .size = self
                .traversal
                .tree
                .neighbors_directed(index, Direction::Outgoing)
                .filter_map(|idx| self.traversal.tree.node_weight(idx).map(|w| w.size))
                .sum();
            match self
                .traversal
                .tree
                .neighbors_directed(index, Direction::Incoming)
                .next()
            {
                None => break,
                Some(parent) => index = parent,
            }
        }
        self.traversal.total_bytes = self
            .traversal
            .tree
            .node_weight(self.traversal.root_index)
            .map(|w| w.size);
    }

    pub fn dispatch_to_mark_pane<B>(&mut self, key: Key, terminal: &mut Terminal<B>)
    where
        B: Backend,
    {
        let res = self.window.mark_pane.take().and_then(|p| p.key(key));
        self.window.mark_pane = match res {
            Some((pane, mode)) => match mode {
                Some(MarkMode::Delete) => {
                    self.state.message = Some("Deleting entries...".to_string());
                    let mut entries_deleted = 0;
                    let res = pane.iterate_deletable_items(|mut pane, entry_to_delete| {
                        self.window.mark_pane = Some(pane);
                        self.draw(terminal).ok();
                        pane = self.window.mark_pane.take().expect("option to be filled");
                        match self.delete_entry(entry_to_delete) {
                            Ok(ed) => {
                                entries_deleted += ed;
                                self.state.message =
                                    Some(format!("Deleted {} entries...", entries_deleted));
                                Ok(pane)
                            }
                            Err(c) => Err((pane, c)),
                        }
                    });
                    self.state.message = None;
                    res
                }
                None => Some(pane),
            },
            None => None,
        };
        if self.window.mark_pane.is_none() {
            self.state.focussed = Main;
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
