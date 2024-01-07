use dua::{WalkResult, traverse::{TreeIndex, Tree}, inodefilter::InodeFilter};
use petgraph::Direction;

use super::{navigation::Navigation, EntryDataBundle, SortMode};

#[derive(Default, Copy, Clone, PartialEq)]
pub enum FocussedPane {
    #[default]
    Main,
    Help,
    Mark,
    Glob,
}

#[derive(Default)]
pub struct Cursor {
    pub show: bool,
    pub x: u16,
    pub y: u16,
}

#[derive(Default)]
pub struct AppState {
    pub navigation: Navigation,
    pub glob_navigation: Option<Navigation>,
    pub entries: Vec<EntryDataBundle>,
    pub sorting: SortMode,
    pub message: Option<String>,
    pub focussed: FocussedPane,
    pub is_scanning: bool,
    pub traversal_state: TraversalState,
}


#[derive(Default)]
pub struct TraversalState {
    pub previous_node_idx: TreeIndex,
    pub parent_node_idx: TreeIndex,
    pub directory_info_per_depth_level: Vec<EntryInfo>,
    pub current_directory_at_depth: EntryInfo,
    pub previous_depth: usize,
    pub inodes: InodeFilter,
}

impl TraversalState {
    pub fn new(root_idx: TreeIndex) -> Self {
        Self {
            previous_node_idx: root_idx,
            parent_node_idx: root_idx,
            directory_info_per_depth_level: Vec::new(),
            current_directory_at_depth: EntryInfo::default(),
            previous_depth: 0,
            inodes: InodeFilter::default(),
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

pub enum ProcessingResult {
    Finished(WalkResult),
    ExitRequested(WalkResult),
}
