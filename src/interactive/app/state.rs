use std::collections::HashSet;
use std::path::PathBuf;

use dua::WalkOptions;
use dua::traverse::{BackgroundTraversal, TraversalStats};

use crate::interactive::widgets::Column;

use super::{EntryDataBundle, SortMode, navigation::Navigation};

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

pub struct FilesystemScan {
    pub active_traversal: BackgroundTraversal,
    /// The selected item prior to starting the traversal, if available, based on its name or index into [`AppState::entries`].
    pub previous_selection: Option<(PathBuf, usize)>,
}

pub struct AppState {
    pub navigation: Navigation,
    pub glob_navigation: Option<Navigation>,
    pub entries: Vec<EntryDataBundle>,
    pub sorting: SortMode,
    pub show_columns: HashSet<Column>,
    pub message: Option<String>,
    pub focussed: FocussedPane,
    pub received_events: bool,
    pub scan: Option<FilesystemScan>,
    pub stats: TraversalStats,
    pub walk_options: WalkOptions,
    /// The paths used in the initial traversal, at least 1.
    pub root_paths: Vec<PathBuf>,
    /// If true, listed entries will be validated for presence when switching directories.
    pub allow_entry_check: bool,
    pub pending_exit: bool,
}

impl AppState {
    pub fn new(walk_options: WalkOptions, input: Vec<PathBuf>) -> Self {
        AppState {
            navigation: Default::default(),
            glob_navigation: None,
            entries: vec![],
            sorting: Default::default(),
            show_columns: Default::default(),
            message: None,
            focussed: Default::default(),
            received_events: false,
            scan: None,
            stats: TraversalStats::default(),
            walk_options,
            root_paths: input,
            allow_entry_check: true,
            pending_exit: false,
        }
    }
}
