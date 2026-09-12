use super::{EntryDataBundle, SortMode, sorted_entries};
use crate::interactive::{EntryCheck, path_of};
use dua::traverse::{Traversal, Tree, TreeIndex};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct TreeView<'a> {
    pub traversal: &'a mut Traversal,
    /// Detached browser root and its visible real nodes, independent of glob results.
    pub scope: Option<(TreeIndex, Arc<[TreeIndex]>)>,
    pub glob_tree_root: Option<TreeIndex>,
    pub glob_matches: Option<Arc<[TreeIndex]>>,
}

impl TreeView<'_> {
    pub fn tree(&self) -> &Tree {
        &self.traversal.tree
    }

    pub fn tree_mut(&mut self) -> &mut Tree {
        &mut self.traversal.tree
    }

    pub fn fs_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        self.traversal.tree.parent(idx)
    }

    pub fn view_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        if let Some((root, matches)) = self.glob_tree_root.zip(self.glob_matches.as_deref())
            && matches.binary_search(&idx).is_ok()
        {
            return Some(root);
        }
        self.scope
            .as_ref()
            .filter(|(_, members)| members.binary_search(&idx).is_ok())
            .map(|(root, _)| *root)
            .or_else(|| self.fs_parent_of(idx))
    }

    pub fn children(&self, root: TreeIndex) -> Vec<TreeIndex> {
        if self.glob_tree_root == Some(root) {
            self.glob_matches.as_deref().unwrap_or_default().to_vec()
        } else if let Some((scope, members)) = &self.scope
            && *scope == root
        {
            members.to_vec()
        } else {
            self.tree().children(root).collect()
        }
    }

    pub fn name_in(&self, root: TreeIndex, index: TreeIndex) -> Option<PathBuf> {
        if self.glob_tree_root == Some(root) {
            return self.exists(index).then(|| self.path_of(index));
        }
        let name = self.tree().name(index)?;
        if self.scope.as_ref().is_some_and(|(scope, _)| *scope == root) {
            Some(
                self.path_of(index)
                    .strip_prefix(self.tree().name(root)?)
                    .ok()?
                    .to_owned(),
            )
        } else {
            Some(name.into_owned())
        }
    }

    pub fn path_of(&self, node_idx: TreeIndex) -> PathBuf {
        path_of(&self.traversal.tree, node_idx, self.glob_tree_root)
    }

    /// Find a filesystem directory after a refresh replaced its tree index.
    pub fn index_at_path(&self, path: &Path) -> Option<TreeIndex> {
        if let Some((root, _)) = &self.scope
            && self.tree().name(*root).as_deref() == Some(path)
        {
            return Some(*root);
        }
        let mut index = self.traversal.root_index;
        loop {
            if self.path_of(index) == path {
                return Some(index);
            }
            index = self.tree().children(index).find(|&child| {
                self.tree().data(child).is_some_and(|entry| entry.is_dir)
                    && path.starts_with(self.path_of(child))
            })?;
        }
    }

    pub fn sorted_entries(
        &self,
        view_root: TreeIndex,
        sorting: SortMode,
        check: EntryCheck,
    ) -> Vec<EntryDataBundle> {
        let use_full_path = self.glob_tree_root == Some(view_root);
        let scoped = self
            .scope
            .as_ref()
            .is_some_and(|(root, _)| *root == view_root);
        let indices = self
            .children(view_root)
            .into_iter()
            .filter(|&index| !scoped || self.name_in(view_root, index).is_some());
        let mut entries =
            sorted_entries(&self.traversal.tree, indices, sorting, use_full_path, check);
        if scoped {
            for entry in &mut entries {
                entry.name = self.name_in(view_root, entry.index).expect("scope member");
            }
        }
        entries
    }

    pub fn current_path(&self, view_root: TreeIndex) -> PathBuf {
        current_path(&self.traversal.tree, view_root, self.glob_tree_root)
    }

    pub fn exists(&self, idx: TreeIndex) -> bool {
        self.tree().contains(idx)
    }

    pub fn total_size(&self) -> u128 {
        self.tree()
            .children(self.traversal.root_index)
            .filter_map(|idx| self.tree().data(idx).map(|entry| entry.size))
            .sum()
    }

    pub fn recompute_sizes_recursively(&mut self, mut index: TreeIndex) {
        loop {
            let (size_of_children, item_count) = self
                .tree()
                .children(index)
                .filter_map(|idx| {
                    self.tree()
                        .data(idx)
                        .map(|entry| (entry.size, entry.entry_count.unwrap_or(1)))
                })
                .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
                .unwrap_or_default();

            self.traversal.tree.update(index, |entry| {
                entry.size = size_of_children;
                entry.entry_count = Some(item_count);
            });

            match self.fs_parent_of(index) {
                None => break,
                Some(parent) => index = parent,
            }
        }
    }
}

fn current_path(tree: &Tree, root: TreeIndex, glob_root: Option<TreeIndex>) -> PathBuf {
    match path_of(tree, root, glob_root) {
        ref p if p.as_os_str().is_empty() => Path::new(".")
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(".")),
        p => p,
    }
}
