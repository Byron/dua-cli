//! Apply confirmed filesystem removals to the UI's scanned tree.
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossbeam::channel::Receiver;
use dua::{
    ByteFormat, Config, WalkResult,
    traverse::{Traversal, TreeIndex},
};

use super::{
    deletion::{DeletionEvent, DeletionTask},
    navigation::Navigation,
    notification,
    state::{AppState, FocussedPane},
    tree_view::TreeView,
};
use crate::interactive::{
    DisplayOptions,
    widgets::{Language, MainWindow, MarkMode},
};

struct Target {
    path: PathBuf,
    // Clear this as soon as the root disappears: glob searches can reuse vacant tree slots.
    index: Option<TreeIndex>,
}

pub(super) struct FilesystemDeletion {
    pub task: DeletionTask,
    pub tick: Receiver<Instant>,
    pub input_closed: bool,
    pub pending: Vec<DeletionEvent>,
    targets: Vec<Target>,
    children: HashMap<TreeIndex, HashMap<OsString, TreeIndex>>,
    format: ByteFormat,
    remaining: u128,
    bytes_removed: u128,
    entries_removed: usize,
}

impl FilesystemDeletion {
    pub fn message(&self, language: Language) -> String {
        if self.task.cancel.load(Ordering::Relaxed) {
            language.ui_text().cancelling_deletion.into()
        } else {
            language.deletion_progress(
                self.entries_removed,
                &self.format.display(self.remaining).to_string(),
                self.is_trash(),
            )
        }
    }

    fn is_trash(&self) -> bool {
        match self.task.mode {
            MarkMode::Delete => false,
            #[cfg(feature = "trash-move")]
            MarkMode::Trash => true,
        }
    }

    fn lookup(&mut self, target: usize, path: &Path, tree: &TreeView<'_>) -> Option<TreeIndex> {
        let target = self.targets.get(target)?;
        let mut index = target.index?;
        for name in path.strip_prefix(&target.path).ok()?.components() {
            let children = self.children.entry(index).or_insert_with(|| {
                tree.tree()
                    .children(index)
                    .filter_map(|child| {
                        Some((
                            tree.tree().name(child)?.into_owned().into_os_string(),
                            child,
                        ))
                    })
                    .collect()
            });
            index = *children.get(name.as_os_str())?;
        }
        Some(index)
    }
}

impl AppState {
    pub fn is_deleting(&self) -> bool {
        self.deletion.is_some()
    }

    pub(super) fn block_deletion_changes(&mut self) -> bool {
        if self.is_deleting() {
            self.message = Some(self.language.ui_text().deletion_running.into());
            true
        } else {
            false
        }
    }

    pub(super) fn start_deletion(
        &mut self,
        window: &MainWindow,
        tree: &TreeView<'_>,
        display: DisplayOptions,
        mode: MarkMode,
    ) -> Result<()> {
        if self.block_deletion_changes() {
            return Ok(());
        }
        if self.read_only {
            anyhow::bail!(self.language.ui_text().snapshots_read_only);
        }
        if self.scan.is_some() {
            anyhow::bail!(self.language.ui_text().traversal_running);
        }
        let Some(pane) = &window.mark else {
            return Ok(());
        };
        let mut targets: Vec<_> = pane
            .marked()
            .keys()
            .filter_map(|&index| {
                let data = tree.tree().data(index)?;
                (index != tree.traversal.root_index)
                    .then(|| (tree.path_of(index), index, data.is_dir))
            })
            .collect();
        targets.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut roots: Vec<Target> = Vec::new();
        let mut last_directory: Option<PathBuf> = None;
        for (path, index, is_dir) in targets {
            if roots.last().is_some_and(|root| root.path == path)
                || last_directory
                    .as_ref()
                    .is_some_and(|parent| path.starts_with(parent))
            {
                continue;
            }
            last_directory = is_dir.then(|| path.clone());
            roots.push(Target {
                path,
                index: Some(index),
            });
        }
        if roots.is_empty() {
            return Ok(());
        }
        let task = DeletionTask::start(
            roots.iter().map(|target| target.path.clone()).collect(),
            self.walk_options.threads,
            mode,
        )?;
        self.deletion = Some(FilesystemDeletion {
            task,
            tick: crossbeam::channel::tick(Duration::from_secs(1)),
            input_closed: false,
            pending: Vec::new(),
            targets: roots,
            children: HashMap::new(),
            format: display.byte_format,
            remaining: pane.total_size(),
            bytes_removed: 0,
            entries_removed: 0,
        });
        self.pending_exit = false;
        self.reset_message();
        Ok(())
    }

