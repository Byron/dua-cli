use crate::{Throttle, WalkOptions, WalkRoot, crossdev, inodefilter::InodeFilter};

use crossbeam::channel::Receiver;
#[cfg(not(any(windows, target_os = "macos")))]
use filesize::PathExt;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt, io,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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

    /// Return the filesystem path formed by the names of `index` and its ancestors.
    ///
    /// # Panics
    /// Panics if `index` is missing.
    #[must_use]
    pub fn path_of(&self, index: TreeIndex) -> PathBuf {
        let names: Vec<_> = std::iter::successors(Some(index), |index| self.parent(*index))
            .map(|index| self.name(index).expect("node exists"))
            .collect();
        names
            .iter()
            .rev()
            .map(|name| name.as_os_str())
            .filter(|name| !name.is_empty())
            .collect()
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
        self.remove_detached_subtrees(vec![index])
    }

    /// Remove selected direct children of `parent` and their descendants in one sibling pass.
    ///
    /// Missing indices and indices that are not direct children of `parent` are ignored.
    /// Returns the total number of removed nodes; parent sizes and counts are unchanged.
    pub fn remove_children(&mut self, parent: TreeIndex, children: &HashSet<TreeIndex>) -> usize {
        let Some(node) = self.node(parent) else {
            return 0;
        };
        let mut current = node.first_child;
        let mut previous = NONE;
        let mut pending = Vec::new();
        while current != NONE {
            let index = TreeIndex::from_raw(current);
            let next = self.nodes[current as usize].next_sibling;
            if children.contains(&index) {
                if previous == NONE {
                    self.nodes[parent.index()].first_child = next;
                } else {
                    self.nodes[previous as usize].next_sibling = next;
                }
                pending.push(index);
            } else {
                previous = current;
            }
            current = next;
        }
        self.remove_detached_subtrees(pending)
    }

    fn remove_detached_subtrees(&mut self, mut pending: Vec<TreeIndex>) -> usize {
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
    /// Original discovery root for each published cleanup candidate, keyed by absolute path.
    pub clean_search_roots: HashMap<PathBuf, PathBuf>,
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
            clean_search_roots: HashMap::new(),
            start_time: Instant::now(),
            cost: None,
        }
    }

    /// Return `true` if this traversal is considered expensive to recompute.
    #[must_use]
    pub fn is_costly(&self) -> bool {
        self.cost.is_none_or(|d| d.as_secs_f32() > 10.0)
    }

    /// Remove selected children and their cleanup provenance in one sibling pass.
    pub fn remove_children(&mut self, parent: TreeIndex, children: &HashSet<TreeIndex>) -> usize {
        if parent == self.root_index && !self.clean_search_roots.is_empty() {
            for &index in children {
                if self.tree.parent(index) == Some(parent)
                    && let Some(name) = self.tree.name(index)
                {
                    self.clean_search_roots.remove(name.as_ref());
                }
            }
        }
        self.tree.remove_children(parent, children)
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
    /// Lightweight discovery progress, as deltas since the previous update.
    DiscoveryProgress {
        /// Number of directory entries inspected.
        entries: u64,
        /// Number of I/O errors encountered.
        io_errors: u64,
    },
    /// Begin staging a cleanup candidate, with its stream index and original discovery root.
    CandidateStarted(usize, PathBuf),
    /// Finish the indexed candidate; only accepted candidates become visible.
    CandidateFinished(usize, bool),
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
    clean_stages: HashMap<usize, CleanStage>,
    _cancelled: Option<CancelOnDrop>,
}

struct CleanStage {
    staging_root: TreeIndex,
    search_root: PathBuf,
    /// Only entries affecting shared inode/clone accounting retain their metadata until acceptance.
    deferred_inodes: Vec<(TreeIndex, crate::walk::Entry, u128)>,
    valid: bool,
}

