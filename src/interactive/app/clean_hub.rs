use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use crossterm::event::KeyEvent;
use dua::{
    Config,
    traverse::{BackgroundTraversal, EntryData, Traversal, TreeIndex},
};
use tui::{buffer::Buffer, layout::Rect};

use super::{
    CursorDirection, EntryCheck, EntryDataBundle, SortMode,
    eventloop::Refresh,
    navigation::Navigation,
    sorted_entries,
    state::{AppState, FocussedPane},
    tree_view::TreeView,
};
use crate::interactive::widgets::{Entries, EntriesProps, EntryMarkMap, EntryRow, MainWindow};

/// Candidate discovery and grouping stay outside the ordinary filesystem browser.
pub struct CleanHub {
    pub inputs: Vec<PathBuf>,
    pub depth: Option<usize>,
    /// Detached browser root while a row is open; `None` displays the hub.
    pub root: Option<TreeIndex>,
    pub(super) rows: Vec<HubRow>,
    selected: usize,
}

pub(super) struct HubRow {
    pub entry: EntryDataBundle,
    pub members: Vec<TreeIndex>,
}

impl CleanHub {
    pub fn new(inputs: Vec<PathBuf>, depth: Option<usize>) -> Self {
        Self {
            inputs,
            depth,
            root: None,
            rows: Vec::new(),
            selected: 0,
        }
    }

    pub fn title(&self) -> PathBuf {
        self.inputs
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
            .into()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.rows
            .get(self.selected)
            .map(|row| row.entry.name.as_path())
    }

    pub fn selected_members(&self) -> &[TreeIndex] {
        self.rows
            .get(self.selected)
            .map_or(&[], |row| row.members.as_slice())
    }

    pub fn select_path(&mut self, path: &Path) {
        if let Some(position) = self
            .rows
            .iter()
            .position(|row| row.entry.name == path)
            .or_else(|| {
                self.rows.iter().position(|row| {
                    row.entry.name.starts_with(path) || path.starts_with(&row.entry.name)
                })
            })
        {
            self.selected = position;
        }
    }

