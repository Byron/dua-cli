use crate::{Throttle, WalkOptions, WalkRoot, crossdev, inodefilter::InodeFilter};

use crossbeam::channel::Receiver;
#[cfg(not(any(windows, target_os = "macos")))]
use filesize::PathExt;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt, io,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const NONE: u32 = u32::MAX;
const FLAG_OCCUPIED: u32 = 1 << 0;
const FLAG_DIRECTORY: u32 = 1 << 1;
const FLAG_METADATA_IO_ERROR: u32 = 1 << 2;
const FLAG_ENTRY_COUNT: u32 = 1 << 3;

/// Stable index of an entry in a [`Tree`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TreeIndex(NonZeroU32);

impl TreeIndex {
    /// Construct an index from its zero-based slot number.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self::from(index)
    }

    /// Return this index as a `usize` for compact side tables.
    #[must_use]
    pub fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }

    fn from_raw(index: u32) -> Self {
        debug_assert_ne!(index, NONE);
        Self(NonZeroU32::new(index + 1).expect("tree index excludes u32::MAX"))
    }
}

impl Default for TreeIndex {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

impl fmt::Debug for TreeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TreeIndex({})", self.index())
    }
}

impl From<u32> for TreeIndex {
    fn from(index: u32) -> Self {
        assert!(index != NONE, "u32::MAX is reserved for missing tree links");
        Self::from_raw(index)
    }
}

impl From<usize> for TreeIndex {
    fn from(index: usize) -> Self {
        let index = u32::try_from(index).expect("tree index exceeds u32::MAX - 1");
        assert!(index != NONE, "u32::MAX is reserved for missing tree links");
        Self::from_raw(index)
    }
}

/// Metadata stored for a filesystem entry, excluding its arena-backed name.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EntryData {
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
            .field("size", &self.size)
            .field("entry_count", &self.entry_count)
            // Skip mtime
            .field("metadata_io_error", &self.metadata_io_error)
            .finish()
    }
}

/// Borrowed view of an entry in a [`Tree`].
#[derive(Debug, Eq, PartialEq)]
pub struct Entry<'a> {
    /// The entry name relative to its parent.
    pub name: Cow<'a, Path>,
    /// The entry metadata.
    pub data: EntryData,
}

impl std::ops::Deref for Entry<'_> {
    type Target = EntryData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Compact arena record for one filesystem entry.
///
/// Nodes live contiguously in [`Tree::nodes`], while names live once in the
/// shared `Tree::names` byte arena and are referenced by `name_start` and
/// `name_len`. Tree links are 32-bit slot indices, and `flags` packs occupancy,
/// directory, metadata-error, and optional-entry-count state. This keeps a node
/// at 64 bytes on supported 64-bit targets and avoids both an owned `PathBuf`
/// and separate `petgraph` edge storage per entry—the main sources of the
/// traversal's reduced memory footprint. Removed slots reuse `next_sibling` as
/// a free-list link so their node storage can be recycled.
#[derive(Clone)]
struct TreeNode {
    size: u128,
    mtime: SystemTime,
    entry_count: u64,
    name_start: u32,
    name_len: u32,
    parent: u32,
    first_child: u32,
    next_sibling: u32,
    flags: u32,
}

impl TreeNode {
    fn new(name_start: u32, name_len: u32, data: EntryData) -> Self {
        Self {
            size: data.size,
            mtime: data.mtime,
            entry_count: data.entry_count.unwrap_or_default(),
            name_start,
            name_len,
            parent: NONE,
            first_child: NONE,
            next_sibling: NONE,
            flags: FLAG_OCCUPIED
                | (u32::from(data.is_dir) * FLAG_DIRECTORY)
                | (u32::from(data.metadata_io_error) * FLAG_METADATA_IO_ERROR)
                | (u32::from(data.entry_count.is_some()) * FLAG_ENTRY_COUNT),
        }
    }

    fn is_occupied(&self) -> bool {
        self.flags & FLAG_OCCUPIED != 0
    }

    fn data(&self) -> EntryData {
        EntryData {
            size: self.size,
            mtime: self.mtime,
            entry_count: (self.flags & FLAG_ENTRY_COUNT != 0).then_some(self.entry_count),
            metadata_io_error: self.flags & FLAG_METADATA_IO_ERROR != 0,
            is_dir: self.flags & FLAG_DIRECTORY != 0,
        }
    }