struct CancelOnDrop(Arc<AtomicBool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

enum CleanInput {
    Discover {
        paths: Vec<PathBuf>,
        depth: Option<usize>,
        pattern_root: Option<PathBuf>,
    },
    Refresh(Vec<(PathBuf, PathBuf)>),
}

impl CleanInput {
    fn discover(
        self,
        options: WalkOptions,
        mut send: impl FnMut(crate::clean::DiscoveryEvent) -> bool,
    ) {
        match self {
            Self::Discover {
                paths,
                depth,
                pattern_root,
            } => {
                crate::clean::discover(paths, options, depth, pattern_root, send);
            }
            Self::Refresh(candidates) => {
                let mut keep_going = true;
                for (path, search_root) in candidates {
                    if !keep_going {
                        break;
                    }
                    crate::clean::discover(
                        vec![path],
                        options.clone(),
                        Some(0),
                        Some(search_root),
                        |event| {
                            keep_going = send(event);
                            keep_going
                        },
                    );
                }
            }
        }
    }
}

impl BackgroundTraversal {
    /// Discover cleanup candidates and publish each root once its detailed scan is complete.
    pub fn start_clean(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: Vec<PathBuf>,
        depth: Option<usize>,
        pattern_root: Option<&Path>,
    ) -> anyhow::Result<Self> {
        Self::start_clean_inner(
            root_idx,
            walk_options,
            CleanInput::Discover {
                paths: input,
                depth,
                pattern_root: pattern_root.map(Path::to_path_buf),
            },
        )
    }

    /// Revalidate published candidate paths with each candidate's original discovery root.
    pub fn refresh_clean_candidates(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        candidates: Vec<(PathBuf, PathBuf)>,
    ) -> anyhow::Result<Self> {
        Self::start_clean_inner(root_idx, walk_options, CleanInput::Refresh(candidates))
    }

    fn start_clean_inner(
        root_idx: TreeIndex,
        walk_options: &WalkOptions,
        input: CleanInput,
    ) -> anyhow::Result<Self> {
        let (entry_tx, event_rx) = crossbeam::channel::bounded(100);
        let cancelled = Arc::new(AtomicBool::new(false));
        std::thread::Builder::new()
            .name("dua-clean-dispatcher".into())
            .spawn({
                let walk_options = walk_options.clone();
                let cancelled = Arc::clone(&cancelled);
                move || {
                    use crate::clean::DiscoveryEvent;
                    if walk_options.threads <= 1 {
                        let mut index = 0;
                        input.discover(walk_options.clone(), |event| {
                            if cancelled.load(Ordering::Relaxed) {
                                return false;
                            }
                            match event {
                                DiscoveryEvent::Candidate { path, search_root } => {
                                    let keep_going = walk_clean_candidate(
                                        &walk_options,
                                        path,
                                        search_root,
                                        index,
                                        &entry_tx,
                                        &cancelled,
                                    );
                                    index += 1;
                                    keep_going
                                }
                                DiscoveryEvent::Progress { entries, io_errors } => entry_tx
                                    .send(TraversalEvent::DiscoveryProgress { entries, io_errors })
                                    .is_ok(),
                            }
                        });
                    } else {
                        walk_clean_candidates(input, &walk_options, &entry_tx, &cancelled);
                    }
                    let _ = entry_tx.send(TraversalEvent::Finished);
                }
            })?;
        Ok(Self {
            walk_options: walk_options.clone(),
            root_idx,
            stats: TraversalStats::default(),
            root_nodes: Vec::new(),
            nodes_by_directory: Vec::new(),
            inodes: InodeFilter::default(),
            throttle: Some(Throttle::new(Duration::from_millis(250), None)),
            skip_root: false,
            use_root_path: true,
            retained_depth: None,
            preexisting_nodes: HashMap::new(),
            event_rx,
            clean_stages: HashMap::new(),
            _cancelled: Some(CancelOnDrop(cancelled)),
        })
    }

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
            clean_stages: HashMap::new(),
            _cancelled: None,
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
            self.integration_root(root_idx),
            name,
            EntryData {
                metadata_io_error: true,
                is_dir: true,
                ..EntryData::default()
            },
        );
        traversal
            .tree
            .update(self.integration_root(root_idx), |entry| {
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

    fn integration_root(&self, index: usize) -> TreeIndex {
        self.clean_stages
            .get(&index)
            .map_or(self.root_idx, |stage| stage.staging_root)
    }

    fn commit_clean_inodes(
        &mut self,
        traversal: &mut Traversal,
        entries: Vec<(TreeIndex, crate::walk::Entry, u128)>,
    ) {
        for (node, entry, original_size) in entries {
            let metadata = entry
                .metadata
                .as_ref()
                .and_then(|m| m.as_ref().ok())
                .expect("accepted metadata");
            let counted = self.walk_options.count_hard_links || self.inodes.add(&entry, metadata);
            let size = if counted { original_size } else { 0 };
            #[cfg(target_os = "macos")]
            let size = if counted
                && !self.walk_options.apparent_size
                && self.walk_options.metadata_options.apfs_clone_metadata
            {
                u128::from(self.inodes.allocated_size(metadata))
            } else {
                size
            };
            let removed_bytes = original_size - size;
            let removed_entry = u64::from(!counted && !entry.file_type.is_dir());
            traversal.tree.update(node, |data| {
                data.size -= removed_bytes;
                if removed_entry != 0 {
                    data.entry_count = Some(0);
                }
            });
            let mut ancestor = traversal.tree.parent(node);
            while let Some(index) = ancestor {
                traversal.tree.update(index, |data| {
                    data.size -= removed_bytes;
                    *data.entry_count.get_or_insert(0) -= removed_entry;
                });
                ancestor = traversal.tree.parent(index);
            }
        }
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
            TraversalEvent::DiscoveryProgress { entries, io_errors } => {
                self.stats.entries_traversed += entries;
                self.stats.io_errors += io_errors;
                return self
                    .throttle
                    .as_ref()
                    .is_some_and(|t| t.can_update())
                    .then_some(false);
            }
            TraversalEvent::CandidateStarted(candidate_index, search_root) => {
                if self.clean_stages.is_empty() {
                    self.nodes_by_directory.clear();
                }
                self.root_nodes
                    .resize(self.root_nodes.len().max(candidate_index + 1), None);
                let staging_root = traversal.tree.add_root("", EntryData::default());
                assert!(
                    self.clean_stages
                        .insert(
                            candidate_index,
                            CleanStage {
                                staging_root,
                                search_root,
                                deferred_inodes: Vec::new(),
                                valid: true,
                            }
                        )
                        .is_none()
                );
            }
            TraversalEvent::CandidateFinished(candidate_index, accepted) => {
                let stage = self
                    .clean_stages
                    .remove(&candidate_index)
                    .expect("candidate has started");
                let candidate = self.root_nodes[candidate_index];
                if let Some(candidate) = candidate.filter(|_| accepted && stage.valid) {
                    self.commit_clean_inodes(traversal, stage.deferred_inodes);
                    traversal.clean_search_roots.insert(
                        traversal
                            .tree
                            .name(candidate)
                            .expect("staged root exists")
                            .into_owned(),
                        stage.search_root,
                    );
                    traversal.tree.detach(candidate);
                    traversal
                        .tree
                        .attach(traversal.root_index, candidate)
                        .expect("staged root is detached");
                    let data = traversal.tree.data(candidate).expect("staged root exists");
                    traversal.tree.update(traversal.root_index, |root| {
                        root.size += data.size;
                        *root.entry_count.get_or_insert(0) += data.entry_count.unwrap_or(0);
                    });
                } else {
                    self.root_nodes[candidate_index] = None;
                }
                traversal.tree.remove_subtree(stage.staging_root);
                return Some(false);
            }
            TraversalEvent::Entry(entry, root_path, device_id, root_idx) => {
                self.stats.entries_traversed += 1;
                let is_clean = self.clean_stages.contains_key(&root_idx);
                let mut data = EntryData::default();
                let Ok(TraversalEntry(entry)) = entry else {
                    if let Some(stage) = self.clean_stages.get_mut(&root_idx) {
                        stage.valid = false;
                    }
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
                if let Some(Ok(m)) = &entry.metadata {
                    if self.walk_options.count_hard_links
                        || (is_clean || self.inodes.add(&entry, m))
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
                                    (!is_clean).then_some(&mut self.inodes),
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
                if data.metadata_io_error
                    && let Some(stage) = self.clean_stages.get_mut(&root_idx)
                {
                    stage.valid = false;
                }
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
                    self.integration_root(root_idx)
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

                if is_clean && InodeFilter::needs_tracking(&entry, &self.walk_options) {
                    self.clean_stages
                        .get_mut(&root_idx)
                        .expect("candidate has started")
                        .deferred_inodes
                        .push((
                            retained_node.expect("clean retains all entries"),
                            entry,
                            file_size,
                        ));
                }
                if self.throttle.as_ref().is_some_and(|t| t.can_update()) {
                    return Some(false);
                }
            }
            TraversalEvent::RootError(root_path, root_idx) => {
                if let Some(stage) = self.clean_stages.get_mut(&root_idx) {
                    stage.valid = false;
                }
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

struct CleanCandidate {
    path: Arc<PathBuf>,
    search_root: PathBuf,
    device: u64,
    accepted: Arc<AtomicBool>,
}

impl CleanCandidate {
    fn new(options: &WalkOptions, path: PathBuf, search_root: PathBuf) -> io::Result<Self> {
        let device = if options.cross_filesystems {
            0
        } else {
            crossdev::init(&search_root)?
        };
        Ok(Self {
            path: Arc::new(path),
            search_root,
            device,
            accepted: Arc::new(AtomicBool::new(true)),
        })
    }

    fn forward_entry(
        &self,
        options: &WalkOptions,
        cwd: &Path,
        index: usize,
        entry: io::Result<crate::walk::Entry>,
        tx: &crossbeam::channel::Sender<TraversalEvent>,
    ) -> bool {
        if self.accepted.load(Ordering::Relaxed) {
            if let Ok(entry) = &entry {
                let entry_path = entry.path();
                let excluded = options.ignore_dirs.contains(entry_path.as_path())
                    || options.ignore_patterns.as_ref().is_some_and(|patterns| {
                        crate::pattern_relative_path(&entry_path, cwd, &self.search_root)
                            .is_some_and(|relative| {
                                patterns.is_excluded(relative, entry.file_type.is_dir())
                            })
                    });
                let boundary = !options.cross_filesystems
                    && entry
                        .metadata
                        .as_ref()
                        .and_then(|m| m.as_ref().ok())
                        .is_some_and(|m| !crossdev::is_same_device(self.device, m));
                let git_marker = crate::clean::is_git_dir_name(&entry.file_name)
                    || (entry
                        .file_name
                        .as_encoded_bytes()
                        .eq_ignore_ascii_case(b"HEAD")
                        && gix::discover::is_git(entry.parent_path.as_ref()).is_ok());
                if git_marker
                    || excluded
                    || boundary
                    || (entry.depth == 0 && !entry.file_type.is_dir())
                {
                    log::debug!(
                        "Skipping cleanup candidate {} because of {}",
                        self.path.display(),
                        entry_path.display()
                    );
                    self.accepted.store(false, Ordering::Relaxed);
                }
            }
            if self.accepted.load(Ordering::Relaxed) {
                let failed = entry
                    .as_ref()
                    .map_or(true, |entry| !matches!(&entry.metadata, Some(Ok(_))));
                if failed {
                    self.accepted.store(false, Ordering::Relaxed);
                }
                return tx
                    .send(TraversalEvent::Entry(
                        entry.map(TraversalEntry),
                        Arc::clone(&self.path),
                        self.device,
                        index,
                    ))
                    .is_ok();
            }
        }
        // Rejected candidates can still have jobs in flight. Count them while those jobs drain.
        tx.send(TraversalEvent::DiscoveryProgress {
            entries: 1,
            io_errors: 0,
        })
        .is_ok()
    }
}

fn walk_clean_candidates(
    input: CleanInput,
    options: &WalkOptions,
    tx: &crossbeam::channel::Sender<TraversalEvent>,
    cancelled: &Arc<AtomicBool>,
) {
    std::thread::scope(|scope| {
        // One discovery reader and a shared sizing pool, within the existing I/O thread budget.
        let (mut roots, walk) = crate::walk::stream_roots(
            options.threads - 1,
            crate::walk::Order::ParentFirst,
            options.metadata_options,
        );
        let (candidate_tx, candidate_rx) = crossbeam::channel::bounded(32);
        // Bound active candidate staging, not just the queue waiting to reach the walker.
        let (slot_tx, slot_rx) = crossbeam::channel::bounded(32);
        for _ in 0..32 {
            slot_tx.send(()).expect("slots fit");
        }
        let discovery = std::thread::Builder::new()
            .name("dua-clean-discovery".into())
            .spawn_scoped(scope, move || {
                let mut index = 0;
                input.discover(options.clone(), |event| {
                    if cancelled.load(Ordering::Relaxed) {
                        return false;
                    }
                    match event {
                        crate::clean::DiscoveryEvent::Progress { entries, io_errors } => tx
                            .send(TraversalEvent::DiscoveryProgress { entries, io_errors })
                            .is_ok(),
                        crate::clean::DiscoveryEvent::Candidate { path, search_root } => {
                            let candidate =
                                match CleanCandidate::new(options, path.clone(), search_root) {
                                    Ok(candidate) => candidate,
                                    Err(err) => {
                                        log::debug!(
                                            "Could not inspect cleanup root {}: {err}",
                                            path.display()
                                        );
                                        return tx
                                            .send(TraversalEvent::DiscoveryProgress {
                                                entries: 0,
                                                io_errors: 1,
                                            })
                                            .is_ok();
                                    }
                                };
                            if slot_rx.recv().is_err() {
                                return false;
                            }
                            let device = candidate.device;
                            let accepted = Arc::clone(&candidate.accepted);
                            let cancelled = Arc::clone(cancelled);
                            let cross_filesystems = options.cross_filesystems;
                            if candidate_tx.send((index, candidate)).is_err() {
                                return false;
                            }
                            let submitted = roots
                                .add_root(index, path, move |entry| {
                                    accepted.load(Ordering::Relaxed)
                                        && !cancelled.load(Ordering::Relaxed)
                                        && (cross_filesystems
                                            || entry
                                                .metadata
                                                .as_ref()
                                                .and_then(|m| m.as_ref().ok())
                                                .is_none_or(|m| {
                                                    crossdev::is_same_device(device, m)
                                                }))
                                })
                                .is_ok();
                            index += 1;
                            submitted
                        }
                    }
                });
            });
        if let Err(err) = discovery {
            log::error!("Could not start cleanup discovery: {err}");
            let _ = tx.send(TraversalEvent::DiscoveryProgress {
                entries: 0,
                io_errors: 1,
            });
            return;
        }
        let cwd = std::env::current_dir().unwrap_or_default();
        let mut candidates = HashMap::new();
        for (index, event) in walk {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            // Discovery queues context before submitting a root, so it is available even when
            // another root's metadata finishes first.
            while !candidates.contains_key(&index) {
                let (candidate_index, candidate) =
                    candidate_rx.recv().expect("submitted root has context");
                if tx
                    .send(TraversalEvent::CandidateStarted(
                        candidate_index,
                        candidate.search_root.clone(),
                    ))
                    .is_err()
                {
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
                candidates.insert(candidate_index, candidate);
            }
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            let keep_going = match event {
                crate::walk::RootEvent::Entry(entry) => {
                    candidates[&index].forward_entry(options, &cwd, index, entry, tx)
                }
                crate::walk::RootEvent::Finished => {
                    let candidate = candidates.remove(&index).expect("root has started");
                    let sent = tx
                        .send(TraversalEvent::CandidateFinished(
                            index,
                            candidate.accepted.load(Ordering::Relaxed),
                        ))
                        .is_ok();
                    slot_tx.send(()).ok();
                    sent
                }
            };
            if !keep_going {
                cancelled.store(true, Ordering::Relaxed);
                break;
            }
        }
        // Unblock discovery before the scope joins it, including cancellation while at capacity.
        drop(slot_tx);
        drop(candidate_rx);
    });
}

fn walk_clean_candidate(
    options: &WalkOptions,
    path: PathBuf,
    search_root: PathBuf,
    index: usize,
    tx: &crossbeam::channel::Sender<TraversalEvent>,
    cancelled: &AtomicBool,
) -> bool {
    if cancelled.load(Ordering::Relaxed) {
        return false;
    }
    let candidate = match CleanCandidate::new(options, path, search_root) {
        Ok(candidate) => candidate,
        Err(err) => {
            log::debug!("Could not inspect cleanup root: {err}");
            return tx
                .send(TraversalEvent::DiscoveryProgress {
                    entries: 0,
                    io_errors: 1,
                })
                .is_ok();
        }
    };
    if tx
        .send(TraversalEvent::CandidateStarted(
            index,
            candidate.search_root.clone(),
        ))
        .is_err()
    {
        return false;
    }
    // Inspect exclusions as well: deleting their containing candidate would delete them too.
    let mut unfiltered = options.clone();
    unfiltered.ignore_dirs.clear();
    unfiltered.ignore_patterns = None;
    let roots = vec![WalkRoot {
        index: 0,
        path: candidate.path.as_ref().clone(),
        #[cfg(any(windows, target_os = "macos"))]
        entry: None,
        pattern_root: None,
        device_id: candidate.device,
    }];
    let cwd = std::env::current_dir().unwrap_or_default();
    for (_, event) in unfiltered.iter_from_paths(roots, false, crate::walk::Order::ParentFirst) {
        if cancelled.load(Ordering::Relaxed) {
            return false;
        }
        if let crate::walk::RootEvent::Entry(entry) = event {
            if !candidate.forward_entry(options, &cwd, index, entry, tx) {
                return false;
            }
            if !candidate.accepted.load(Ordering::Relaxed) {
                break;
            }
        }
    }
    tx.send(TraversalEvent::CandidateFinished(
        index,
        candidate.accepted.load(Ordering::Relaxed),
    ))
    .is_ok()
}

#[cfg(not(any(windows, target_os = "macos")))]
/// Return disk usage for `name` on Unix-like platforms.
fn size_on_disk(
    _parent: &Path,
    name: &Path,
    meta: &crate::walk::Metadata,
    _is_dir: bool,
    _options: &WalkOptions,
    _inodes: Option<&mut InodeFilter>,
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
    inodes: Option<&mut InodeFilter>,
) -> io::Result<u64> {
    Ok(inodes
        .filter(|_| options.metadata_options.apfs_clone_metadata)
        .map_or_else(
            || meta.allocated_size(),
            |inodes| inodes.allocated_size(meta),
        ))
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
    _inodes: Option<&mut InodeFilter>,
) -> io::Result<u64> {
    Ok(if is_dir { 0 } else { meta.allocated_size() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_walk_options() -> WalkOptions {
        WalkOptions {
            threads: 2,
            count_hard_links: false,
            apparent_size: true,
            cross_filesystems: true,
            ignore_dirs: std::collections::BTreeSet::new(),
            ignore_patterns: None,
            metadata_options: crate::TraversalOptions::default(),
        }
    }

    #[test]
    fn clean_candidates_can_finish_out_of_order_without_sharing_unaccepted_inodes() {
        let directory = tempfile::tempdir().unwrap();
        let paths = ["slow", "fast", "last"].map(|name| directory.path().join(name));
        for path in &paths {
            std::fs::create_dir(path).unwrap();
        }
        std::fs::write(paths[0].join("payload"), b"content").unwrap();
        for path in &paths[1..] {
            std::fs::hard_link(paths[0].join("payload"), path.join("payload")).unwrap();
        }
        let mut traversal = Traversal::new();
        let options = clean_walk_options();
        let mut background = BackgroundTraversal::start_clean(
            traversal.root_index,
            &options,
            vec![directory.path().to_owned()],
            Some(0),
            None,
        )
        .unwrap();
        while background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            != Some(true)
        {}

        for index in 0..2 {
            background.integrate_traversal_event(
                &mut traversal,
                TraversalEvent::CandidateStarted(index, directory.path().to_owned()),
            );
        }
        for (index, path) in paths[..2].iter().enumerate() {
            for entry in crate::walk::walk(
                path,
                1,
                crate::walk::Order::ParentFirst,
                options.metadata_options,
                |_| true,
            ) {
                background.integrate_traversal_event(
                    &mut traversal,
                    TraversalEvent::Entry(
                        entry.map(TraversalEntry),
                        Arc::new(path.clone()),
                        0,
                        index,
                    ),
                );
            }
        }
        assert_eq!(traversal.tree.children(traversal.root_index).count(), 0);
        background
            .integrate_traversal_event(&mut traversal, TraversalEvent::CandidateFinished(1, true));
        let fast = background.root_nodes[1].unwrap();
        let payload = traversal.tree.children(fast).next().unwrap();
        assert_eq!(traversal.tree.data(payload).unwrap().size, 7);
        assert_eq!(traversal.tree.children(traversal.root_index).count(), 1);
        background
            .integrate_traversal_event(&mut traversal, TraversalEvent::CandidateFinished(0, false));
        background.integrate_traversal_event(
            &mut traversal,
            TraversalEvent::CandidateStarted(2, directory.path().to_owned()),
        );
        for entry in crate::walk::walk(
            &paths[2],
            1,
            crate::walk::Order::ParentFirst,
            options.metadata_options,
            |_| true,
        ) {
            background.integrate_traversal_event(
                &mut traversal,
                TraversalEvent::Entry(entry.map(TraversalEntry), Arc::new(paths[2].clone()), 0, 2),
            );
        }
        background
            .integrate_traversal_event(&mut traversal, TraversalEvent::CandidateFinished(2, true));
        let last = background.root_nodes[2].unwrap();
        let payload = traversal.tree.children(last).next().unwrap();
        assert_eq!(
            traversal.tree.data(payload).unwrap().size,
            0,
            "rejecting the slow root must not erase the fast root's accepted accounting"
        );
        assert!(background.clean_stages.is_empty());
        assert!(background.root_nodes[0].is_none());
    }

    #[test]
    fn clean_streams_more_candidates_than_its_capacity_with_any_thread_budget() {
        let directory = tempfile::tempdir().unwrap();
        for index in 0..40 {
            let candidate = directory
                .path()
                .join(format!("project-{index}/node_modules"));
            std::fs::create_dir_all(candidate.join("child")).unwrap();
            std::fs::write(candidate.join("child/payload"), b"content").unwrap();
            if index % 10 == 0 {
                std::fs::create_dir(candidate.join("child/.git")).unwrap();
            }
        }
        for threads in [1, 2, 4] {
            let mut traversal = Traversal::new();
            let mut options = clean_walk_options();
            options.threads = threads;
            let mut background = BackgroundTraversal::start_clean(
                traversal.root_index,
                &options,
                vec![directory.path().to_owned()],
                None,
                None,
            )
            .unwrap();
            loop {
                let event = background
                    .event_rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("scan makes progress");
                if background.integrate_traversal_event(&mut traversal, event) == Some(true) {
                    break;
                }
            }
            assert_eq!(traversal.tree.children(traversal.root_index).count(), 36);
            assert_eq!(background.stats.io_errors, 0);
            assert!(background.clean_stages.is_empty());
            for root in traversal.tree.children(traversal.root_index) {
                let child = traversal.tree.children(root).next().unwrap();
                let payload = traversal.tree.children(child).next().unwrap();
                assert_eq!(traversal.tree.data(payload).unwrap().size, 7);
            }
        }
    }

    #[test]
    fn clean_roots_are_published_only_after_sizing_and_vetting() {
        let directory = tempfile::tempdir().unwrap();
        let good = directory.path().join("good/node_modules");
        let bad = directory.path().join("bad/node_modules");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::create_dir_all(bad.join("nested/.git")).unwrap();
        std::fs::write(good.join("payload"), b"content").unwrap();
        std::fs::write(bad.join("payload"), b"private").unwrap();
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start_clean(
            traversal.root_index,
            &clean_walk_options(),
            vec![directory.path().to_owned()],
            None,
            None,
        )
        .unwrap();
        let mut published = 0;
        loop {
            let event = background.event_rx.recv().unwrap();
            if matches!(&event, TraversalEvent::CandidateFinished(_, true)) {
                published += 1;
            }
            let finished = background.integrate_traversal_event(&mut traversal, event);
            assert_eq!(
                traversal.tree.children(traversal.root_index).count(),
                published,
                "in-progress and rejected candidates must remain hidden"
            );
            if finished == Some(true) {
                break;
            }
        }
        let roots = traversal
            .tree
            .children(traversal.root_index)
            .collect::<Vec<_>>();
        assert_eq!(roots.len(), 1);
        assert_eq!(
            traversal.tree.name(roots[0]).unwrap(),
            good.canonicalize().unwrap()
        );
        assert_eq!(traversal.tree.children(roots[0]).count(), 1);
        assert!(traversal.tree.data(roots[0]).unwrap().size >= 7);
        assert_eq!(
            traversal.tree.len(),
            3,
            "discarded staging trees leave no live nodes"
        );
        assert_eq!(background.stats.io_errors, 0);
        assert_eq!(traversal.clean_search_roots.len(), 1);
        traversal.remove_children(roots[0], &traversal.tree.children(roots[0]).collect());
        assert_eq!(traversal.clean_search_roots.len(), 1);
        traversal.remove_children(traversal.root_index, &HashSet::from([roots[0]]));
        assert!(traversal.clean_search_roots.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn concurrent_clean_candidates_deduplicate_only_accepted_apfs_clones() {
        use std::os::unix::fs::MetadataExt as _;

        let directory = tempfile::tempdir().unwrap();
        let paths =
            ["bad", "first", "second"].map(|name| directory.path().join(name).join("node_modules"));
        for path in &paths {
            std::fs::create_dir_all(path).unwrap();
        }
        std::fs::create_dir_all(paths[0].join("nested/.git")).unwrap();
        let original = paths[0].join("payload");
        std::fs::write(&original, vec![1; 8192]).unwrap();
        for path in &paths[1..] {
            std::fs::copy(&original, path.join("payload")).unwrap();
        }
        let allocated = u128::from(std::fs::metadata(&original).unwrap().blocks()) * 512;
        for deduplicate in [false, true] {
            let mut options = clean_walk_options();
            options.threads = 4;
            options.apparent_size = false;
            options.metadata_options.apfs_clone_metadata = deduplicate;
            let mut traversal = Traversal::new();
            let mut background = BackgroundTraversal::start_clean(
                traversal.root_index,
                &options,
                vec![directory.path().to_owned()],
                None,
                None,
            )
            .unwrap();
            while background
                .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
                != Some(true)
            {}
            assert_eq!(traversal.clean_search_roots.len(), 2);
            let total: u128 = traversal
                .tree
                .children(traversal.root_index)
                .map(|root| {
                    let payload = traversal.tree.children(root).next().unwrap();
                    traversal.tree.data(payload).unwrap().size
                })
                .sum();
            assert_eq!(total, allocated * if deduplicate { 1 } else { 2 });
        }
    }

    #[test]
    fn clean_rejects_case_variants_of_nested_gitfiles() {
        for marker in [".git", ".GIT", ".Git"] {
            let directory = tempfile::tempdir().unwrap();
            let repo = gix::init(directory.path().join("admin")).unwrap();
            let search = directory.path().join("scan");
            let nested = search.join("node_modules/nested");
            std::fs::create_dir_all(&nested).unwrap();
            std::fs::write(
                nested.join(marker),
                format!("gitdir: {}\n", repo.path().display()),
            )
            .unwrap();
            let mut traversal = Traversal::new();
            let mut background = BackgroundTraversal::start_clean(
                traversal.root_index,
                &clean_walk_options(),
                vec![search],
                None,
                None,
            )
            .unwrap();
            while background
                .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
                != Some(true)
            {}
            assert_eq!(traversal.tree.len(), 1, "protect the {marker} worktree");
            assert!(traversal.clean_search_roots.is_empty());
        }
    }

    #[test]
    fn clean_empty_scan_finishes_without_roots() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("ordinary"), b"content").unwrap();
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start_clean(
            traversal.root_index,
            &clean_walk_options(),
            vec![directory.path().to_owned()],
            None,
            None,
        )
        .unwrap();
        while background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            != Some(true)
        {}
        assert_eq!(traversal.tree.len(), 1);
        assert_eq!(background.stats.total_bytes, Some(0));
        assert!(background.stats.elapsed.is_some());
    }

    #[test]
    fn clean_rejection_restores_hardlink_accounting() {
        let directory = tempfile::tempdir().unwrap();
        let bad = directory.path().join("a/node_modules");
        let good = directory.path().join("b/node_modules");
        std::fs::create_dir_all(bad.join("nested/repo/.git")).unwrap();
        std::fs::create_dir_all(&good).unwrap();
        std::fs::write(bad.join("payload"), b"content").unwrap();
        std::fs::hard_link(bad.join("payload"), good.join("payload")).unwrap();
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start_clean(
            traversal.root_index,
            &clean_walk_options(),
            vec![bad, good.clone()],
            None,
            None,
        )
        .unwrap();
        while background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            != Some(true)
        {}
        let roots = traversal
            .tree
            .children(traversal.root_index)
            .collect::<Vec<_>>();
        assert_eq!(roots.len(), 1);
        assert_eq!(
            traversal.tree.name(roots[0]).unwrap(),
            good.canonicalize().unwrap()
        );
        let payload = traversal.tree.children(roots[0]).next().unwrap();
        assert_eq!(
            traversal.tree.data(payload).unwrap().size,
            7,
            "a rejected root must not consume the accepted root's hardlink accounting"
        );
    }

    #[test]
    fn clean_rejects_containers_of_excluded_entries() {
        let directory = tempfile::tempdir().unwrap();
        let candidate = directory.path().join("node_modules");
        let protected = candidate.join("precious");
        std::fs::create_dir_all(&protected).unwrap();
        std::fs::write(protected.join("file"), b"keep").unwrap();
        let mut options = clean_walk_options();
        options
            .ignore_dirs
            .insert(protected.canonicalize().unwrap());
        let mut traversal = Traversal::new();
        let mut background = BackgroundTraversal::start_clean(
            traversal.root_index,
            &options,
            vec![candidate],
            None,
            None,
        )
        .unwrap();
        while background
            .integrate_traversal_event(&mut traversal, background.event_rx.recv().unwrap())
            != Some(true)
        {}
        assert_eq!(traversal.tree.len(), 1);
        assert_eq!(background.stats.io_errors, 0);
    }

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
                        skip_metadata: false,
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
    fn tree_removes_selected_children_in_one_batch() {
        let mut tree = Tree::new();
        let root_data = EntryData {
            size: 100,
            entry_count: Some(10),
            ..EntryData::default()
        };
        let root = tree.add_root("root", root_data);
        let tail = tree.add_child(root, "tail", EntryData::default());
        let kept = tree.add_child(root, "kept", EntryData::default());
        let kept_child = tree.add_child(kept, "kept-child", EntryData::default());
        let middle = tree.add_child(root, "middle", EntryData::default());
        let nested = tree.add_child(middle, "nested", EntryData::default());
        let head = tree.add_child(root, "head", EntryData::default());
        let other_root = tree.add_root("other", EntryData::default());
        let other_child = tree.add_child(other_root, "other-child", EntryData::default());
        let missing = TreeIndex::from(100_usize);

        assert_eq!(
            tree.remove_children(
                root,
                &HashSet::from([head, middle, tail, nested, kept_child, other_child, missing]),
            ),
            4,
            "only selected direct children and their subtrees are removed"
        );
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.children(root).collect::<Vec<_>>(), [kept]);
        assert_eq!(tree.children(kept).collect::<Vec<_>>(), [kept_child]);
        assert_eq!(tree.children(other_root).collect::<Vec<_>>(), [other_child]);
        assert_eq!(
            tree.data(root),
            Some(root_data),
            "accounting remains with the caller"
        );
        assert_eq!(tree.remove_children(missing, &HashSet::from([kept])), 0);
        assert_eq!(tree.remove_children(root, &HashSet::new()), 0);

        let reused = tree.add_child(root, "reused", EntryData::default());
        assert!([head, middle, nested, tail].contains(&reused));
        assert_eq!(tree.children(root).collect::<Vec<_>>(), [reused, kept]);
        assert_eq!(
            tree.remove_children(root, &HashSet::from([reused, kept])),
            3
        );
        assert!(tree.children(root).next().is_none());
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