    /// Turn an actual exit request into cancellation followed by draining the worker.
    pub(super) fn exit_after_deletion(&mut self) -> Option<WalkResult> {
        if let Some(deletion) = &self.deletion {
            deletion.task.cancel.store(true, Ordering::Relaxed);
            self.pending_exit = false;
            self.reset_message();
            None
        } else {
            Some(WalkResult {
                num_errors: self.stats.io_errors,
            })
        }
    }

    pub(super) fn flush_deletion(
        &mut self,
        traversal: &mut Traversal,
        window: &mut MainWindow,
        config: &Config,
    ) -> Result<Option<WalkResult>> {
        let Some(mut deletion) = self.deletion.take() else {
            return Ok(None);
        };
        if deletion.pending.is_empty() {
            self.deletion = Some(deletion);
            self.reset_message();
            return Ok(None);
        }
        let mut tree = self.tree_view(traversal);
        let mut removed = HashSet::new();
        let mut target_errors = Vec::new();
        let mut finished = None;
        for event in std::mem::take(&mut deletion.pending) {
            match event {
                DeletionEvent::Removed { target, path } => {
                    if let Some(index) = deletion.lookup(target, &path, &tree) {
                        removed.insert(index);
                    }
                }
                DeletionEvent::TargetFinished { target, errors } => {
                    target_errors.push((target, errors));
                }
                DeletionEvent::Finished {
                    entries,
                    errors,
                    cancelled,
                } => {
                    finished = Some((entries, errors, cancelled));
                }
            }
        }
        // A successful directory removal subsumes its descendants in the same batch.
        let roots: HashSet<_> = removed
            .iter()
            .copied()
            .filter(|&index| {
                let mut parent = tree.fs_parent_of(index);
                while let Some(index) = parent {
                    if removed.contains(&index) {
                        return false;
                    }
                    parent = tree.fs_parent_of(index);
                }
                true
            })
            .collect();
        repair_removed_navigation(&mut self.navigation, &tree, &roots);
        if let Some(navigation) = &mut self.glob_navigation {
            repair_removed_navigation(navigation, &tree, &roots);
            tree.glob_matches = Some(navigation.matches.clone());
        }

        let mut deltas = HashMap::<TreeIndex, (u128, u64)>::new();
        let mut parents = HashMap::<TreeIndex, HashSet<TreeIndex>>::new();
        for &index in &roots {
            let Some(data) = tree.tree().data(index) else {
                continue;
            };
            let Some(parent) = tree.fs_parent_of(index) else {
                continue;
            };
            deletion.bytes_removed += data.size;
            let mut ancestor = Some(parent);
            while let Some(index) = ancestor {
                let delta = deltas.entry(index).or_default();
                delta.0 += data.size;
                delta.1 += data.entry_count.unwrap_or(1);
                ancestor = tree.fs_parent_of(index);
            }
            parents.entry(parent).or_default().insert(index);
            if let Some(children) = deletion.children.get_mut(&parent)
                && let Some(name) = tree.tree().name(index)
            {
                children.remove(name.as_os_str());
            }
            // Invalidate only removed directory caches, without scanning every cached parent.
            let mut pending = vec![index];
            while let Some(index) = pending.pop() {
                deletion.children.remove(&index);
                pending.extend(tree.tree().children(index));
            }
        }
        for target in &mut deletion.targets {
            if target.index.is_some_and(|index| roots.contains(&index)) {
                target.index = None;
            }
        }
        for (parent, children) in parents {
            deletion.entries_removed += tree.traversal.remove_children(parent, &children);
        }
        for (index, (bytes, count)) in deltas {
            tree.tree_mut().update(index, |entry| {
                entry.size = entry.size.saturating_sub(bytes);
                if let Some(entries) = &mut entry.entry_count {
                    *entries = entries.saturating_sub(count);
                }
            });
        }
        if let Some(pane) = &mut window.mark {
            pane.reconcile(&tree);
            for (target, errors) in target_errors {
                if errors == 0 {
                    continue;
                }
                let path = &deletion.targets[target].path;
                let indices: Vec<_> = pane
                    .marked()
                    .iter()
                    .filter_map(|(&index, mark)| mark.path.starts_with(path).then_some(index))
                    .collect();
                for index in indices {
                    pane.set_deletion_error(index, errors);
                }
            }
            deletion.remaining = pane.total_size();
            if pane.is_empty() {
                window.mark = None;
                if self.focussed == FocussedPane::Mark {
                    self.focussed = FocussedPane::Main;
                }
            }
        }
        self.stats.total_bytes = Some(tree.total_size());
        let should_exit = deletion.task.cancel.load(Ordering::Relaxed) || deletion.input_closed;
        self.deletion = Some(deletion);
        self.sync_clean_hub(&mut tree);
        self.update_entries(&tree);
        self.reset_message();
        if let Some((entries, errors, cancelled)) = finished {
            let mut deletion = self.deletion.take().expect("active deletion");
            deletion
                .task
                .join()
                .map_err(|_| anyhow::anyhow!("Deletion worker panicked"))?;
            if !cancelled && !deletion.task.cancel.load(Ordering::Relaxed) {
                let action = match deletion.task.mode {
                    MarkMode::Delete => self.language.ui_text().notification_deletion,
                    #[cfg(feature = "trash-move")]
                    MarkMode::Trash => self.language.ui_text().notification_trash,
                };
                let message = notification::deletion_finished(
                    self.language,
                    action,
                    if deletion.is_trash() {
                        deletion.entries_removed
                    } else {
                        entries
                    },
                    (errors == 0).then_some(deletion.bytes_removed),
                    deletion.task.started.elapsed(),
                    errors,
                    deletion.format,
                );
                if let Err(err) = notification::emit_if_unfocused(
                    config.notifications.delete_finished,
                    self.terminal_focus.is_focussed(),
                    &message,
                ) {
                    log::debug!("Could not emit terminal notification: {err}");
                }
            }
            if should_exit {
                self.reset_message();
                return Ok(Some(WalkResult {
                    num_errors: self.stats.io_errors,
                }));
            }
            self.update_entry_annotations(&tree);
            self.reset_message();
        }
        Ok(None)
    }
}

