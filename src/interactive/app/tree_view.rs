use super::{EntryDataBundle, SortMode, sorted_entries};
use crate::interactive::{EntryCheck, path_of};
use dua::traverse::{Traversal, Tree, TreeIndex};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct TreeView<'a> {
    pub traversal: &'a mut Traversal,
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
        match self.glob_tree_root.zip(self.glob_matches.as_deref()) {
            Some((glob_root, matches)) if matches.binary_search(&idx).is_ok() => Some(glob_root),
            _ => self.traversal.tree.parent(idx),
        }
    }

    pub fn path_of(&self, node_idx: TreeIndex) -> PathBuf {
        path_of(&self.traversal.tree, node_idx, self.glob_tree_root)
    }

    pub fn sorted_entries(
        &self,
        view_root: TreeIndex,
        sorting: SortMode,
        check: EntryCheck,
    ) -> Vec<EntryDataBundle> {
        sorted_entries(
            &self.traversal.tree,
            view_root,
            sorting,
            self.glob_tree_root,
            self.glob_matches.as_deref(),
            check,
        )
    }

    pub fn current_path(&self, view_root: TreeIndex) -> PathBuf {
        current_path(&self.traversal.tree, view_root, self.glob_tree_root)
    }

    pub fn remove_entries(&mut self, root_index: TreeIndex, remove_root_node: bool) -> usize {
        if remove_root_node {
            return self.tree_mut().remove_subtree(root_index);
        }
        let children = self.tree().children(root_index).collect::<Vec<_>>();
        children
            .into_iter()
            .map(|child| self.tree_mut().remove_subtree(child))
            .sum()
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
