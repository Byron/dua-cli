use crate::{Throttle, WalkOptions, WalkRoot, crossdev, inodefilter::InodeFilter};

use crossbeam::channel::Receiver;
#[cfg(not(windows))]
use filesize::PathExt;
use petgraph::{Directed, Direction, graph::NodeIndex, stable_graph::StableGraph};
use std::time::Instant;
use std::{
    collections::HashMap,
    fmt, io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Node index type used by the traversal tree graph.
pub type TreeIndex = NodeIndex;
/// Graph type used to represent traversed filesystem entries.
pub type Tree = StableGraph<EntryData, (), Directed>;

/// Data stored for each filesystem entry in the traversal tree.
#[derive(Eq, PartialEq, Clone)]
pub struct EntryData {
    /// The entry name relative to its parent.
    pub name: PathBuf,
    /// The entry's size in bytes. If it's a directory, the size is the aggregated file size of all children
    /// plus the  size of the directory entry itself
    pub size: u128,
    /// Last modification time if available.
    pub mtime: SystemTime,
    /// Recursive entry count for directories, or `None` for files.
    pub entry_count: Option<u64>,
    /// If set, the item meta-data could not be obtained
    pub metadata_io_error: bool,
    /// `true` if the entry is a directory.
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
    /// Create a new empty traversal with a synthetic root node.
    #[must_use]
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

    /// Return `true` if this traversal is considered expensive to recompute.
    #[must_use]
    pub fn is_costly(&self) -> bool {
        self.cost.is_none_or(|d| d.as_secs_f32() > 10.0)
    }
}

/// Runtime statistics gathered while traversal is running.
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

/// A filesystem entry waiting to be integrated into a traversal.
pub struct TraversalEntry(crate::walk::RootEntry);

/// Events emitted by a background filesystem traversal.
pub enum TraversalEvent {
    /// A discovered entry and its traversal context.
    Entry(io::Result<TraversalEntry>, Arc<PathBuf>, u64),
    /// Traversal completed with the number of root-initialization I/O errors.
    Finished(u64),
}

/// An in-progress traversal which exposes newly obtained entries
pub struct BackgroundTraversal {
    walk_options: WalkOptions,
    /// Tree node index that acts as root for this traversal integration.
    pub root_idx: TreeIndex,
    /// Running traversal statistics.
    pub stats: TraversalStats,
    /// Nodes keyed by root allocation identity and path so overlapping roots build separate trees.
    nodes_by_path: HashMap<(usize, PathBuf), TreeIndex>,
    inodes: InodeFilter,
    throttle: Option<Throttle>,
    skip_root: bool,
    use_root_path: bool,
    /// Receiver used to obtain traversal events from the worker thread.
    pub event_rx: Receiver<TraversalEvent>,
}

impl BackgroundTraversal {
    /// Start a background thread to perform the actual tree walk, and dispatch the results
    /// as events to be received on [`BackgroundTraversal::event_rx`].
    pub fn start(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: Vec<PathBuf>,
        pattern_roots: Option<&[PathBuf]>,
        skip_root: bool,
        use_root_path: bool,
    ) -> anyhow::Result<BackgroundTraversal> {
        let (entry_tx, entry_rx) = crossbeam::channel::bounded(100);
        let pattern_roots = pattern_roots.map(<[PathBuf]>::to_owned);
        std::thread::Builder::new()
            .name("dua-fs-walk-dispatcher".to_string())
            .spawn({
                let walk_options = walk_options.clone();
                move || {
                    let mut io_errors = 0;
                    let (mut root_paths, mut device_ids, mut walk_roots) = (
                        Vec::with_capacity(input.len()),
                        Vec::with_capacity(input.len()),
                        Vec::with_capacity(input.len()),
                    );
                    for root_path in input {
                        log::info!("Walking {}", root_path.display());
                        let pattern_root = pattern_roots.as_deref().map(|pattern_roots| {
                            pattern_roots
                                .iter()
                                .filter(|candidate| root_path.starts_with(candidate))
                                .max_by_key(|candidate| candidate.components().count())
                                .cloned()
                                .unwrap_or_else(|| root_path.clone())
                        });
                        let device_id = if walk_options.cross_filesystems {
                            0
                        } else {
                            let Ok(device_id) = crossdev::init(&root_path) else {
                                // Skip roots that can't be accessed entirely.
                                io_errors += 1;
                                continue;
                            };
                            device_id
                        };
                        walk_roots.push(WalkRoot {
                            index: walk_roots.len(),
                            pattern_root,
                            path: root_path.clone(),
                            device_id,
                        });
                        device_ids.push(device_id);
                        root_paths.push(Arc::new(root_path));
                    }

                    for (root, event) in walk_options.iter_from_paths(
                        walk_roots,
                        skip_root,
                        crate::walk::Order::ParentFirst,
                    ) {
                        let crate::walk::RootEvent::Entry(entry) = event else {
                            continue;
                        };
                        if entry_tx
                            .send(TraversalEvent::Entry(
                                entry.map(TraversalEntry),
                                Arc::clone(&root_paths[root]),
                                device_ids[root],
                            ))
                            .is_err()
                        {
                            // The channel is closed, this means the user has
                            // requested to quit the app. Abort the walking.
                            return;
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
            nodes_by_path: HashMap::new(),
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
    ///
    /// # Panics
    ///
    /// Panics if a child entry arrives before its parent, violating the parent-first traversal
    /// invariant.
    #[expect(
        clippy::too_many_lines,
        reason = "event integration keeps tree updates atomic"
    )]
    pub fn integrate_traversal_event(
        &mut self,
        traversal: &mut Traversal,
        event: TraversalEvent,
    ) -> Option<bool> {
        match event {
            TraversalEvent::Entry(entry, root_path, device_id) => {
                let root = Arc::as_ptr(&root_path) as usize;
                self.stats.entries_traversed += 1;
                let mut data = EntryData::default();
                match entry {
                    Ok(TraversalEntry(entry)) => {
                        let walk_depth = entry.depth;
                        if self.skip_root {
                            data.name = entry.file_name.clone().into();
                        } else {
                            data.name = if walk_depth < 1 && self.use_root_path {
                                (*root_path).clone()
                            } else {
                                entry.file_name.clone().into()
                            }
                        }

                        let mut file_size = 0u128;
                        let mut mtime: SystemTime = UNIX_EPOCH;
                        data.is_dir = entry.file_type.is_dir();
                        if let Ok(m) = &entry.metadata {
                            if self.walk_options.count_hard_links
                                || self.inodes.add(m)
                                    && (self.walk_options.cross_filesystems
                                        || crossdev::is_same_device(device_id, m))
                            {
                                if self.walk_options.apparent_size {
                                    file_size = u128::from(m.len());
                                } else {
                                    file_size =
                                        u128::from(
                                            size_on_disk(
                                                &entry.parent_path,
                                                &data.name,
                                                m,
                                                data.is_dir,
                                            )
                                            .unwrap_or_else(|_| {
                                                self.stats.io_errors += 1;
                                                data.metadata_io_error = true;
                                                0
                                            }),
                                        );
                                }
                            } else {
                                data.entry_count = Some(0);
                            }

                            if let Ok(modified) = m.modified() {
                                mtime = modified;
                            } else {
                                self.stats.io_errors += 1;
                                data.metadata_io_error = true;
                            }
                        } else {
                            self.stats.io_errors += 1;
                            data.metadata_io_error = true;
                        }

                        data.mtime = mtime;
                        data.size = file_size;
                        if data.is_dir {
                            data.entry_count = Some(1);
                        }
                        let entry_count = u64::from(data.is_dir || data.entry_count != Some(0));

                        let parent_index = if walk_depth == 0 {
                            self.root_idx
                        } else {
                            if self.skip_root {
                                self.nodes_by_path
                                    .entry((root, (*root_path).clone()))
                                    .or_insert(self.root_idx);
                            }
                            *self
                                .nodes_by_path
                                .get(&(root, entry.parent_path.to_path_buf()))
                                .expect("parent entries are emitted before their children")
                        };
                        let entry_index = traversal.tree.add_node(data);
                        traversal.tree.add_edge(parent_index, entry_index, ());
                        if traversal.tree[entry_index].is_dir {
                            self.nodes_by_path.insert((root, entry.path()), entry_index);
                        }

                        let mut ancestor = Some(parent_index);
                        while let Some(index) = ancestor {
                            ancestor = traversal
                                .tree
                                .neighbors_directed(index, Direction::Incoming)
                                .next();
                            let entry = &mut traversal.tree[index];
                            entry.size += file_size;
                            *entry.entry_count.get_or_insert(0) += entry_count;
                        }
                    }
                    Err(_) => self.stats.io_errors += 1,
                }

                if self.throttle.as_ref().is_some_and(|t| t.can_update()) {
                    return Some(false);
                }
            }
            TraversalEvent::Finished(io_errors) => {
                self.stats.io_errors += io_errors;
                self.throttle = None;
                let root_size = traversal.tree[self.root_idx].size;
                self.nodes_by_path = HashMap::new();
                self.stats.total_bytes = Some(root_size);
                self.stats.elapsed = Some(self.stats.start.elapsed());

                return Some(true);
            }
        }
        None
    }
}

#[cfg(not(windows))]
/// Return disk usage for `name` on Unix-like platforms.
fn size_on_disk(
    _parent: &Path,
    name: &Path,
    meta: &crate::walk::RootMetadata,
    _is_dir: bool,
) -> io::Result<u64> {
    name.size_on_disk_fast(meta)
}

#[cfg(windows)]
/// Return disk usage for `name` on Windows platforms.
#[allow(clippy::unnecessary_wraps)]
fn size_on_disk(
    _parent: &Path,
    _name: &Path,
    meta: &crate::walk::RootMetadata,
    is_dir: bool,
) -> io::Result<u64> {
    Ok(if is_dir { 0 } else { meta.allocated_size() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestor_sizes_update_before_traversal_finishes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        std::fs::write(dir.path().join("nested/file"), b"content").unwrap();

        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start(
            traversal.root_index,
            &WalkOptions {
                threads: 2,
                count_hard_links: true,
                apparent_size: true,
                cross_filesystems: true,
                ignore_dirs: std::collections::BTreeSet::default(),
                ignore_patterns: None,
            },
            vec![dir.path().to_owned()],
            None,
            false,
            false,
        )
        .unwrap();

        loop {
            let event = background.event_rx.recv().unwrap();
            let is_file = matches!(
                &event,
                TraversalEvent::Entry(Ok(TraversalEntry(entry)), _, _)
                    if entry.file_name == "file"
            );
            background.integrate_traversal_event(&mut traversal, event);
            if is_file {
                let root_size = traversal.tree[traversal.root_index].size;
                assert!(
                    root_size >= 7,
                    "root size should include the 7-byte nested file, got {root_size}"
                );
                let nested_size = traversal
                    .tree
                    .node_weights()
                    .find(|entry| entry.name == Path::new("nested"))
                    .unwrap()
                    .size;
                assert!(
                    nested_size >= 7,
                    "nested directory size should include its 7-byte file, got {nested_size}"
                );
                break;
            }
        }
    }

    #[test]
    fn duplicate_roots_keep_their_own_children() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file"), b"content").unwrap();
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start(
            traversal.root_index,
            &WalkOptions {
                threads: 1,
                count_hard_links: true,
                apparent_size: true,
                cross_filesystems: true,
                ignore_dirs: std::collections::BTreeSet::default(),
                ignore_patterns: None,
            },
            vec![dir.path().to_owned(), dir.path().to_owned()],
            None,
            false,
            false,
        )
        .unwrap();

        while !background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            .unwrap_or(false)
        {}

        let roots = traversal
            .tree
            .neighbors_directed(traversal.root_index, Direction::Outgoing)
            .collect::<Vec<_>>();
        assert_eq!(roots.len(), 2);
        for root in roots {
            assert_eq!(
                traversal
                    .tree
                    .neighbors_directed(root, Direction::Outgoing)
                    .count(),
                1
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn root_device_error_is_reported() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("dangling");
        let valid = dir.path().join("valid");
        symlink(dir.path().join("missing"), &root).unwrap();
        std::fs::write(&valid, b"content").unwrap();
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start(
            traversal.root_index,
            &WalkOptions {
                threads: 1,
                count_hard_links: true,
                apparent_size: true,
                cross_filesystems: false,
                ignore_dirs: std::collections::BTreeSet::default(),
                ignore_patterns: None,
            },
            vec![root.clone(), valid.clone()],
            None,
            false,
            false,
        )
        .unwrap();

        while !background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            .unwrap_or(false)
        {}

        assert_eq!(background.stats.io_errors, 1);
    }

    #[test]
    fn size_of_entry_data() {
        assert!(
            std::mem::size_of::<EntryData>() <= 80,
            "the size of this ({}) should not exceed 80 as it affects overall memory consumption",
            std::mem::size_of::<EntryData>()
        );
    }
}