fn repair_removed_navigation(
    navigation: &mut Navigation,
    tree: &TreeView<'_>,
    removed: &HashSet<TreeIndex>,
) {
    let is_removed = |mut index| {
        loop {
            if removed.contains(&index) {
                return true;
            }
            match tree.fs_parent_of(index) {
                Some(parent) => index = parent,
                None => return false,
            }
        }
    };
    while is_removed(navigation.view_root) {
        navigation.view_root = if navigation
            .matches
            .binary_search(&navigation.view_root)
            .is_ok()
        {
            navigation.tree_root
        } else {
            tree.fs_parent_of(navigation.view_root)
                .unwrap_or(navigation.tree_root)
        };
    }
    navigation.selected = navigation.selected.filter(|&index| !is_removed(index));
    navigation
        .bookmarks
        .retain(|&view, selected| !is_removed(view) && !is_removed(*selected));
    if navigation.matches.iter().any(|&index| is_removed(index)) {
        navigation.matches = navigation
            .matches
            .iter()
            .copied()
            .filter(|&index| !is_removed(index))
            .collect();
    }
}

#[cfg(test)]
impl FilesystemDeletion {
    pub(super) fn for_test(
        tree: &TreeView<'_>,
        targets: Vec<TreeIndex>,
        events: Receiver<DeletionEvent>,
    ) -> Self {
        let remaining = targets
            .iter()
            .filter_map(|&index| tree.tree().data(index).map(|entry| entry.size))
            .sum();
        Self {
            task: DeletionTask::from_events(events),
            tick: crossbeam::channel::tick(Duration::from_secs(1)),
            input_closed: false,
            pending: Vec::new(),
            targets: targets
                .into_iter()
                .map(|index| Target {
                    path: tree.path_of(index),
                    index: Some(index),
                })
                .collect(),
            children: HashMap::new(),
            format: ByteFormat::Metric,
            remaining,
            bytes_removed: 0,
            entries_removed: 0,
        }
    }
}
