use crate::{get_size_or_panic};

use filesize::PathExt;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed, Direction};
use std::{
    fmt,
    fs::Metadata,
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub type TreeIndex = NodeIndex;
pub type Tree = StableGraph<EntryData, (), Directed>;

#[derive(Eq, PartialEq, Clone)]
pub struct EntryData {
    pub name: PathBuf,
    /// The entry's size in bytes. If it's a directory, the size is the aggregated file size of all children
    pub size: u128,
    pub mtime: SystemTime,
    pub entry_count: Option<u64>,
    /// If set, the item meta-data could not be obtained
    pub metadata_io_error: bool,
    pub is_dir: bool,
}

impl Default for EntryData {
    fn default() -> EntryData {
        EntryData {
            name: PathBuf::default(),
            size: u128::default(),
            mtime: UNIX_EPOCH,
            entry_count: None,
            metadata_io_error: bool::default(),
            is_dir: false,
        }
    }
}

impl fmt::Debug for EntryData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntryData")
            .field("name", &self.name)
            .field("size", &self.size)
            .field("entry_count", &self.entry_count)
            // Skip mtime
            .field("metadata_io_error", &self.metadata_io_error)
            .finish()
    }
}

/// The result of the previous filesystem traversal
#[derive(Debug)]
pub struct Traversal {
    /// A tree representing the entire filestem traversal
    pub tree: Tree,
    /// The top-level node of the tree.
    pub root_index: TreeIndex,
    /// Amount of files or directories we have seen during the filesystem traversal
    pub entries_traversed: u64,
    /// The time at which the traversal started.
    pub start: std::time::Instant,
    /// The amount of time it took to finish the traversal. Set only once done.
    pub elapsed: Option<std::time::Duration>,
    /// Total amount of IO errors encountered when traversing the filesystem
    pub io_errors: u64,
    /// Total amount of bytes seen during the traversal
    pub total_bytes: Option<u128>,
}

impl Traversal {
    pub fn recompute_root_size(&self) -> u128 {
        self.tree
            .neighbors_directed(self.root_index, Direction::Outgoing)
            .map(|idx| get_size_or_panic(&self.tree, idx))
            .sum()
    }
}

#[cfg(not(windows))]
pub fn size_on_disk(_parent: &Path, name: &Path, meta: &Metadata) -> io::Result<u64> {
    name.size_on_disk_fast(meta)
}

#[cfg(windows)]
pub fn size_on_disk(parent: &Path, name: &Path, meta: &Metadata) -> io::Result<u64> {
    parent.join(name).size_on_disk_fast(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_entry_data() {
        assert!(
            std::mem::size_of::<EntryData>() <= 80,
            "the size of this ({}) should not exceed 80 as it affects overall memory consumption",
            std::mem::size_of::<EntryData>()
        );
    }
}
