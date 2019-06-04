use crate::{
    interactive::{
        widgets::{Entries, Footer, ListState},
        AppState, DisplayOptions,
    },
    traverse::Traversal,
};
use tui::widgets::Block;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

/// The state that can be mutated while drawing
/// This is easiest compared to alternatives, but at least it's restricted to a subset of the state
#[derive(Default)]
pub struct DrawState {
    entries_list: ListState,
}

pub struct MainWindow<'a, 'b, 'c> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'b AppState,
    pub draw_state: &'c mut DrawState,
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
            ref mut draw_state,
        } = self;
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(256), Constraint::Length(1)].as_ref())
            .split(area);
        let (entries_area, footer_area) = (regions[0], regions[1]);
        let (entries_area, help_area_state) = match state.help_pane {
            Some(state) => (entries_area, Some((entries_area, state))),
            None => (entries_area, None),
        };
        Entries {
            tree: &tree,
            root: state.root,
            display: *display,
            sorting: state.sorting,
            selected: state.selected,
            list: &mut draw_state.entries_list,
        }
        .draw(entries_area, buf);

        if let Some((help_area, _)) = help_area_state {
            use tui::widgets::Borders;
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .draw(help_area, buf);
        }

        Footer {
            total_bytes: *total_bytes,
            entries_traversed: *entries_traversed,
            format: display.byte_format,
            message: state.message.clone(),
        }
        .draw(footer_area, buf);
    }
}