    fn set_data(&mut self, data: EntryData) {
        self.size = data.size;
        self.mtime = data.mtime;
        self.entry_count = data.entry_count.unwrap_or_default();
        self.flags = FLAG_OCCUPIED
            | (u32::from(data.is_dir) * FLAG_DIRECTORY)
            | (u32::from(data.metadata_io_error) * FLAG_METADATA_IO_ERROR)
            | (u32::from(data.entry_count.is_some()) * FLAG_ENTRY_COUNT);
    }
}

/// Failure to grow or modify a traversal tree.
#[derive(Debug)]
pub enum TreeError {
    /// A backing allocation failed.
    Allocation(std::collections::TryReserveError),
    /// A name or node index exceeded the tree's `u32` storage limit.
    Capacity,
    /// A supplied node index does not exist.
    InvalidIndex,
    /// The child already belongs to another parent.
    AlreadyAttached,
    /// Attaching the child would create a cycle.
    Cycle,
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation(err) => err.fmt(f),
            Self::Capacity => f.write_str("tree exceeds its u32 storage limit"),
            Self::InvalidIndex => f.write_str("tree index does not exist"),
            Self::AlreadyAttached => f.write_str("tree node already has a parent"),
            Self::Cycle => f.write_str("tree attachment would create a cycle"),
        }
    }
}

impl std::error::Error for TreeError {}

impl From<std::collections::TryReserveError> for TreeError {
    fn from(err: std::collections::TryReserveError) -> Self {
        Self::Allocation(err)
    }
}

/// Arena-backed filesystem tree with stable 32-bit node indices.
#[derive(Clone)]
pub struct Tree {
    /// Contiguous arena slots addressed by [`TreeIndex`]; vacant slots form the free list.
    nodes: Vec<TreeNode>,
    /// Append-only platform-native name bytes referenced by each node's offset and length.
    names: Vec<u8>,
    /// First vacant node slot, chained through `TreeNode::next_sibling`, or [`NONE`].
    free_head: u32,
    /// Number of occupied nodes, which may be smaller than `nodes.len()` after removals.
    len: usize,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree")
            .field("nodes", &self.nodes.len())
            .field("names", &self.names.len())
            .field("len", &self.len)
            .finish()
    }
}

