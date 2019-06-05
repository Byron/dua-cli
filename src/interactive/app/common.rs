use dua::path_of;
use dua::traverse::{EntryData, Tree, TreeIndex};
use itertools::Itertools;
use petgraph::Direction;
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    SizeDescending,
    SizeAscending,
}

impl SortMode {
    pub fn toggle_size(&mut self) {
        use SortMode::*;
        *self = match self {
            SizeAscending => SizeDescending,
            SizeDescending => SizeAscending,
        }
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::SizeDescending
    }
}

pub fn sorted_entries(
    tree: &Tree,
    node_idx: TreeIndex,
    sorting: SortMode,
) -> Vec<(TreeIndex, &EntryData, PathBuf)> {
    use SortMode::*;
    tree.neighbors_directed(node_idx, Direction::Outgoing)
        .filter_map(|idx| tree.node_weight(idx).map(|w| (idx, w, path_of(tree, idx))))
        .sorted_by(|(_, l, _), (_, r, _)| match sorting {
            SizeDescending => r.size.cmp(&l.size),
            SizeAscending => l.size.cmp(&r.size),
        })
        .collect()
}
