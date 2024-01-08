

use dua::{
    traverse::{RunningTraversal}, WalkResult,
};


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
    pub received_event: bool,
    pub running_traversal: Option<RunningTraversal>,
}

pub enum ProcessingResult {
    ExitRequested(WalkResult),
}
