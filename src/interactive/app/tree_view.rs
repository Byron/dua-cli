use super::{sorted_entries, EntryDataBundle, SortMode};
use crate::interactive::path_of;
use dua::traverse::{EntryData, Traversal, Tree, TreeIndex};
use petgraph::{visit::Bfs, Direction};
use std::path::{Path, PathBuf};

pub trait TreeView {
    fn traversal(&self) -> &Traversal;
    fn traversal_as_mut(&mut self) -> &mut Traversal;

    fn tree(&self) -> &Tree;
    fn tree_as_mut(&mut self) -> &mut Tree;

    fn fs_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex>;
    fn view_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        self.fs_parent_of(idx)
    }

    fn exists(&self, idx: TreeIndex) -> bool {
        self.tree().node_weight(idx).is_some()
    }

    fn path_of(&self, node_idx: TreeIndex) -> PathBuf {
        path_of(self.tree(), node_idx, None)
    }

    fn sorted_entries(&self, view_root: TreeIndex, sorting: SortMode) -> Vec<EntryDataBundle> {
        sorted_entries(self.tree(), view_root, sorting, None)
    }

    fn current_path(&self, view_root: TreeIndex) -> String {
        current_path(self.tree(), view_root, None)
    }

    fn remove_entries(&mut self, index: TreeIndex) -> usize;
    fn recompute_sizes_recursively(&mut self, index: TreeIndex);
}

pub struct NormalTreeView<'a> {
    pub traversal: &'a mut Traversal,
}

impl<'a> TreeView for NormalTreeView<'a> {
    fn tree(&self) -> &Tree {
        &self.traversal.tree
    }

    fn tree_as_mut(&mut self) -> &mut Tree {
        &mut self.traversal.tree
    }

    fn fs_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        self.traversal
            .tree
            .neighbors_directed(idx, Direction::Incoming)
            .next()
    }

    fn remove_entries(&mut self, index: TreeIndex) -> usize {
        remove_entries(self.traversal, index)
    }

    fn traversal(&self) -> &Traversal {
        self.traversal
    }

    fn traversal_as_mut(&mut self) -> &mut Traversal {
        self.traversal
    }

    fn recompute_sizes_recursively(&mut self, mut index: TreeIndex) {
        loop {
            self.traversal
                .tree
                .node_weight_mut(index)
                .expect("valid index")
                .size = neighbours_size(self.tree(), index);

            match self.fs_parent_of(index) {
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
}

pub struct GlobTreeView<'a> {
    pub traversal: &'a mut Traversal,
    pub glob_tree_root: TreeIndex,
}

impl<'a> TreeView for GlobTreeView<'a> {
    fn tree(&self) -> &Tree {
        &self.traversal.tree
    }

    fn tree_as_mut(&mut self) -> &mut Tree {
        &mut self.traversal.tree
    }

    fn fs_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        let iter = self
            .traversal
            .tree
            .neighbors_directed(idx, petgraph::Incoming);

        let mut parent = None;
        for parent_idx in iter {
            if parent_idx == self.glob_tree_root {
                continue;
            }
            parent = Some(parent_idx);
        }
        parent
    }

    fn view_parent_of(&self, idx: TreeIndex) -> Option<TreeIndex> {
        let iter = self
            .traversal
            .tree
            .neighbors_directed(idx, petgraph::Incoming);

        let mut parent = None;
        for parent_idx in iter {
            parent = Some(parent_idx);
            if parent_idx == self.glob_tree_root {
                break;
            }
        }
        parent
    }

    fn remove_entries(&mut self, index: TreeIndex) -> usize {
        remove_entries(self.traversal, index)
    }

    fn path_of(&self, node_idx: TreeIndex) -> PathBuf {
        path_of(&self.traversal.tree, node_idx, Some(self.glob_tree_root))
    }

    fn sorted_entries(&self, view_root: TreeIndex, sorting: SortMode) -> Vec<EntryDataBundle> {
        sorted_entries(
            &self.traversal.tree,
            view_root,
            sorting,
            Some(self.glob_tree_root),
        )
    }

    fn current_path(&self, view_root: TreeIndex) -> String {
        current_path(&self.traversal.tree, view_root, Some(self.glob_tree_root))
    }

    fn traversal(&self) -> &Traversal {
        self.traversal
    }

    fn traversal_as_mut(&mut self) -> &mut Traversal {
        self.traversal
    }

    fn recompute_sizes_recursively(&mut self, mut index: TreeIndex) {
        loop {
            self.traversal
                .tree
                .node_weight_mut(index)
                .expect("valid index")
                .size = neighbours_size(self.tree(), index);

            match self.fs_parent_of(index) {
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
}

fn neighbours_size(tree: &Tree, index: TreeIndex) -> u128 {
    tree.neighbors_directed(index, Direction::Outgoing)
        .filter_map(|idx| tree.node_weight(idx).map(|w| w.size))
        .sum()
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

fn remove_entries(traversal: &mut Traversal, index: TreeIndex) -> usize {
    let mut entries_deleted = 0;

    let mut bfs = Bfs::new(&traversal.tree, index);
    while let Some(nx) = bfs.next(&traversal.tree) {
        traversal.tree.remove_node(nx);
        traversal.entries_traversed -= 1;
        entries_deleted += 1;
    }

    entries_deleted
}
