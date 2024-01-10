use std::collections::HashSet;

use dua::traverse::BackgroundTraversal;

use crate::interactive::widgets::Column;

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
    pub show_columns: HashSet<Column>,
    pub message: Option<String>,
    pub focussed: FocussedPane,
    pub received_events: bool,
    pub active_traversal: Option<BackgroundTraversal>,
}
