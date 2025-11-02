use crate::{Throttle, WalkOptions, crossdev, get_size_or_panic, inodefilter::InodeFilter};

use crossbeam::channel::Receiver;
use filesize::PathExt;
use petgraph::{Directed, Direction, graph::NodeIndex, stable_graph::StableGraph};
use std::time::Instant;
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
    /// plus the  size of the directory entry itself
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
    /// The time at which the instance was created, typically the start of the traversal.
    pub start_time: Instant,
    /// The time it cost to compute the traversal, when done.
    pub cost: Option<Duration>,
}

impl Default for Traversal {
    fn default() -> Self {
        Self::new()
    }
}

impl Traversal {
    pub fn new() -> Self {
        let mut tree = Tree::new();
        let root_index = tree.add_node(EntryData::default());
        Self {
            tree,
            root_index,
            start_time: Instant::now(),
            cost: None,
        }
    }

    pub fn recompute_node_size(&self, node_index: TreeIndex) -> u128 {
        self.tree
            .neighbors_directed(node_index, Direction::Outgoing)
            .map(|idx| get_size_or_panic(&self.tree, idx))
            .sum()
    }

    pub fn is_costly(&self) -> bool {
        self.cost.is_none_or(|d| d.as_secs_f32() > 10.0)
    }
}