    pub fn update(&mut self, traversal: &mut Traversal, navigation: &mut Navigation) {
        let tree = &traversal.tree;
        if let Some(root) = self.root {
            let path = tree.name(root).expect("open cleanup root exists");
            let mut members = Vec::new();
            for index in tree.children(traversal.root_index) {
                let name = tree.name(index).expect("candidate exists");
                if name == path {
                    members.extend(tree.children(index));
                } else if name.starts_with(path.as_ref()) {
                    members.push(index);
                }
            }
            members.sort_unstable();
            navigation.matches = members.into();
            return;
        }

        let candidates = sorted_entries(
            tree,
            tree.children(traversal.root_index),
            SortMode::default(),
            false,
            EntryCheck::Disabled,
        );
        let mut parents = BTreeMap::new();
        for entry in &candidates {
            *parents
                .entry(entry.name.parent().expect("absolute candidate"))
                .or_insert(0) += 1;
        }
        let parents: BTreeSet<_> = parents
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(path, _)| path.to_owned())
            .collect();
        let mut rows = BTreeMap::<PathBuf, HubRow>::new();
        for mut entry in candidates {
            if let Some(parent) = entry
                .name
                .ancestors()
                .skip(1)
                .filter_map(|path| parents.get(path))
                .last()
            {
                entry.name.clone_from(parent);
            }
            match rows.entry(entry.name.clone()) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    let index = entry.index;
                    slot.insert(HubRow {
                        entry,
                        members: vec![index],
                    });
                }
                std::collections::btree_map::Entry::Occupied(mut slot) => {
                    let row = slot.get_mut();
                    row.entry.size += entry.size;
                    row.entry.mtime = row.entry.mtime.max(entry.mtime);
                    *row.entry.entry_count.get_or_insert(0) += entry.entry_count.unwrap_or(0);
                    row.members.push(entry.index);
                }
            }
        }
        let selected = self.selected_path().map(Path::to_owned);
        self.rows = rows.into_values().collect();
        self.rows
            .sort_by_key(|row| std::cmp::Reverse(row.entry.size));
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
        if let Some(path) = selected {
            self.select_path(&path);
        }
    }

    fn move_selection(&mut self, direction: CursorDirection) {
        self.selected = direction
            .move_cursor(self.selected)
            .min(self.rows.len().saturating_sub(1));
    }

    pub fn render(
        &self,
        widget: &mut Entries,
        props: EntriesProps<'_>,
        marked: Option<&EntryMarkMap>,
        cleanup: bool,
        area: Rect,
        buffer: &mut Buffer,
    ) {
        let language = props.language;
        let entries = self.rows.iter().map(|row| EntryRow {
            entry: &row.entry,
            marked: marked
                .is_some_and(|marks| row.members.iter().all(|member| marks.contains_key(member))),
            cleanup,
            gitignored: false,
            suffix: (row.members.len() > 1)
                .then(|| language.cleanup_group_label(row.members.len())),
        });
        let hint = props.is_focussed.then(|| {
            format!(
                " → = {} | {} = {} | ↻ = {}/{} ",
                props.keys.descend.primary(),
                props.language.ui_text().entries_mark_toggle,
                props.keys.toggle_mark.primary(),
                props.keys.refresh_selected.primary(),
                props.keys.refresh_all.primary(),
            )
        });
        let style = props.border_style;
        widget.render(
            EntriesProps {
                current_path: self.title(),
                selected: (!self.rows.is_empty()).then_some(self.selected),
                sort_mode: SortMode::SizeDescending,
                show_hints: false,
                ..props
            },
            entries,
            area,
            buffer,
        );
        if let Some(hint) = hint
            && area.height > 0
            && area.width > 2
        {
            buffer.set_stringn(
                area.x + 1,
                area.bottom() - 1,
                hint,
                area.width.saturating_sub(2) as usize,
                style,
            );
        }
    }
}

