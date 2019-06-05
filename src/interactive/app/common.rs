use dua::path_of;
use dua::traverse::{EntryData, Tree, TreeIndex};
use itertools::Itertools;
use petgraph::Direction;

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

pub struct EntryDataBundle {
    pub index: TreeIndex,
    pub data: EntryData,
    pub is_dir: bool,
    pub exists: bool,
}

pub fn sorted_entries(tree: &Tree, node_idx: TreeIndex, sorting: SortMode) -> Vec<EntryDataBundle> {
    use SortMode::*;
    tree.neighbors_directed(node_idx, Direction::Outgoing)
        .filter_map(|idx| {
            tree.node_weight(idx).map(|w| {
                let p = path_of(tree, idx);
                EntryDataBundle {
                    index: idx,
                    data: w.clone(),
                    is_dir: p.is_dir(),
                    exists: p.exists(),
                }
            })
        })
        .sorted_by(|l, r| match sorting {
            SizeDescending => r.data.size.cmp(&l.data.size),
            SizeAscending => l.data.size.cmp(&r.data.size),
        })
        .collect()
}
