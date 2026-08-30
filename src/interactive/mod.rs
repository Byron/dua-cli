mod app;
pub use app::*;

pub mod widgets;

mod utils {
    use dua::traverse::{Tree, TreeIndex};
    use std::path::PathBuf;

    pub fn path_of(tree: &Tree, mut node_idx: TreeIndex, _glob_root: Option<TreeIndex>) -> PathBuf {
        let mut entries = Vec::new();

        while let Some(parent_idx) = tree.parent(node_idx) {
            entries.push(
                tree.name(node_idx)
                    .expect("node should always be retrievable with valid index"),
            );
            node_idx = parent_idx;
        }
        entries.push(
            tree.name(node_idx)
                .expect("node should always be retrievable with valid index"),
        );
        entries
            .iter()
            .rev()
            .filter(|name| !name.as_os_str().is_empty())
            .fold(PathBuf::new(), |mut acc, entry| {
                acc.push(entry);
                acc
            })
    }
}

pub use utils::path_of;