impl AppState {
    pub(super) fn clean_refresh(
        &self,
        tree: &TreeView<'_>,
        index: TreeIndex,
    ) -> Result<(Vec<TreeIndex>, BackgroundTraversal)> {
        let root = tree.traversal.root_index;
        let hub = self.clean_hub.as_ref().expect("clean refresh");
        if index == root {
            return Ok((
                tree.tree().children(root).collect(),
                BackgroundTraversal::start_clean(
                    root,
                    &self.walk_options,
                    hub.inputs.clone(),
                    hub.depth,
                    None,
                )?,
            ));
        }
        let indices = if hub.root.is_none() {
            hub.selected_members().to_vec()
        } else if hub.root == Some(index) {
            let path = tree.path_of(index);
            tree.tree()
                .children(root)
                .filter(|&candidate| tree.path_of(candidate).starts_with(&path))
                .collect()
        } else {
            // Any interior refresh revalidates the entire candidate's deletion scope.
            vec![
                std::iter::successors(Some(index), |index| tree.fs_parent_of(*index))
                    .find(|index| tree.fs_parent_of(*index) == Some(root))
                    .context("Cleanup entry has no candidate root")?,
            ]
        };
        let candidates = indices
            .iter()
            .map(|&index| {
                let path = tree.path_of(index);
                let search_root = tree
                    .traversal
                    .clean_search_roots
                    .get(&path)
                    .context("Cleanup candidate has no discovery root")?
                    .clone();
                Ok((path, search_root))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok((
            indices,
            BackgroundTraversal::refresh_clean_candidates(root, &self.walk_options, candidates)?,
        ))
    }

    pub(super) fn sync_clean_hub(&mut self, tree: &mut TreeView<'_>) {
        if let Some(hub) = &mut self.clean_hub {
            hub.update(tree.traversal, &mut self.navigation);
            tree.scope = hub
                .root
                .map(|root| (root, Arc::clone(&self.navigation.matches)));
        }
    }

    pub(super) fn process_clean_key(
        &mut self,
        key: KeyEvent,
        window: &mut MainWindow,
        tree: &mut TreeView<'_>,
        config: &Config,
    ) -> anyhow::Result<bool> {
        let Some(hub) = &mut self.clean_hub else {
            return Ok(false);
        };
        let keys = &config.keys;
        if let Some(root) = hub.root {
            if self.focussed == FocussedPane::Main
                && self.glob_navigation.is_none()
                && self.navigation.view_root == root
                && (keys.ascend.matches(key)
                    || (keys.esc_navigates_back && keys.close_pane.matches(key)))
            {
                tree.tree_mut().remove_subtree(root);
                hub.root = None;
                self.navigation = Navigation {
                    tree_root: tree.traversal.root_index,
                    view_root: tree.traversal.root_index,
                    ..Navigation::default()
                };
                self.sorting = SortMode::default();
                self.sync_clean_hub(tree);
                self.update_entries(tree);
                self.reset_message();
                return Ok(true);
            }
            return Ok(false);
        }
        if keys.open_search.matches(key) {
            return Ok(true);
        }
        if self.focussed != FocussedPane::Main {
            return Ok(false);
        }

        if let Some(direction) = CursorDirection::from_key(key, keys) {
            hub.move_selection(direction);
        } else if keys.descend.matches(key) {
            if let Some(path) = hub.selected_path() {
                let root = tree.tree_mut().add_detached(
                    path,
                    EntryData {
                        is_dir: true,
                        ..EntryData::default()
                    },
                );
                hub.root = Some(root);
                self.navigation = Navigation {
                    tree_root: root,
                    view_root: root,
                    ..Navigation::default()
                };
                self.sync_clean_hub(tree);
                self.update_entries(tree);
            }
        } else if keys.refresh_selected.matches(key) {
            self.refresh(tree, window, Refresh::Selected)?;
        } else if keys.refresh_all.matches(key) {
            self.refresh(tree, window, Refresh::AllInView)?;
        } else if keys.toggle_mark.matches(key)
            || keys.mark_for_deletion.matches(key)
            || keys.toggle_mark_and_move_down.matches(key)
            || keys.toggle_all.matches(key)
            || keys.mark_cleanup.matches(key)
        {
            if self.block_deletion_changes() {
                return Ok(true);
            }
            let hub = self.clean_hub.as_mut().expect("cleanup hub");
            let all = keys.toggle_all.matches(key) || keys.mark_cleanup.matches(key);
            let toggle = !keys.mark_for_deletion.matches(key) && !keys.mark_cleanup.matches(key);
            for (_, row) in hub
                .rows
                .iter()
                .enumerate()
                .filter(|(position, _)| all || *position == hub.selected)
            {
                let unmark = toggle
                    && window.mark.as_ref().is_some_and(|pane| {
                        row.members
                            .iter()
                            .all(|index| pane.marked().contains_key(index))
                    });
                for &index in &row.members {
                    window.mark = window
                        .mark
                        .take()
                        .unwrap_or_default()
                        .toggle_index(index, tree, true, unmark);
                }
            }
            if keys.mark_for_deletion.matches(key) || keys.toggle_mark_and_move_down.matches(key) {
                hub.move_selection(CursorDirection::Down);
            }
        } else if keys.open_entry.matches(key) {
            let path = hub.selected_path().map(Path::to_owned);
            self.open_path(path);
            return Ok(true);
        } else if !(keys.sort_by_size.matches(key)
            || keys.sort_by_name.matches(key)
            || keys.sort_by_mtime.matches(key)
            || keys.sort_by_count.matches(key)
            || keys.cycle_mtime_mode.matches(key)
            || keys.toggle_count_column.matches(key)
            || keys.cycle_visualization.matches(key)
            || keys.toggle_cleanup.matches(key)
            || keys.toggle_gitignore.matches(key)
            || keys.mark_gitignore.matches(key)
            || keys.scan_parent.matches(key)
            || keys.ascend.matches(key))
        {
            return Ok(false);
        }
        self.reset_message();
        Ok(true)
    }
}
