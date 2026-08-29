use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use dua::WalkOptions;
use dua::traverse::{BackgroundTraversal, TraversalStats};

use crate::interactive::widgets::Column;

use super::{EntryDataBundle, SortMode, input::TerminalFocus, navigation::Navigation};

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
    /// Snapshot destination for the initial scan, if requested.
    pub snapshot_export: Option<(PathBuf, Option<i32>)>,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "independent UI state flags are clearer than an artificial state machine"
)]
pub struct AppState {
    /// Navigation state for the main traversal view.
    pub navigation: Navigation,
    /// Navigation state for an active glob-filtered view, if one is open.
    pub glob_navigation: Option<Navigation>,
    /// Entries currently displayed in the active view.
    pub entries: Vec<EntryDataBundle>,
    /// Displayed entries that match known cleanup-directory names, or `None` if disabled.
    pub cleanup_candidates: Option<BTreeSet<dua::traverse::TreeIndex>>,
    /// Displayed entries ignored by the current git repository, or `None` if disabled.
    pub gitignored_entries: Option<BTreeSet<dua::traverse::TreeIndex>>,
    /// Active ordering for `entries`.
    pub sorting: SortMode,
    /// Optional columns explicitly enabled by the user.
    pub show_columns: HashSet<Column>,
    /// Status message shown in the footer.
    pub message: Option<String>,
    /// Pane that currently receives keyboard input.
    pub focussed: FocussedPane,
    /// Focus state shared with the terminal input reader.
    pub terminal_focus: TerminalFocus,
    /// Whether user input or terminal events have arrived since the current scan started.
    pub received_events: bool,
    /// Active background filesystem traversal, if a scan or refresh is running.
    pub scan: Option<FilesystemScan>,
    /// Latest traversal progress and error counters.
    pub stats: TraversalStats,
    /// Options used when starting filesystem walks.
    pub walk_options: WalkOptions,
    /// The paths used in the initial traversal, at least 1.
    pub root_paths: Vec<PathBuf>,
    /// Filesystem directory completely represented by the traversal root, if one exists.
    ///
    /// `Some(path)` means the root contains the complete, walk-option-filtered contents of
    /// `path`; this lets an upward scan reuse the existing tree as that directory's subtree.
    /// `None` means the root groups explicitly selected paths and may omit their siblings, so it
    /// cannot itself be treated as a complete directory.
    pub root_path: Option<PathBuf>,
    /// If true, listed entries will be validated for presence when switching directories.
    pub allow_entry_check: bool,
    /// Whether this traversal was loaded from a snapshot and must not touch the filesystem.
    pub read_only: bool,
    /// Whether the next quit/back action should exit the app.
    pub pending_exit: bool,
}

impl AppState {
    pub fn new(
        walk_options: WalkOptions,
        input: Vec<PathBuf>,
        root_path: Option<PathBuf>,
        read_only: bool,
    ) -> Self {
        AppState {
            navigation: Navigation::default(),
            glob_navigation: None,
            entries: vec![],
            cleanup_candidates: Some(BTreeSet::new()),
            gitignored_entries: Some(BTreeSet::new()),
            sorting: SortMode::default(),
            show_columns: HashSet::default(),
            message: None,
            focussed: FocussedPane::default(),
            terminal_focus: TerminalFocus::default(),
            received_events: false,
            scan: None,
            stats: TraversalStats::default(),
            walk_options,
            root_paths: input,
            root_path,
            allow_entry_check: !read_only,
            read_only,
            pending_exit: false,
        }
    }
}
