use crate::{
    interactive::{
        widgets::{Entries, Footer, ListState},
        AppState, DisplayOptions,
    },
    traverse::Traversal,
};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

#[derive(Default)]
pub struct DrawState {
    entries_list: ListState,
}

pub struct MainWindow<'a, 'b, 'c> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'b AppState,
    pub widgets: &'c mut DrawState,
}

impl<'a, 'b, 'c> Widget for MainWindow<'a, 'b, 'c> {
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
            ref mut widgets,
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
            list: &mut widgets.entries_list,
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
