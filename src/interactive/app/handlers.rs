use crate::interactive::{
    app::{EntryMark, FocussedPane, TerminalApp},
    sorted_entries,
    widgets::HelpPane,
};
use dua::path_of;
use itertools::Itertools;
use petgraph::Direction;

pub enum CursorDirection {
    PageDown,
    Down,
    Up,
    PageUp,
}

impl TerminalApp {
    pub fn cycle_focus(&mut self) {
        use FocussedPane::*;
        self.state.focussed = match (self.state.focussed, &self.window.help_pane) {
            (Main, Some(_)) => Help,
            (Help, _) => Main,
            _ => Main,
        };
    }

    pub fn toggle_help_pane(&mut self) {
        use FocussedPane::*;
        self.state.focussed = match self.state.focussed {
            Main => {
                self.window.help_pane = Some(HelpPane::default());
                Help
            }
            Help => {
                self.window.help_pane = None;
                Main
            }
        }
    }

    pub fn update_message(&mut self) {
        self.state.message = None;
    }

    pub fn open_that(&mut self) {
        match self.state.selected {
            Some(ref idx) => {
                open::that(path_of(&self.traversal.tree, *idx)).ok();
            }
            None => {}
        }
    }

    pub fn exit_node(&mut self) {
        match self
            .traversal
            .tree
            .neighbors_directed(self.state.root, Direction::Incoming)
            .next()
        {
            Some(parent_idx) => {
                self.state.root = parent_idx;
                self.state.entries =
                    sorted_entries(&self.traversal.tree, parent_idx, self.state.sorting);
                self.state.selected = self.state.entries.get(0).map(|b| b.index);
            }
            None => self.state.message = Some("Top level reached".into()),
        }
    }

    pub fn enter_node(&mut self) {
        if let Some(new_root) = self.state.selected {
            self.state.entries = sorted_entries(&self.traversal.tree, new_root, self.state.sorting);
            match self.state.entries.get(0) {
                Some(b) => {
                    self.state.root = new_root;
                    self.state.selected = Some(b.index);
                }
                None => self.state.message = Some("Entry is a file or an empty directory".into()),
            }
        }
    }

    pub fn scroll_help(&mut self, direction: CursorDirection) {
        use CursorDirection::*;
        if let Some(ref mut pane) = self.window.help_pane {
            pane.scroll = match direction {
                Down => pane.scroll.saturating_add(1),
                Up => pane.scroll.saturating_sub(1),
                PageDown => pane.scroll.saturating_add(10),
                PageUp => pane.scroll.saturating_sub(10),
            };
        }
    }

    pub fn change_entry_selection(&mut self, direction: CursorDirection) {
        let entries = sorted_entries(&self.traversal.tree, self.state.root, self.state.sorting);
        let next_selected_pos = match self.state.selected {
            Some(ref selected) => entries
                .iter()
                .find_position(|b| b.index == *selected)
                .map(|(idx, _)| match direction {
                    CursorDirection::PageDown => idx.saturating_add(10),
                    CursorDirection::Down => idx.saturating_add(1),
                    CursorDirection::Up => idx.saturating_sub(1),
                    CursorDirection::PageUp => idx.saturating_sub(10),
                })
                .unwrap_or(0),
            None => 0,
        };
        self.state.selected = entries
            .get(next_selected_pos)
            .or(entries.last())
            .map(|b| b.index)
            .or(self.state.selected)
    }

    pub fn cycle_sorting(&mut self) {
        self.state.sorting.toggle_size();
        self.state.entries =
            sorted_entries(&self.traversal.tree, self.state.root, self.state.sorting);
    }

    pub fn mark_entry(&mut self, advance_cursor: bool) {
        if let Some(index) = self.state.selected {
            // TODO: consider using the Entry::Occupied/Vacant API to remove things
            if self.state.marked.get(&index).is_some() {
                self.state.marked.remove(&index);
            } else {
                self.state.marked.insert(index, EntryMark {});
            }
            if advance_cursor {
                self.change_entry_selection(CursorDirection::Down)
            }
        }
    }
}
