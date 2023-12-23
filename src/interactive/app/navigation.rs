use dua::traverse::TreeIndex;
use itertools::Itertools;
use std::collections::BTreeMap;

use super::{CursorDirection, EntryDataBundle};

#[derive(Default)]
pub struct Navigation {
    pub tree_root: TreeIndex,
    pub view_root: TreeIndex,
    pub selected: Option<TreeIndex>,
    pub bookmarks: BTreeMap<TreeIndex, TreeIndex>,
}

impl Navigation {
    pub fn previously_selected_index(
        &self,
        view_root: TreeIndex,
        entries: &[EntryDataBundle],
    ) -> Option<TreeIndex> {
        let idx = self
            .bookmarks
            .get(&view_root)
            .and_then(|selected| {
                entries
                    .iter()
                    .find_position(|b| b.index == *selected)
                    .map(|(pos, _)| pos)
            })
            .unwrap_or(0);
        entries.get(idx).map(|a| a.index)
    }

    pub fn enter_node(&mut self, previously_selected: TreeIndex, new_selected: TreeIndex) {
        let view_root = self.view_root;
        self.bookmarks.insert(view_root, previously_selected);
        self.view_root = previously_selected;
        self.selected = Some(new_selected);
    }

    pub fn exit_node(&mut self, parent_idx: TreeIndex, entries: &[EntryDataBundle]) {
        self.view_root = parent_idx;
        self.selected = self
            .bookmarks
            .get(&parent_idx)
            .copied()
            .or_else(|| entries.first().map(|b| b.index));
    }

    pub fn next_index(
        &self,
        direction: CursorDirection,
        entries: &[EntryDataBundle],
    ) -> Option<TreeIndex> {
        let next_selected_pos = match self.selected {
            Some(ref selected) => entries
                .iter()
                .find_position(|b| b.index == *selected)
                .map(|(idx, _)| direction.move_cursor(idx))
                .unwrap_or(0),
            None => 0,
        };

        entries
            .get(next_selected_pos)
            .or_else(|| entries.last())
            .map(|b| b.index)
            .or(self.selected)
    }

    pub fn select(&mut self, selected: Option<TreeIndex>) {
        self.selected = selected;
        if let Some(selected) = selected {
            self.bookmarks.insert(self.view_root, selected);
        }
    }
}
