mod app;
pub use app::*;

pub mod widgets;

use dua::traverse::{Tree, TreeIndex};
use std::path::PathBuf;

pub fn path_of(tree: &Tree, node_idx: TreeIndex, _glob_root: Option<TreeIndex>) -> PathBuf {
    tree.path_of(node_idx)
}
