use crate::{Throttle, WalkOptions, WalkRoot, crossdev, inodefilter::InodeFilter};

use crossbeam::channel::Receiver;
#[cfg(not(any(windows, target_os = "macos")))]
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
pub struct TraversalEntry(pub(crate) crate::walk::Entry);

/// Events emitted by a background filesystem traversal.
pub enum TraversalEvent {
    /// A discovered entry and its traversal context:
    ///
    /// 0. The discovered entry, or the I/O error encountered while reading it.
    /// 1. The path of the input root being traversed.
    /// 2. The input root's device ID.
    /// 3. The input root's index in the original input list, used to place its graph node in the
    ///    per-root side table so callers can recover input order, including failed roots.
    Entry(io::Result<TraversalEntry>, Arc<PathBuf>, u64, usize),
    /// A root that could not be initialized, with its input index.
    RootError(Arc<PathBuf>, usize),
    /// Traversal completed.
    Finished,
}

/// An in-progress traversal which exposes newly obtained entries
pub struct BackgroundTraversal {
    walk_options: WalkOptions,
    /// Tree node index that acts as root for this traversal integration.
    pub root_idx: TreeIndex,
    /// Running traversal statistics.
    pub stats: TraversalStats,
    /// Root nodes in input order; populated as root traversal events are integrated.
    pub(crate) root_nodes: Vec<Option<TreeIndex>>,
    /// Nodes keyed by root allocation identity and path so overlapping roots build separate trees.
    nodes_by_path: HashMap<(usize, PathBuf), TreeIndex>,
    inodes: InodeFilter,
    throttle: Option<Throttle>,
    skip_root: bool,
    use_root_path: bool,
    retained_depth: Option<usize>,
    preexisting_nodes: HashMap<PathBuf, (TreeIndex, bool)>,
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
        Self::start_inner(
            root_idx,
            walk_options,
            input,
            pattern_roots,
            skip_root,
            use_root_path,
            HashMap::new(),
        )
    }

    /// Start an incremental traversal that preserves subtrees listed in `preexisting_nodes`.
    ///
    /// This is used when extending a traversal to a parent directory: rescanning an existing
    /// subtree would waste work and duplicate its nodes and totals. Each tuple is
    /// `(path, node, needs_metadata)`. `needs_metadata` is true when the node was a synthetic
    /// traversal root that represented the directory's contents but not the directory entry
    /// itself; once that root becomes a child, its own metadata must be integrated.
    pub fn start_incremental(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: Vec<PathBuf>,
        pattern_roots: Option<&[PathBuf]>,
        skip_root: bool,
        use_root_path: bool,
        preexisting_nodes: Vec<(PathBuf, TreeIndex, bool)>,
    ) -> anyhow::Result<BackgroundTraversal> {
        let mut walk_options = walk_options.clone();
        walk_options.ignore_dirs.extend(
            preexisting_nodes
                .iter()
                .filter_map(|(path, _, _)| gix::path::realpath(path).ok()),
        );
        Self::start_inner(
            root_idx,
            &walk_options,
            input,
            pattern_roots,
            skip_root,
            use_root_path,
            preexisting_nodes
                .into_iter()
                .map(|(path, index, needs_metadata)| (path, (index, needs_metadata)))
                .collect(),
        )
    }

    fn start_inner(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: Vec<PathBuf>,
        pattern_roots: Option<&[PathBuf]>,
        skip_root: bool,
        use_root_path: bool,
        preexisting_nodes: HashMap<PathBuf, (TreeIndex, bool)>,
    ) -> anyhow::Result<BackgroundTraversal> {
        let num_roots = input.len();
        let (entry_tx, entry_rx) = crossbeam::channel::bounded(100);
        let pattern_roots = pattern_roots.map(<[PathBuf]>::to_owned);
        std::thread::Builder::new()
            .name("dua-fs-walk-dispatcher".to_string())
            .spawn({
                let walk_options = walk_options.clone();
                move || {
                    let (mut root_paths, mut root_indices, mut device_ids, mut walk_roots) = (
                        Vec::with_capacity(input.len()),
                        Vec::with_capacity(input.len()),
                        Vec::with_capacity(input.len()),
                        Vec::with_capacity(input.len()),
                    );
                    for (root_idx, root_path) in input.into_iter().enumerate() {
                        log::info!("Walking {}", root_path.display());
                        let device_id = if walk_options.cross_filesystems {
                            0
                        } else {
                            let Ok(device_id) = crossdev::init(&root_path) else {
                                if entry_tx
                                    .send(TraversalEvent::RootError(Arc::new(root_path), root_idx))
                                    .is_err()
                                {
                                    return;
                                }
                                continue;
                            };
                            device_id
                        };
                        let pattern_root = pattern_roots.as_deref().map(|pattern_roots| {
                            pattern_roots
                                .iter()
                                .filter(|candidate| root_path.starts_with(candidate))
                                .max_by_key(|candidate| candidate.components().count())
                                .cloned()
                                .unwrap_or_else(|| root_path.clone())
                        });
                        walk_roots.push(WalkRoot {
                            index: walk_roots.len(),
                            pattern_root,
                            path: root_path.clone(),
                            #[cfg(any(windows, target_os = "macos"))]
                            entry: None,
                            device_id,
                        });
                        root_indices.push(root_idx);
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
                                root_indices[root],
                            ))
                            .is_err()
                        {
                            // The channel is closed, this means the user has
                            // requested to quit the app. Abort the walking.
                            return;
                        }
                    }
                    if entry_tx.send(TraversalEvent::Finished).is_err() {
                        log::error!("Failed to send TraversalEvents::Finished event");
                    }
                }
            })?;

        Ok(Self {
            walk_options: walk_options.clone(),
            root_idx,
            stats: TraversalStats::default(),
            root_nodes: vec![None; num_roots],
            nodes_by_path: HashMap::new(),
            inodes: InodeFilter::default(),
            throttle: Some(Throttle::new(Duration::from_millis(250), None)),
            skip_root,
            use_root_path,
            retained_depth: None,
            preexisting_nodes,
            event_rx: entry_rx,
        })
    }

    /// Return the top-level nodes in the same order as the traversal inputs once all roots exist.
    #[must_use]
    pub fn root_nodes(&self) -> Option<Vec<TreeIndex>> {
        self.root_nodes.iter().copied().collect()
    }

    /// Keep graph nodes through `depth`, while still aggregating all sizes, or retain all nodes when
    /// it is `None`. For example, 0 retains roots only, 1 also retains their immediate children,
    /// and 2 also retains grandchildren.
    pub(crate) fn retain_depth(mut self, depth: Option<usize>) -> Self {
        self.retained_depth = depth;
        self
    }

    fn record_error_on_root(
        &mut self,
        traversal: &mut Traversal,
        root_idx: usize,
        root_path: &Path,
    ) {
        if self.skip_root {
            return;
        }
        // Entry errors carry no descendant path, so report them on the corresponding root.
        if let Some(root) = self.root_nodes[root_idx] {
            traversal.tree[root].metadata_io_error = true;
            return;
        }
        let name = if self.use_root_path {
            root_path.to_owned()
        } else {
            root_path
                .file_name()
                .unwrap_or(root_path.as_os_str())
                .into()
        };
        let node = traversal.tree.add_node(EntryData {
            name,
            metadata_io_error: true,
            is_dir: true,
            ..EntryData::default()
        });
        traversal.tree.add_edge(self.root_idx, node, ());
        *traversal.tree[self.root_idx].entry_count.get_or_insert(0) += 1;
        self.root_nodes[root_idx] = Some(node);
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
            TraversalEvent::Entry(entry, root_path, device_id, root_idx) => {
                let root = Arc::as_ptr(&root_path) as usize;
                self.stats.entries_traversed += 1;
                let mut data = EntryData::default();
                let Ok(TraversalEntry(entry)) = entry else {
                    self.stats.io_errors += 1;
                    self.record_error_on_root(traversal, root_idx, &root_path);
                    return self
                        .throttle
                        .as_ref()
                        .is_some_and(|t| t.can_update())
                        .then_some(false);
                };
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
                let mut has_mtime = false;
                data.is_dir = entry.file_type.is_dir();
                if let Ok(m) = &entry.metadata {
                    if self.walk_options.count_hard_links
                        || self.inodes.add(&entry, m)
                            && (self.walk_options.cross_filesystems
                                || crossdev::is_same_device(device_id, m))
                    {
                        if self.walk_options.apparent_size {
                            file_size = u128::from(m.len());
                        } else {
                            file_size = u128::from(
                                size_on_disk(
                                    &entry.parent_path,
                                    &data.name,
                                    m,
                                    data.is_dir,
                                    &self.walk_options,
                                    &mut self.inodes,
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
                        has_mtime = true;
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
                if let Some((index, needs_metadata)) = self.preexisting_nodes.remove(&entry.path())
                {
                    if needs_metadata {
                        let existing = &mut traversal.tree[index];
                        existing.size += file_size;
                        *existing.entry_count.get_or_insert(0) += entry_count;
                        if has_mtime {
                            existing.mtime = data.mtime;
                        }
                        existing.metadata_io_error |= data.metadata_io_error;
                        existing.is_dir = data.is_dir;

                        let mut ancestor = traversal
                            .tree
                            .neighbors_directed(index, Direction::Incoming)
                            .next();
                        while let Some(ancestor_index) = ancestor {
                            ancestor = traversal
                                .tree
                                .neighbors_directed(ancestor_index, Direction::Incoming)
                                .next();
                            let entry = &mut traversal.tree[ancestor_index];
                            entry.size += file_size;
                            *entry.entry_count.get_or_insert(0) += entry_count;
                        }
                    }
                    return self
                        .throttle
                        .as_ref()
                        .is_some_and(|t| t.can_update())
                        .then_some(false);
                }
                let retain_entry = self.retained_depth.is_none_or(|depth| walk_depth <= depth);

                let parent_index = if walk_depth == 0 {
                    self.root_idx
                } else if self.retained_depth == Some(0) {
                    if self.skip_root {
                        self.root_idx
                    } else {
                        self.root_nodes[root_idx]
                            .expect("root entries are emitted before their children")
                    }
                } else {
                    if self.skip_root {
                        self.nodes_by_path
                            .entry((root, (*root_path).clone()))
                            .or_insert(self.root_idx);
                    }
                    let mut parent_path = entry.parent_path.to_path_buf();
                    loop {
                        if let Some(index) = self.nodes_by_path.get(&(root, parent_path.clone())) {
                            break *index;
                        }
                        if !parent_path.pop() {
                            assert!(
                                !retain_entry,
                                "parent entries are emitted before their children"
                            );
                            break self.root_idx;
                        }
                    }
                };
                if retain_entry {
                    let entry_index = traversal.tree.add_node(data);
                    traversal.tree.add_edge(parent_index, entry_index, ());
                    if walk_depth == 0 {
                        self.root_nodes[root_idx] = Some(entry_index);
                    }
                    if traversal.tree[entry_index].is_dir {
                        self.nodes_by_path.insert((root, entry.path()), entry_index);
                    }
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

                if self.throttle.as_ref().is_some_and(|t| t.can_update()) {
                    return Some(false);
                }
            }
            TraversalEvent::RootError(root_path, root_idx) => {
                self.stats.io_errors += 1;
                self.record_error_on_root(traversal, root_idx, &root_path);
            }
            TraversalEvent::Finished => {
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

#[cfg(not(any(windows, target_os = "macos")))]
/// Return disk usage for `name` on Unix-like platforms.
fn size_on_disk(
    _parent: &Path,
    name: &Path,
    meta: &crate::walk::Metadata,
    _is_dir: bool,
    _options: &WalkOptions,
    _inodes: &mut InodeFilter,
) -> io::Result<u64> {
    name.size_on_disk_fast(meta)
}

#[cfg(target_os = "macos")]
/// Return disk usage from metadata already collected by the macOS filesystem walker.
#[allow(clippy::unnecessary_wraps)]
fn size_on_disk(
    _parent: &Path,
    _name: &Path,
    meta: &crate::walk::Metadata,
    _is_dir: bool,
    options: &WalkOptions,
    inodes: &mut InodeFilter,
) -> io::Result<u64> {
    Ok(if options.metadata_options.apfs_clone_metadata {
        inodes.allocated_size(meta)
    } else {
        meta.allocated_size()
    })
}

#[cfg(windows)]
/// Return disk usage for `name` on Windows platforms.
#[allow(clippy::unnecessary_wraps)]
fn size_on_disk(
    _parent: &Path,
    _name: &Path,
    meta: &crate::walk::Metadata,
    is_dir: bool,
    _options: &WalkOptions,
    _inodes: &mut InodeFilter,
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
                metadata_options: crate::TraversalOptions::default(),
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
                TraversalEvent::Entry(Ok(TraversalEntry(entry)), _, _, _)
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
                metadata_options: crate::TraversalOptions::default(),
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

    #[test]
    fn retained_depth_rolls_deeper_sizes_into_the_last_kept_node() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("one/two")).unwrap();
        std::fs::write(dir.path().join("one/two/file"), b"content").unwrap();
        for (depth, expected_nodes) in [(0, 2), (1, 3)] {
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
                    metadata_options: crate::TraversalOptions::default(),
                },
                vec![dir.path().to_owned()],
                None,
                false,
                true,
            )
            .unwrap()
            .retain_depth(Some(depth));

            while !background
                .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
                .unwrap_or(false)
            {}

            assert_eq!(traversal.tree.node_count(), expected_nodes);
            assert!(traversal.tree[traversal.root_index].size >= 7);
            let root = traversal
                .tree
                .neighbors_directed(traversal.root_index, Direction::Outgoing)
                .next()
                .unwrap();
            let last_retained = if depth == 0 {
                root
            } else {
                traversal
                    .tree
                    .neighbors_directed(root, Direction::Outgoing)
                    .next()
                    .unwrap()
            };
            assert!(traversal.tree[last_retained].size >= 7);
        }
    }

    #[test]
    fn descendant_entry_errors_mark_the_retained_root() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().to_owned();
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
                metadata_options: crate::TraversalOptions::default(),
            },
            vec![root_path.clone()],
            None,
            false,
            true,
        )
        .unwrap()
        .retain_depth(Some(0));

        while background.root_nodes[0].is_none() {
            let event = background.event_rx.recv().unwrap();
            background.integrate_traversal_event(&mut traversal, event);
        }
        let root = background.root_nodes[0].unwrap();
        background.integrate_traversal_event(
            &mut traversal,
            TraversalEvent::Entry(
                Err(io::Error::other("unreadable descendant")),
                Arc::new(root_path),
                0,
                0,
            ),
        );

        assert_eq!(background.stats.io_errors, 1);
        assert!(
            traversal.tree[root].metadata_io_error,
            "a path-less descendant error is reported on its retained root: {:?}",
            traversal.tree[root]
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn interactive_traversal_deduplicates_apfs_clones() {
        use std::os::unix::fs::MetadataExt as _;

        fn total(path: &Path, deduplicate: bool) -> u128 {
            let mut traversal = Traversal::new();
            let mut background = BackgroundTraversal::start(
                traversal.root_index,
                &WalkOptions {
                    threads: 2,
                    count_hard_links: false,
                    apparent_size: false,
                    cross_filesystems: true,
                    ignore_dirs: std::collections::BTreeSet::default(),
                    ignore_patterns: None,
                    metadata_options: crate::TraversalOptions {
                        apfs_clone_metadata: deduplicate,
                    },
                },
                vec![path.to_owned()],
                None,
                false,
                false,
            )
            .unwrap();

            while !background
                .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
                .unwrap_or(false)
            {}
            traversal.tree[traversal.root_index].size
        }

        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("original");
        let clone = directory.path().join("clone");
        std::fs::write(&original, vec![1; 8192]).unwrap();
        // std::fs::copy uses fclonefileat(2) first on Apple platforms, producing an APFS clone.
        std::fs::copy(&original, clone).unwrap();
        let data_fork_size = u128::from(std::fs::metadata(original).unwrap().blocks()) * 512;

        assert_eq!(
            total(directory.path(), false) - total(directory.path(), true),
            data_fork_size
        );
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
                metadata_options: crate::TraversalOptions::default(),
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
        let roots = background
            .root_nodes
            .iter()
            .copied()
            .collect::<Option<Vec<_>>>()
            .unwrap();
        assert_eq!(roots.len(), 2, "one node per input root: {roots:?}");
        assert_eq!(
            traversal.tree[traversal.root_index].entry_count,
            Some(2),
            "the synthetic root counts both input roots"
        );
        assert!(
            traversal.tree[roots[0]].metadata_io_error,
            "the failed root records its I/O error: {:?}",
            traversal.tree[roots[0]]
        );
        assert_eq!(
            traversal.tree[roots[0]].name,
            Path::new("dangling"),
            "the failed root retains its display name"
        );
        assert!(
            roots.iter().all(|root| {
                traversal
                    .tree
                    .find_edge(traversal.root_index, *root)
                    .is_some()
            }),
            "all input roots are children of the synthetic root: {roots:?}"
        );
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
