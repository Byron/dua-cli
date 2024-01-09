use crate::{crossdev, get_size_or_panic, inodefilter::InodeFilter, Throttle, WalkOptions};

use crossbeam::channel::Receiver;
use filesize::PathExt;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed, Direction};
use std::{
    fmt,
    fs::Metadata,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
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

#[derive(Default, Copy, Clone)]
pub struct EntryInfo {
    pub size: u128,
    pub entries_count: Option<u64>,
}

impl EntryInfo {
    pub fn add_count(&mut self, other: &Self) {
        self.entries_count = match (self.entries_count, other.entries_count) {
            (Some(a), Some(b)) => Some(a + b),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        };
    }
}

pub fn set_entry_info_or_panic(
    tree: &mut Tree,
    node_idx: TreeIndex,
    EntryInfo {
        size,
        entries_count,
    }: EntryInfo,
) {
    let node = tree
        .node_weight_mut(node_idx)
        .expect("node for parent index we just retrieved");
    node.size = size;
    node.entry_count = entries_count;
}

pub fn parent_or_panic(tree: &mut Tree, parent_node_idx: TreeIndex) -> TreeIndex {
    tree.neighbors_directed(parent_node_idx, Direction::Incoming)
        .next()
        .expect("every node in the iteration has a parent")
}

pub fn pop_or_panic(v: &mut Vec<EntryInfo>) -> EntryInfo {
    v.pop().expect("sizes per level to be in sync with graph")
}

pub type TraversalEntry =
    Result<jwalk::DirEntry<((), Option<Result<std::fs::Metadata, jwalk::Error>>)>, jwalk::Error>;

#[allow(clippy::large_enum_variant)]
pub enum TraversalEvent {
    Entry(TraversalEntry, Arc<PathBuf>, u64),
    Finished(u64),
}

/// An in-progress traversal which exposes newly obtained entries
pub struct BackgroundTraversal {
    walk_options: WalkOptions,
    previous_node_idx: TreeIndex,
    parent_node_idx: TreeIndex,
    directory_info_per_depth_level: Vec<EntryInfo>,
    current_directory_at_depth: EntryInfo,
    previous_depth: usize,
    inodes: InodeFilter,
    throttle: Option<Throttle>,
    pub event_rx: Receiver<TraversalEvent>,
}

impl BackgroundTraversal {
    /// Start a background thread to perform the actual tree walk, and dispatch the results
    /// as events to be received on [BackgroundTraversal::event_rx].
    pub fn start(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: Vec<PathBuf>,
    ) -> anyhow::Result<BackgroundTraversal> {
        let (entry_tx, entry_rx) = crossbeam::channel::bounded(100);
        std::thread::Builder::new()
            .name("dua-fs-walk-dispatcher".to_string())
            .spawn({
                let walk_options = walk_options.clone();
                let mut io_errors: u64 = 0;
                move || {
                    for root_path in input.into_iter() {
                        let device_id = match crossdev::init(root_path.as_ref()) {
                            Ok(id) => id,
                            Err(_) => {
                                io_errors += 1;
                                continue;
                            }
                        };

                        let root_path = Arc::new(root_path);
                        for entry in walk_options
                            .iter_from_path(root_path.as_ref(), device_id)
                            .into_iter()
                        {
                            if entry_tx
                                .send(TraversalEvent::Entry(
                                    entry,
                                    Arc::clone(&root_path),
                                    device_id,
                                ))
                                .is_err()
                            {
                                // The channel is closed, this means the user has
                                // requested to quit the app. Abort the walking.
                                return;
                            }
                        }
                    }
                    if entry_tx.send(TraversalEvent::Finished(io_errors)).is_err() {
                        log::error!("Failed to send TraversalEvents::Finished event");
                    }
                }
            })?;

        Ok(Self {
            walk_options: walk_options.clone(),
            previous_node_idx: root_idx,
            parent_node_idx: root_idx,
            directory_info_per_depth_level: Vec::new(),
            current_directory_at_depth: EntryInfo::default(),
            previous_depth: 0,
            inodes: InodeFilter::default(),
            throttle: Some(Throttle::new(Duration::from_millis(250), None)),
            event_rx: entry_rx,
        })
    }

    /// Integrate `event` into traversal `t` so its information is represented by it.
    /// This builds the traversal tree from a directory-walk.
    ///
    /// Returns
    /// * `Some(true)` if the traversal is finished
    /// * `Some(false)` if the caller may update its state after throttling kicked in
    /// * `None` - the event was written into the traversal, but there is nothing else to do
    pub fn integrate_traversal_event(
        &mut self,
        t: &mut Traversal,
        event: TraversalEvent,
    ) -> Option<bool> {
        match event {
            TraversalEvent::Entry(entry, root_path, device_id) => {
                t.entries_traversed += 1;
                let mut data = EntryData::default();
                match entry {
                    Ok(entry) => {
                        data.name = if entry.depth < 1 {
                            (*root_path).clone()
                        } else {
                            entry.file_name.into()
                        };

                        let mut file_size = 0u128;
                        let mut mtime: SystemTime = UNIX_EPOCH;
                        match &entry.client_state {
                            Some(Ok(ref m)) => {
                                if !m.is_dir()
                                    && (self.walk_options.count_hard_links || self.inodes.add(m))
                                    && (self.walk_options.cross_filesystems
                                        || crossdev::is_same_device(device_id, m))
                                {
                                    if self.walk_options.apparent_size {
                                        file_size = m.len() as u128;
                                    } else {
                                        file_size = size_on_disk(&entry.parent_path, &data.name, m)
                                            .unwrap_or_else(|_| {
                                                t.io_errors += 1;
                                                data.metadata_io_error = true;
                                                0
                                            })
                                            as u128;
                                    }
                                } else {
                                    data.entry_count = Some(0);
                                    data.is_dir = true;
                                }

                                match m.modified() {
                                    Ok(modified) => {
                                        mtime = modified;
                                    }
                                    Err(_) => {
                                        t.io_errors += 1;
                                        data.metadata_io_error = true;
                                    }
                                }
                            }
                            Some(Err(_)) => {
                                t.io_errors += 1;
                                data.metadata_io_error = true;
                            }
                            None => {}
                        }

                        match (entry.depth, self.previous_depth) {
                            (n, p) if n > p => {
                                self.directory_info_per_depth_level
                                    .push(self.current_directory_at_depth);
                                self.current_directory_at_depth = EntryInfo {
                                    size: file_size,
                                    entries_count: Some(1),
                                };
                                self.parent_node_idx = self.previous_node_idx;
                            }
                            (n, p) if n < p => {
                                for _ in n..p {
                                    set_entry_info_or_panic(
                                        &mut t.tree,
                                        self.parent_node_idx,
                                        self.current_directory_at_depth,
                                    );
                                    let dir_info =
                                        pop_or_panic(&mut self.directory_info_per_depth_level);

                                    self.current_directory_at_depth.size += dir_info.size;
                                    self.current_directory_at_depth.add_count(&dir_info);

                                    self.parent_node_idx =
                                        parent_or_panic(&mut t.tree, self.parent_node_idx);
                                }
                                self.current_directory_at_depth.size += file_size;
                                *self
                                    .current_directory_at_depth
                                    .entries_count
                                    .get_or_insert(0) += 1;
                                set_entry_info_or_panic(
                                    &mut t.tree,
                                    self.parent_node_idx,
                                    self.current_directory_at_depth,
                                );
                            }
                            _ => {
                                self.current_directory_at_depth.size += file_size;
                                *self
                                    .current_directory_at_depth
                                    .entries_count
                                    .get_or_insert(0) += 1;
                            }
                        };

                        data.mtime = mtime;
                        data.size = file_size;
                        let entry_index = t.tree.add_node(data);

                        t.tree.add_edge(self.parent_node_idx, entry_index, ());
                        self.previous_node_idx = entry_index;
                        self.previous_depth = entry.depth;
                    }
                    Err(_) => {
                        if self.previous_depth == 0 {
                            data.name = (*root_path).clone();
                            let entry_index = t.tree.add_node(data);
                            t.tree.add_edge(self.parent_node_idx, entry_index, ());
                        }

                        t.io_errors += 1
                    }
                }

                if self.throttle.as_ref().map_or(false, |t| t.can_update()) {
                    return Some(false);
                }
            }
            TraversalEvent::Finished(io_errors) => {
                t.io_errors += io_errors;

                self.throttle = None;
                self.directory_info_per_depth_level
                    .push(self.current_directory_at_depth);
                self.current_directory_at_depth = EntryInfo::default();
                for _ in 0..self.previous_depth {
                    let dir_info = pop_or_panic(&mut self.directory_info_per_depth_level);
                    self.current_directory_at_depth.size += dir_info.size;
                    self.current_directory_at_depth.add_count(&dir_info);

                    set_entry_info_or_panic(
                        &mut t.tree,
                        self.parent_node_idx,
                        self.current_directory_at_depth,
                    );
                    self.parent_node_idx = parent_or_panic(&mut t.tree, self.parent_node_idx);
                }
                let root_size = t.recompute_root_size();
                set_entry_info_or_panic(
                    &mut t.tree,
                    t.root_index,
                    EntryInfo {
                        size: root_size,
                        entries_count: (t.entries_traversed > 0).then_some(t.entries_traversed),
                    },
                );
                t.total_bytes = Some(root_size);
                t.elapsed = Some(t.start.elapsed());

                return Some(true);
            }
        }
        None
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