impl Tree {
    /// Create an empty tree.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            names: Vec::new(),
            free_head: NONE,
            len: 0,
        }
    }

    pub(crate) fn try_reserve_exact(
        &mut self,
        additional_nodes: usize,
        additional_name_bytes: usize,
    ) -> Result<(), TreeError> {
        let node_count = self
            .nodes
            .len()
            .checked_add(additional_nodes)
            .ok_or(TreeError::Capacity)?;
        let name_bytes = self
            .names
            .len()
            .checked_add(additional_name_bytes)
            .ok_or(TreeError::Capacity)?;
        if node_count > u32::MAX as usize || name_bytes > u32::MAX as usize {
            return Err(TreeError::Capacity);
        }
        self.nodes.try_reserve_exact(additional_nodes)?;
        self.names.try_reserve_exact(additional_name_bytes)?;
        Ok(())
    }

    /// Add a parentless node.
    ///
    /// # Panics
    ///
    /// Panics if the tree exceeds its storage limits or allocation fails.
    pub fn add_root(&mut self, name: impl AsRef<Path>, data: EntryData) -> TreeIndex {
        self.try_add_root(name, data)
            .expect("tree storage can be allocated")
    }

    /// Add a node that is not yet attached to the tree.
    pub fn add_detached(&mut self, name: impl AsRef<Path>, data: EntryData) -> TreeIndex {
        self.add_root(name, data)
    }

    /// Add a child to `parent`, placing it before previously attached children.
    ///
    /// # Panics
    ///
    /// Panics if `parent` is missing or the tree cannot grow.
    pub fn add_child(
        &mut self,
        parent: TreeIndex,
        name: impl AsRef<Path>,
        data: EntryData,
    ) -> TreeIndex {
        self.try_add_child(parent, name, data)
            .expect("tree storage can be allocated and parent is valid")
    }

    /// Try to add a parentless node.
    pub fn try_add_root(
        &mut self,
        name: impl AsRef<Path>,
        data: EntryData,
    ) -> Result<TreeIndex, TreeError> {
        let (name_start, name_len) = self.try_append_name(name.as_ref())?;
        self.try_allocate(TreeNode::new(name_start, name_len, data))
    }

    /// Try to add a child to `parent`.
    pub fn try_add_child(
        &mut self,
        parent: TreeIndex,
        name: impl AsRef<Path>,
        data: EntryData,
    ) -> Result<TreeIndex, TreeError> {
        if !self.contains(parent) {
            return Err(TreeError::InvalidIndex);
        }
        let child = self.try_add_root(name, data)?;
        self.attach(parent, child)?;
        Ok(child)
    }

    pub(crate) fn try_add_child_native(
        &mut self,
        parent: TreeIndex,
        name: &[u8],
        data: EntryData,
    ) -> Result<TreeIndex, TreeError> {
        if !self.contains(parent) {
            return Err(TreeError::InvalidIndex);
        }
        let (name_start, name_len) = self.try_append_native_name(name)?;
        let child = self.try_allocate(TreeNode::new(name_start, name_len, data))?;
        self.attach(parent, child)?;
        Ok(child)
    }

    /// Attach a parentless node to `parent`, before its existing children.
    pub fn attach(&mut self, parent: TreeIndex, child: TreeIndex) -> Result<(), TreeError> {
        if !self.contains(parent) || !self.contains(child) {
            return Err(TreeError::InvalidIndex);
        }
        if self.nodes[child.index()].parent != NONE {
            return Err(TreeError::AlreadyAttached);
        }
        let mut ancestor = Some(parent);
        while let Some(index) = ancestor {
            if index == child {
                return Err(TreeError::Cycle);
            }
            ancestor = self.parent(index);
        }
        let first_child = self.nodes[parent.index()].first_child;
        self.nodes[child.index()].parent = parent.index() as u32;
        self.nodes[child.index()].next_sibling = first_child;
        self.nodes[parent.index()].first_child = child.index() as u32;
        Ok(())
    }

    /// Return a node's parent, or `None` for a parentless or missing node.
    #[must_use]
    pub fn parent(&self, index: TreeIndex) -> Option<TreeIndex> {
        let node = self.node(index)?;
        (node.parent != NONE).then(|| TreeIndex::from_raw(node.parent))
    }

    /// Iterate a node's children in reverse insertion order.
    #[must_use]
    pub fn children(&self, index: TreeIndex) -> Children<'_> {
        Children {
            nodes: &self.nodes,
            next: self.node(index).map_or(NONE, |node| node.first_child),
        }
    }

    /// Iterate all currently occupied node indices.
    pub fn indices(&self) -> impl Iterator<Item = TreeIndex> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.is_occupied())
            .map(|(index, _)| TreeIndex::from(index))
    }

    /// Return an entry view for `index`.
    #[must_use]
    pub fn entry(&self, index: TreeIndex) -> Option<Entry<'_>> {
        let node = self.node(index)?;
        let name = self.name(index)?;
        Some(Entry {
            name,
            data: node.data(),
        })
    }

    /// Return a copy of an entry's metadata.
    #[must_use]
    pub fn data(&self, index: TreeIndex) -> Option<EntryData> {
        self.node(index).map(TreeNode::data)
    }

    /// Replace an entry's metadata, returning `false` if the index is missing.
    pub fn set_data(&mut self, index: TreeIndex, data: EntryData) -> bool {
        let Some(node) = self.node_mut(index) else {
            return false;
        };
        node.set_data(data);
        true
    }

    /// Mutate an entry's metadata, returning `false` if the index is missing.
    pub fn update(&mut self, index: TreeIndex, edit: impl FnOnce(&mut EntryData)) -> bool {
        let Some(mut data) = self.data(index) else {
            return false;
        };
        edit(&mut data);
        self.set_data(index, data)
    }

    /// Return an entry's arena-backed name.
    #[must_use]
    pub fn name(&self, index: TreeIndex) -> Option<Cow<'_, Path>> {
        let bytes = self.native_name(index)?;
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt as _;
            Some(Cow::Borrowed(Path::new(std::ffi::OsStr::from_bytes(bytes))))
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt as _;
            let wide = bytes
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect::<Vec<_>>();
            Some(Cow::Owned(PathBuf::from(std::ffi::OsString::from_wide(
                &wide,
            ))))
        }
    }

    /// Replace an entry's name. Old arena bytes remain reserved until the tree is dropped.
    pub fn rename(&mut self, index: TreeIndex, name: impl AsRef<Path>) -> Result<(), TreeError> {
        if !self.contains(index) {
            return Err(TreeError::InvalidIndex);
        }
        let (start, len) = self.try_append_name(name.as_ref())?;
        let node = &mut self.nodes[index.index()];
        node.name_start = start;
        node.name_len = len;
        Ok(())
    }

    /// Remove `index` and all descendants, returning the number removed.
    pub fn remove_subtree(&mut self, index: TreeIndex) -> usize {
        if !self.contains(index) {
            return 0;
        }
        self.detach(index);
        let mut pending = vec![index];
        let mut removed = 0;
        while let Some(index) = pending.pop() {
            let mut child = self.nodes[index.index()].first_child;
            while child != NONE {
                pending.push(TreeIndex::from_raw(child));
                child = self.nodes[child as usize].next_sibling;
            }
            let node = &mut self.nodes[index.index()];
            node.flags = 0;
            node.parent = NONE;
            node.first_child = NONE;
            node.next_sibling = self.free_head;
            self.free_head = index.index() as u32;
            self.len -= 1;
            removed += 1;
        }
        removed
    }

    /// Return whether `index` refers to a live node.
    #[must_use]
    pub fn contains(&self, index: TreeIndex) -> bool {
        self.node(index).is_some()
    }

    /// Return the number of live nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return whether this tree has no live nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn native_name(&self, index: TreeIndex) -> Option<&[u8]> {
        let node = self.node(index)?;
        let start = node.name_start as usize;
        Some(&self.names[start..][..node.name_len as usize])
    }

    fn node(&self, index: TreeIndex) -> Option<&TreeNode> {
        self.nodes
            .get(index.index())
            .filter(|node| node.is_occupied())
    }

    fn node_mut(&mut self, index: TreeIndex) -> Option<&mut TreeNode> {
        self.nodes
            .get_mut(index.index())
            .filter(|node| node.is_occupied())
    }

    fn try_allocate(&mut self, node: TreeNode) -> Result<TreeIndex, TreeError> {
        let index = if self.free_head == NONE {
            let index = u32::try_from(self.nodes.len()).map_err(|_| TreeError::Capacity)?;
            if index == NONE {
                return Err(TreeError::Capacity);
            }
            self.nodes.try_reserve(1)?;
            self.nodes.push(node);
            index
        } else {
            let index = self.free_head;
            self.free_head = self.nodes[index as usize].next_sibling;
            self.nodes[index as usize] = node;
            index
        };
        self.len += 1;
        Ok(TreeIndex::from_raw(index))
    }

    fn try_append_name(&mut self, name: &Path) -> Result<(u32, u32), TreeError> {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt as _;
            self.try_append_native_name(name.as_os_str().as_bytes())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt as _;
            let units = name.as_os_str().encode_wide();
            let byte_len = units
                .clone()
                .count()
                .checked_mul(2)
                .ok_or(TreeError::Capacity)?;
            let start = self.names.len();
            let end = start.checked_add(byte_len).ok_or(TreeError::Capacity)?;
            let (start, len) = (
                u32::try_from(start).map_err(|_| TreeError::Capacity)?,
                u32::try_from(byte_len).map_err(|_| TreeError::Capacity)?,
            );
            u32::try_from(end).map_err(|_| TreeError::Capacity)?;
            self.names.try_reserve(byte_len)?;
            self.names.extend(units.flat_map(u16::to_le_bytes));
            Ok((start, len))
        }
    }

    fn try_append_native_name(&mut self, name: &[u8]) -> Result<(u32, u32), TreeError> {
        let start = self.names.len();
        let end = start.checked_add(name.len()).ok_or(TreeError::Capacity)?;
        let (start, len) = (
            u32::try_from(start).map_err(|_| TreeError::Capacity)?,
            u32::try_from(name.len()).map_err(|_| TreeError::Capacity)?,
        );
        u32::try_from(end).map_err(|_| TreeError::Capacity)?;
        self.names.try_reserve(name.len())?;
        self.names.extend_from_slice(name);
        Ok((start, len))
    }

    fn detach(&mut self, index: TreeIndex) {
        let parent = self.nodes[index.index()].parent;
        if parent == NONE {
            return;
        }
        let mut link = self.nodes[parent as usize].first_child;
        let mut previous = NONE;
        while link != NONE {
            if link == index.index() as u32 {
                let next = self.nodes[link as usize].next_sibling;
                if previous == NONE {
                    self.nodes[parent as usize].first_child = next;
                } else {
                    self.nodes[previous as usize].next_sibling = next;
                }
                self.nodes[index.index()].parent = NONE;
                self.nodes[index.index()].next_sibling = NONE;
                return;
            }
            previous = link;
            link = self.nodes[link as usize].next_sibling;
        }
    }
}

