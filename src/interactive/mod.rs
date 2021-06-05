mod app;
pub use app::*;

pub mod widgets;

mod utils {
    use dua::{
        get_entry_or_panic,
        traverse::{Tree, TreeIndex},
    };
    use std::path::PathBuf;

    pub fn path_of(tree: &Tree, mut node_idx: TreeIndex) -> PathBuf {
        const THE_ROOT: usize = 1;
        let mut entries = Vec::new();

        while let Some(parent_idx) = tree.neighbors_directed(node_idx, petgraph::Incoming).next() {
            entries.push(get_entry_or_panic(tree, node_idx));
            node_idx = parent_idx;
        }
        entries.push(get_entry_or_panic(tree, node_idx));
        entries
            .iter()
            .rev()
            .skip(THE_ROOT)
            .fold(PathBuf::new(), |mut acc, entry| {
                acc.push(&entry.name);
                acc
            })
    }
}
pub use utils::path_of;