#[derive(Clone, Copy)]
pub struct TraversalStats {
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

impl Default for TraversalStats {
    fn default() -> Self {
        Self {
            entries_traversed: 0,
            start: std::time::Instant::now(),
            elapsed: None,
            io_errors: 0,
            total_bytes: None,
        }
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
    node_own_size: u128,
    EntryInfo {
        size,
        entries_count,
    }: EntryInfo,
) {
    let node = tree
        .node_weight_mut(node_idx)
        .expect("node for parent index we just retrieved");
    node.size = size + node_own_size;
    node.entry_count = entries_count.map(|n| n + 1);
}

pub fn parent_or_panic(tree: &mut Tree, parent_node_idx: TreeIndex) -> TreeIndex {
    tree.neighbors_directed(parent_node_idx, Direction::Incoming)
        .next()
        .expect("every node in the iteration has a parent")
}

pub fn pop_or_panic<T>(v: &mut Vec<T>) -> T {
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
    pub root_idx: TreeIndex,
    pub stats: TraversalStats,
    previous_node_idx: TreeIndex,
    parent_node_idx: TreeIndex,
    directory_info_per_depth_level: Vec<EntryInfo>,
    current_directory_at_depth: EntryInfo,
    parent_node_size: u128,
    parent_node_size_per_depth_level: Vec<u128>,
    previous_node_size: u128,
    previous_depth: usize,
    inodes: InodeFilter,
    throttle: Option<Throttle>,
    skip_root: bool,
    use_root_path: bool,
    pub event_rx: Receiver<TraversalEvent>,
}

impl BackgroundTraversal {
    /// Start a background thread to perform the actual tree walk, and dispatch the results
    /// as events to be received on [BackgroundTraversal::event_rx].
    pub fn start(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: Vec<PathBuf>,
        skip_root: bool,
        use_root_path: bool,
    ) -> anyhow::Result<BackgroundTraversal> {
        let (entry_tx, entry_rx) = crossbeam::channel::bounded(100);
        std::thread::Builder::new()
            .name("dua-fs-walk-dispatcher".to_string())
            .spawn({
                let walk_options = walk_options.clone();
                let mut io_errors: u64 = 0;
                move || {
                    for root_path in input.into_iter() {
                        log::info!("Walking {root_path:?}");
                        let device_id = match crossdev::init(root_path.as_ref()) {
                            Ok(id) => id,
                            Err(_) => {
                                io_errors += 1;
                                continue;
                            }
                        };

                        let root_path = Arc::new(root_path);
                        for entry in walk_options
                            .iter_from_path(root_path.as_ref(), device_id, skip_root)
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
            root_idx,
            stats: TraversalStats::default(),
            previous_node_idx: root_idx,
            parent_node_idx: root_idx,
            previous_node_size: 0,
            parent_node_size: 0,
            parent_node_size_per_depth_level: Vec::new(),
            directory_info_per_depth_level: Vec::new(),
            current_directory_at_depth: EntryInfo::default(),
            previous_depth: 0,
            inodes: InodeFilter::default(),
            throttle: Some(Throttle::new(Duration::from_millis(250), None)),
            skip_root,
            use_root_path,
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
        traversal: &mut Traversal,
        event: TraversalEvent,
    ) -> Option<bool> {
        match event {
            TraversalEvent::Entry(entry, root_path, device_id) => {
                self.stats.entries_traversed += 1;
                let mut data = EntryData::default();
                match entry {
                    Ok(mut entry) => {
                        if self.skip_root {
                            entry.depth -= 1;
                            data.name = entry.file_name.into()
                        } else {
                            data.name = if entry.depth < 1 && self.use_root_path {
                                (*root_path).clone()
                            } else {
                                entry.file_name.into()
                            }
                        }

                        let mut file_size = 0u128;
                        let mut mtime: SystemTime = UNIX_EPOCH;
                        let mut file_count = 0u64;
                        match &entry.client_state {
                            Some(Ok(m)) => {
                                if self.walk_options.count_hard_links
                                    || self.inodes.add(m)
                                        && (self.walk_options.cross_filesystems
                                            || crossdev::is_same_device(device_id, m))
                                {
                                    file_count = 1;
                                    if self.walk_options.apparent_size {
                                        file_size = m.len() as u128;
                                    } else {
                                        file_size = size_on_disk(&entry.parent_path, &data.name, m)
                                            .unwrap_or_else(|_| {
                                                self.stats.io_errors += 1;
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
                                        self.stats.io_errors += 1;
                                        data.metadata_io_error = true;
                                    }
                                }
                            }
                            Some(Err(_)) => {
                                self.stats.io_errors += 1;
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
                                    entries_count: Some(file_count),
                                };

                                self.parent_node_size_per_depth_level
                                    .push(self.previous_node_size);

                                self.parent_node_idx = self.previous_node_idx;
                                self.parent_node_size = self.previous_node_size;
                            }
                            (n, p) if n < p => {
                                for _ in n..p {
                                    set_entry_info_or_panic(
                                        &mut traversal.tree,
                                        self.parent_node_idx,
                                        self.parent_node_size,
                                        self.current_directory_at_depth,
                                    );
                                    let dir_info =
                                        pop_or_panic(&mut self.directory_info_per_depth_level);

                                    self.current_directory_at_depth.size += dir_info.size;
                                    self.current_directory_at_depth.add_count(&dir_info);

                                    self.parent_node_idx =
                                        parent_or_panic(&mut traversal.tree, self.parent_node_idx);
                                    self.parent_node_size =
                                        pop_or_panic(&mut self.parent_node_size_per_depth_level);
                                }
                                self.current_directory_at_depth.size += file_size;
                                *self
                                    .current_directory_at_depth
                                    .entries_count
                                    .get_or_insert(0) += file_count;
                                set_entry_info_or_panic(
                                    &mut traversal.tree,
                                    self.parent_node_idx,
                                    self.parent_node_size,
                                    self.current_directory_at_depth,
                                );
                            }
                            _ => {
                                self.current_directory_at_depth.size += file_size;
                                *self
                                    .current_directory_at_depth
                                    .entries_count
                                    .get_or_insert(0) += file_count;
                            }
                        };

                        data.mtime = mtime;
                        data.size = file_size;
                        let entry_index = traversal.tree.add_node(data);

                        traversal
                            .tree
                            .add_edge(self.parent_node_idx, entry_index, ());
                        self.previous_node_idx = entry_index;
                        self.previous_node_size = file_size;
                        self.previous_depth = entry.depth;
                    }
                    Err(_) => {
                        if self.previous_depth == 0 {
                            data.name.clone_from(&(*root_path));
                            let entry_index = traversal.tree.add_node(data);
                            traversal
                                .tree
                                .add_edge(self.parent_node_idx, entry_index, ());
                        }

                        self.stats.io_errors += 1
                    }
                }

                if self.throttle.as_ref().is_some_and(|t| t.can_update()) {
                    return Some(false);
                }
            }
            TraversalEvent::Finished(io_errors) => {
                self.stats.io_errors += io_errors;

                self.throttle = None;
                self.directory_info_per_depth_level
                    .push(self.current_directory_at_depth);
                self.current_directory_at_depth = EntryInfo::default();
                for _ in 0..self.previous_depth {
                    let dir_info = pop_or_panic(&mut self.directory_info_per_depth_level);
                    self.current_directory_at_depth.size += dir_info.size;
                    self.current_directory_at_depth.add_count(&dir_info);

                    set_entry_info_or_panic(
                        &mut traversal.tree,
                        self.parent_node_idx,
                        self.parent_node_size,
                        self.current_directory_at_depth,
                    );
                    self.parent_node_idx =
                        parent_or_panic(&mut traversal.tree, self.parent_node_idx);
                }
                let root_size = traversal.recompute_node_size(self.root_idx);
                set_entry_info_or_panic(
                    &mut traversal.tree,
                    self.root_idx,
                    root_size,
                    EntryInfo {
                        size: root_size,
                        entries_count: (self.stats.entries_traversed > 0)
                            .then_some(self.stats.entries_traversed),
                    },
                );
                self.stats.total_bytes = Some(root_size);
                self.stats.elapsed = Some(self.stats.start.elapsed());

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
