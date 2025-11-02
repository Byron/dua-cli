use super::{EntryDataBundle, SortMode, sorted_entries};
use crate::interactive::{EntryCheck, path_of};
use dua::traverse::{EntryData, Traversal, Tree, TreeIndex};
use petgraph::{Direction, visit::Bfs};
use std::path::{Path, PathBuf};

pub struct TreeView<'a> {
    pub traversal: &'a mut Traversal,
    pub glob_tree_root: Option<TreeIndex>,
}

impl TreeView<'_> {
    pub fn tree(&self) -> &Tree {
        &self.traversal.tree
    }

    pub fn tree_mut(&mut self) -> &mut Tree {
        &mut self.traversal.tree
    }

    pub fn fs_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        self.traversal
            .tree
            .neighbors_directed(idx, petgraph::Incoming)
            .find(|idx| match self.glob_tree_root {
                None => true,
                Some(glob_root) => *idx != glob_root,
            })
    }

    pub fn view_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        let mut iter = self
            .traversal
            .tree
            .neighbors_directed(idx, petgraph::Incoming);
        match self.glob_tree_root {
            None => iter.next(),
            Some(glob_root) => iter
                .clone()
                .find(|idx| *idx == glob_root)
                .or_else(|| iter.next()),
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
            check,
        )
    }

    pub fn current_path(&self, view_root: TreeIndex) -> String {
        current_path(&self.traversal.tree, view_root, self.glob_tree_root)
    }

    pub fn remove_entries(&mut self, root_index: TreeIndex, remove_root_node: bool) -> usize {
        let mut entries_deleted = 0;
        let mut bfs = Bfs::new(self.tree(), root_index);

        while let Some(nx) = bfs.next(&self.tree()) {
            if nx == root_index && !remove_root_node {
                continue;
            }
            self.tree_mut().remove_node(nx);
            entries_deleted += 1;
        }
        entries_deleted
    }

    pub fn exists(&self, idx: TreeIndex) -> bool {
        self.tree().node_weight(idx).is_some()
    }

    pub fn total_size(&self) -> u128 {
        self.tree()
            .neighbors_directed(self.traversal.root_index, Direction::Outgoing)
            .filter_map(|idx| self.tree().node_weight(idx).map(|w| w.size))
            .sum()
    }

    pub fn recompute_sizes_recursively(&mut self, mut index: TreeIndex) {
        loop {
            let (size_of_children, item_count) = self
                .tree()
                .neighbors_directed(index, Direction::Outgoing)
                .filter_map(|idx| {
                    self.tree()
                        .node_weight(idx)
                        .map(|w| (w.size, w.entry_count.unwrap_or(1)))
                })
                .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
                .unwrap_or_default();

            let node = self
                .traversal
                .tree
                .node_weight_mut(index)
                .expect("valid index");

            node.size = size_of_children;
            node.entry_count = Some(item_count);

            match self.fs_parent_of(index) {
                None => break,
                Some(parent) => index = parent,
            }
        }
    }
}

fn current_path(
    tree: &petgraph::stable_graph::StableGraph<EntryData, ()>,
    root: petgraph::stable_graph::NodeIndex,
    glob_root: Option<TreeIndex>,
) -> String {
    match path_of(tree, root, glob_root).to_string_lossy().to_string() {
        ref p if p.is_empty() => Path::new(".")
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from(".")),
        p => p,
    }
}