/// Iterator over a node's children.
pub struct Children<'a> {
    nodes: &'a [TreeNode],
    next: u32,
}

impl Iterator for Children<'_> {
    type Item = TreeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == NONE {
            return None;
        }
        let index = self.next;
        self.next = self.nodes[index as usize].next_sibling;
        Some(TreeIndex::from_raw(index))
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
        let root_index = tree.add_root("", EntryData::default());
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
    /// 3. The input root's index in the original input list, used to place its tree node in the
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
    /// Retained tree node for each dense directory identifier emitted by the walker.
    nodes_by_directory: Vec<Option<TreeIndex>>,
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
            nodes_by_directory: Vec::new(),
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

    /// Keep tree nodes through `depth`, while still aggregating all sizes, or retain all nodes when
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
            traversal
                .tree
                .update(root, |entry| entry.metadata_io_error = true);
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
        let node = traversal.tree.add_child(
            self.root_idx,
            name,
            EntryData {
                metadata_io_error: true,
                is_dir: true,
                ..EntryData::default()
            },
        );
        traversal.tree.update(self.root_idx, |entry| {
            *entry.entry_count.get_or_insert(0) += 1;
        });
        self.root_nodes[root_idx] = Some(node);
    }

    fn set_directory_node(&mut self, directory_id: usize, node: TreeIndex) {
        if self.nodes_by_directory.len() <= directory_id {
            self.nodes_by_directory.resize(directory_id + 1, None);
        }
        self.nodes_by_directory[directory_id] = Some(node);
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
                let name = if !self.skip_root && walk_depth == 0 && self.use_root_path {
                    root_path.as_path()
                } else {
                    Path::new(&entry.file_name)
                };

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
                                    name,
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
                let preexisting = if self.preexisting_nodes.is_empty() {
                    None
                } else {
                    self.preexisting_nodes.remove(&entry.path())
                };
                if let Some((index, needs_metadata)) = preexisting {
                    if let Some(directory_id) = entry.directory_id {
                        self.set_directory_node(directory_id.index(), index);
                    }
                    if needs_metadata {
                        traversal.tree.update(index, |existing| {
                            existing.size += file_size;
                            *existing.entry_count.get_or_insert(0) += entry_count;
                            if has_mtime {
                                existing.mtime = data.mtime;
                            }
                            existing.metadata_io_error |= data.metadata_io_error;
                            existing.is_dir = data.is_dir;
                        });

                        let mut ancestor = traversal.tree.parent(index);
                        while let Some(ancestor_index) = ancestor {
                            ancestor = traversal.tree.parent(ancestor_index);
                            traversal.tree.update(ancestor_index, |entry| {
                                entry.size += file_size;
                                *entry.entry_count.get_or_insert(0) += entry_count;
                            });
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
                } else {
                    let parent_id = entry
                        .parent_directory_id
                        .expect("non-root entries have a parent directory identifier");
                    if self.skip_root && walk_depth == 1 {
                        self.set_directory_node(parent_id.index(), self.root_idx);
                    }
                    self.nodes_by_directory
                        .get(parent_id.index())
                        .copied()
                        .flatten()
                        .expect("parent entries are emitted before their children")
                };
                let mut retained_node = None;
                if retain_entry {
                    let entry_index = traversal.tree.add_child(parent_index, name, data);
                    retained_node = Some(entry_index);
                    if walk_depth == 0 {
                        self.root_nodes[root_idx] = Some(entry_index);
                    }
                }
                if let Some(directory_id) = entry.directory_id {
                    self.set_directory_node(
                        directory_id.index(),
                        retained_node.unwrap_or(parent_index),
                    );
                }

                let mut ancestor = Some(parent_index);
                while let Some(index) = ancestor {
                    ancestor = traversal.tree.parent(index);
                    traversal.tree.update(index, |entry| {
                        entry.size += file_size;
                        *entry.entry_count.get_or_insert(0) += entry_count;
                    });
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
                let root_size = traversal
                    .tree
                    .data(self.root_idx)
                    .expect("traversal root exists")
                    .size;
                self.nodes_by_directory.clear();
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
                let root_size = traversal.tree.data(traversal.root_index).unwrap().size;
                assert!(
                    root_size >= 7,
                    "root size should include the 7-byte nested file, got {root_size}"
                );
                let nested_size = traversal
                    .tree
                    .indices()
                    .find_map(|index| {
                        (traversal.tree.name(index).as_deref() == Some(Path::new("nested")))
                            .then(|| traversal.tree.data(index).unwrap().size)
                    })
                    .unwrap();
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
            .children(traversal.root_index)
            .collect::<Vec<_>>();
        assert_eq!(roots.len(), 2);
        for root in roots {
            assert_eq!(traversal.tree.children(root).count(), 1);
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

            assert_eq!(traversal.tree.len(), expected_nodes);
            assert!(traversal.tree.data(traversal.root_index).unwrap().size >= 7);
            let root = traversal
                .tree
                .children(traversal.root_index)
                .next()
                .unwrap();
            let last_retained = if depth == 0 {
                root
            } else {
                traversal.tree.children(root).next().unwrap()
            };
            assert!(traversal.tree.data(last_retained).unwrap().size >= 7);
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
            traversal.tree.data(root).unwrap().metadata_io_error,
            "a path-less descendant error is reported on its retained root: {:?}",
            traversal.tree.entry(root).unwrap()
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
            traversal.tree.data(traversal.root_index).unwrap().size
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
            traversal
                .tree
                .data(traversal.root_index)
                .unwrap()
                .entry_count,
            Some(2),
            "the synthetic root counts both input roots"
        );
        assert!(
            traversal.tree.data(roots[0]).unwrap().metadata_io_error,
            "the failed root records its I/O error: {:?}",
            traversal.tree.entry(roots[0]).unwrap()
        );
        assert_eq!(
            traversal.tree.name(roots[0]).unwrap(),
            Path::new("dangling"),
            "the failed root retains its display name"
        );
        assert!(
            roots
                .iter()
                .all(|root| traversal.tree.parent(*root) == Some(traversal.root_index)),
            "all input roots are children of the synthetic root: {roots:?}"
        );
    }

    #[test]
    fn tree_tracks_parents_and_reverse_insertion_order() {
        let mut tree = Tree::new();
        let root = tree.add_root("root", EntryData::default());
        let first = tree.add_child(root, "first", EntryData::default());
        let second = tree.add_child(root, "second", EntryData::default());

        assert_eq!(tree.parent(first), Some(root));
        assert_eq!(tree.parent(second), Some(root));
        assert_eq!(tree.children(root).collect::<Vec<_>>(), [second, first]);
    }

    #[test]
    fn tree_removal_is_stable_and_reuses_slots() {
        let mut tree = Tree::new();
        let root = tree.add_root("root", EntryData::default());
        let kept = tree.add_child(root, "kept", EntryData::default());
        let removed = tree.add_child(root, "removed", EntryData::default());
        let nested = tree.add_child(removed, "nested", EntryData::default());

        assert_eq!(tree.remove_subtree(removed), 2);
        assert!(tree.contains(kept));
        assert_eq!(tree.children(root).collect::<Vec<_>>(), [kept]);

        let reused = tree.add_child(root, "reused", EntryData::default());
        assert!(
            reused == removed || reused == nested,
            "a deleted slot is reused"
        );
        assert_eq!(tree.name(reused).as_deref(), Some(Path::new("reused")));
        assert_eq!(tree.children(root).collect::<Vec<_>>(), [reused, kept]);
    }

    #[test]
    fn detached_nodes_can_be_renamed_and_attached() {
        let mut tree = Tree::new();
        let root = tree.add_root("root", EntryData::default());
        let child = tree.add_detached("before", EntryData::default());

        tree.rename(child, "after").unwrap();
        tree.attach(root, child).unwrap();

        assert_eq!(tree.name(child).as_deref(), Some(Path::new("after")));
        assert_eq!(tree.parent(child), Some(root));
        assert!(matches!(tree.attach(child, root), Err(TreeError::Cycle)));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn tree_nodes_and_optional_indices_are_compact() {
        assert_eq!(std::mem::size_of::<TreeNode>(), 64);
        assert_eq!(std::mem::size_of::<Option<TreeIndex>>(), 4);
    }
}
