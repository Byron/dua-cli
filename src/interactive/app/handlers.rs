use crate::interactive::widgets::MarkMode;
use crate::interactive::{
    app::{FocussedPane::*, TerminalApp},
    sorted_entries,
    widgets::{HelpPane, MarkPane},
};
use dua::path_of;
use dua::traverse::TreeIndex;
use itertools::Itertools;
use petgraph::Direction;
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

    pub fn update_message(&mut self) {
        self.state.message = None;
    }

    pub fn open_that(&mut self) {
        if let Some(ref idx) = self.state.selected {
            open::that(path_of(&self.traversal.tree, *idx)).ok();
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
            let new_entries = sorted_entries(&self.traversal.tree, new_root, self.state.sorting);
            match new_entries.get(0) {
                Some(b) => {
                    self.state.root = new_root;
                    self.state.selected = Some(b.index);
                    self.state.entries = new_entries;
                }
                None => self.state.message = Some("Entry is a file or an empty directory".into()),
            }
        }
    }

    pub fn change_entry_selection(&mut self, direction: CursorDirection) {
        let entries = sorted_entries(&self.traversal.tree, self.state.root, self.state.sorting);
        let next_selected_pos = match self.state.selected {
            Some(ref selected) => entries
                .iter()
                .find_position(|b| b.index == *selected)
                .map(|(idx, _)| direction.move_cursor(idx))
                .unwrap_or(0),
            None => 0,
        };
        self.state.selected = entries
            .get(next_selected_pos)
            .or_else(|| entries.last())
            .map(|b| b.index)
            .or(self.state.selected)
    }

    pub fn cycle_sorting(&mut self) {
        self.state.sorting.toggle_size();
        self.state.entries =
            sorted_entries(&self.traversal.tree, self.state.root, self.state.sorting);
    }

    pub fn mark_entry(&mut self, advance_cursor: bool) {
        match (self.state.selected, self.window.mark_pane.take()) {
            (Some(index), Some(pane)) => {
                self.window.mark_pane = pane.toggle_index(index, &self.traversal.tree);
            }
            (Some(index), None) => {
                self.window.mark_pane =
                    MarkPane::default().toggle_index(index, &self.traversal.tree)
            }
            _ => {}
        };
        if advance_cursor {
            self.change_entry_selection(CursorDirection::Down)
        }
    }

    pub fn delete_entry(&mut self, _index: TreeIndex) -> Result<(), usize> {
        Ok(())
    }

    pub fn dispatch_to_mark_pane<B>(&mut self, key: Key, terminal: &mut Terminal<B>)
    where
        B: Backend,
    {
        let res = self.window.mark_pane.take().and_then(|p| p.key(key));
        self.window.mark_pane = match res {
            Some((mut pane, mode)) => match mode {
                Some(MarkMode::Delete) => {
                    while let Some(entry_to_delete) = pane.next_entry_for_deletion() {
                        self.draw(terminal).ok();
                        match self.delete_entry(entry_to_delete) {
                            Ok(_) => match pane.delete_entry() {
                                Some(p) => pane = p,
                                None => break,
                            },
                            Err(num_errors) => pane.set_error_on_marked_item(num_errors),
                        }
                    }
                    None
                }
                None => Some(pane),
            },
            None => {
                self.state.focussed = Main;
                None
            }
        };
    }
}
