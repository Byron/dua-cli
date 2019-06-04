use crate::{
    interactive::{
        widgets::{Entries, Footer},
        DisplayOptions,
    },
    traverse::{Traversal, TreeIndex},
};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    SizeDescending,
    SizeAscending,
}

impl SortMode {
    pub fn toggle_size(&mut self) {
        use SortMode::*;
        *self = match self {
            SizeAscending => SizeDescending,
            SizeDescending => SizeAscending,
        }
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::SizeDescending
    }
}

pub struct DisplayState {
    pub root: TreeIndex,
    pub selected: Option<TreeIndex>,
    pub entries_list_start: usize,
    pub sorting: SortMode,
    pub message: Option<String>,
}

pub struct MainWindow<'a, 'b> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'b DisplayState,
}

impl<'a, 'b, 'c> Widget for MainWindow<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            traversal:
                Traversal {
                    tree,
                    entries_traversed,
                    total_bytes,
                    ..
                },
            display,
            state,
        } = self;
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(256), Constraint::Length(1)].as_ref())
            .split(area);
        let (entries, footer) = (regions[0], regions[1]);
        Entries {
            tree: &tree,
            root: state.root,
            display: *display,
            sorting: state.sorting,
            selected: state.selected,
            list_start: state.entries_list_start,
        }
        .draw(entries, buf);

        Footer {
            total_bytes: *total_bytes,
            entries_traversed: *entries_traversed,
            format: display.byte_format,
            message: state.message.clone(),
        }
        .draw(footer, buf);
    }
}
