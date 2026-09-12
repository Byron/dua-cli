use crate::interactive::{
    DisplayOptions, EntryDataBundle,
    app::tree_view::TreeView,
    widgets::{Column, GlobPane, HelpPane, MainWindow, MarkPane},
};
use crossterm::event::KeyEvent;
use dua::Config;
use dua::traverse::TreeIndex;
use std::{collections::BTreeSet, path::PathBuf};

use super::state::{
    AppState,
    FocussedPane::{Glob, Help, Main, Mark},
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

#[derive(Clone, Copy)]
enum AnnotationKind {
    Cleanup,
    Gitignored,
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
    pub fn from_key(key: KeyEvent, keys: &dua::KeysConfig) -> Option<Self> {
        [
            (&keys.move_to_top, Self::ToTop),
            (&keys.move_to_bottom, Self::ToBottom),
            (&keys.page_up, Self::PageUp),
            (&keys.move_up, Self::Up),
            (&keys.move_down, Self::Down),
            (&keys.page_down, Self::PageDown),
        ]
        .into_iter()
        .find_map(|(binding, direction)| binding.matches(key).then_some(direction))
    }

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
    pub fn open_path(&mut self, path: Option<PathBuf>) {
        if self.block_deletion_changes() {
            return;
        }
        if let Some(path) = path {
            let t = self.language.ui_text();
            if self.read_only && !path.exists() {
                self.message = Some(format!("{}{}", t.snapshot_path_unavailable, path.display()));
                return;
            }
            if let Err(err) = open::that(&path) {
                self.message = Some(format!("{}{}: {err}", t.failed_to_open, path.display()));
            }
        }
    }

    pub fn exit_node_with_traversal(&mut self, tree_view: &TreeView<'_>, scan_parent_key: &str) {
        if let Some(parent) = tree_view.view_parent_of(self.navigation().view_root) {
            let navigation = self.navigation_mut();
            navigation.view_root = parent;
            navigation.selected = navigation.bookmarks.get(&parent).copied();
            self.update_entries(tree_view);
            self.reset_message();
        } else {
            self.message = Some(if self.can_scan_parent(tree_view) {
                self.language.top_level_with_scan(scan_parent_key)
            } else {
                self.language.ui_text().top_level.into()
            });
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
                None => {
                    self.message = Some(self.language.ui_text().entry_file_or_empty.into());
                }
            }
        }
    }

    pub fn change_entry_selection(&mut self, direction: CursorDirection) {
        let next_index = self.navigation().next_index(direction, &self.entries);
        self.navigation_mut().select(next_index);
    }

    pub fn cycle_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_size();
        self.update_entries(tree_view);
    }

    pub fn cycle_mtime_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_mtime();
        self.update_entries(tree_view);
    }

    pub fn cycle_count_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_count();
        self.update_entries(tree_view);
    }

    pub fn cycle_name_sorting(&mut self, tree_view: &TreeView<'_>) {
        self.sorting.toggle_name();
        self.update_entries(tree_view);
    }

    pub fn cycle_mtime_sort_mode(&mut self, tree_view: &TreeView<'_>) {
        if self.sorting.mtime_sort().is_some() {
            self.sorting.cycle_mtime_sort();
            self.update_entries(tree_view);
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
            self.message = Some(
                self.language
                    .ui_text()
                    .gitignore_snapshot_unavailable
                    .into(),
            );
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
        if let Some(deletion) = &self.deletion {
            self.message = Some(deletion.message(self.language));
        } else if self.scan.is_some() {
            self.message = Some(self.language.ui_text().scanning.into());
        } else if let Some(hub) = self.clean_hub.as_ref().filter(|hub| hub.root.is_none()) {
            self.message = hub
                .is_empty()
                .then(|| self.language.ui_text().no_cleanup_candidates.into());
        } else {
            self.message = self.language.annotation_message(
                self.cleanup_candidates.as_ref().map_or(0, BTreeSet::len),
                self.gitignored_entries.as_ref().map_or(0, BTreeSet::len),
            );
        }
    }

    pub fn toggle_right_panes(&mut self, window: &mut MainWindow) {
        if window.help.is_none() && window.mark.is_none() {
            return;
        }
        window.right_panes_minimized = !window.right_panes_minimized;
        if window.right_panes_minimized {
            if let Some(pane) = window.mark.as_mut() {
                pane.set_focus(false);
            }
            if matches!(self.focussed, Help | Mark) {
                self.focussed = Main;
            }
        }
    }

    pub fn toggle_help_pane(&mut self, window: &mut MainWindow) {
        if window.right_panes_minimized {
            window.right_panes_minimized = false;
            window.help.get_or_insert_with(HelpPane::default);
            self.focussed = Help;
            return;
        }
        self.focussed = match self.focussed {
            Main | Mark | Glob => {
                window.help = Some(HelpPane::default());
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
        if window.right_panes_minimized {
            self.focussed = if self.focussed == Main && window.glob.is_some() {
                Glob
            } else {
                Main
            };
            return;
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

    pub fn dispatch_to_mark_pane(
        &mut self,
        key: KeyEvent,
        window: &mut MainWindow,
        tree_view: &mut TreeView<'_>,
        display: DisplayOptions,
        config: &Config,
    ) {
        if window.right_panes_minimized {
            return;
        }
        let res = window
            .mark
            .take()
            .and_then(|pane| pane.process_events(key, &config.keys, !self.is_deleting()));
        if let Some((pane, mode)) = res {
            window.mark = Some(pane);
            if let Some(mode) = mode
                && let Err(err) = self.start_deletion(window, tree_view, display, mode)
            {
                self.message = Some(err.to_string());
            }
        }
        if window.mark.is_none() && self.focussed == Mark {
            self.focussed = Main;
        }
    }

    fn mark_entry_by_index(
        &mut self,
        index: TreeIndex,
        mode: MarkEntryMode,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
    ) {
        if self.block_deletion_changes() {
            return;
        }
        let Some(data) = tree_view.tree().data(index) else {
            return;
        };
        let is_dir = self
            .entries
            .iter()
            .find(|entry| entry.index == index)
            .map_or(data.is_dir, |entry| entry.is_dir);
        window.mark = window.mark.take().unwrap_or_default().toggle_index(
            index,
            tree_view,
            is_dir,
            matches!(mode, MarkEntryMode::Toggle),
        );
    }

    pub fn mark_entry(
        &mut self,
        cursor: CursorMode,
        mode: MarkEntryMode,
        window: &mut MainWindow,
        tree_view: &TreeView<'_>,
    ) {
        if self.block_deletion_changes() {
            return;
        }
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
        if self.block_deletion_changes() {
            return;
        }
        for index in self.entries.iter().map(|e| e.index).collect::<Vec<_>>() {
            self.mark_entry_by_index(index, mode, window, tree_view);
        }
    }

    pub fn mark_cleanup_candidates(&mut self, window: &mut MainWindow, tree_view: &TreeView<'_>) {
        if self.block_deletion_changes() {
            return;
        }
        match self.cleanup_candidates.clone() {
            Some(cleanup_candidates) => self.mark_annotation_candidates(
                cleanup_candidates,
                AnnotationKind::Cleanup,
                window,
                tree_view,
            ),
            None => {
                self.message = Some(self.language.ui_text().cleanup_detection_disabled.into());
            }
        }
    }

    pub fn mark_gitignored_entries(&mut self, window: &mut MainWindow, tree_view: &TreeView<'_>) {
        if self.block_deletion_changes() {
            return;
        }
        match self.gitignored_entries.clone() {
            Some(gitignored_entries) => self.mark_annotation_candidates(
                gitignored_entries,
                AnnotationKind::Gitignored,
                window,
                tree_view,
            ),
            None => {
                self.message = Some(self.language.ui_text().gitignore_detection_disabled.into());
            }
        }
    }

    fn mark_annotation_candidates(
        &mut self,
        annotation_candidates: BTreeSet<TreeIndex>,
        kind: AnnotationKind,
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
            let t = self.language.ui_text();
            self.message = Some(
                match (kind, annotation_candidates.is_empty()) {
                    (AnnotationKind::Cleanup, true) => t.no_cleanup_candidates,
                    (AnnotationKind::Cleanup, false) => t.cleanup_candidates_already_marked,
                    (AnnotationKind::Gitignored, true) => t.no_gitignored_entries,
                    (AnnotationKind::Gitignored, false) => t.gitignored_entries_already_marked,
                }
                .into(),
            );
        } else {
            self.message =
                Some(self.language.marked_candidates(
                    candidates.len(),
                    matches!(kind, AnnotationKind::Gitignored),
                ));
        }
    }

    pub(super) fn update_entries(&mut self, tree: &TreeView<'_>) {
        if self
            .clean_hub
            .as_ref()
            .is_some_and(|hub| hub.root.is_none())
        {
            self.entries.clear();
            return;
        }
        self.entries = tree.sorted_entries(
            self.navigation().view_root,
            self.sorting,
            self.entry_check(),
        );
        let selected = self
            .navigation()
            .selected
            .filter(|selected| self.entries.iter().any(|entry| entry.index == *selected))
            .or_else(|| self.entries.first().map(|entry| entry.index));
        self.navigation_mut().selected = selected;
        self.update_entry_annotations(tree);
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
            if self.is_deleting() {
                // Progress and navigation must not reread a potentially large Git index.
                // Retain known annotations until completion can check the current view again.
                if let Some(known) = &self.gitignored_entries {
                    self.gitignored_entries = Some(
                        self.entries
                            .iter()
                            .filter_map(|entry| known.contains(&entry.index).then_some(entry.index))
                            .collect(),
                    );
                }
            } else if self.gitignored_entries.is_some() {
                self.gitignored_entries = Some(super::gitignore::gitignored_entries(
                    tree_view,
                    &self.display_path(tree_view),
                    &self.entries,
                ));
            }
        }
    }
}
