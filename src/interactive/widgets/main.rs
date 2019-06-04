use crate::{
    interactive::{
        widgets::{Entries, Footer, HelpPane, ListState},
        AppState, DisplayOptions, FocussedPane,
    },
    traverse::Traversal,
};
use tui::style::{Color, Style};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
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
            Some(state) => {
                let regions = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(entries_area);
                (regions[0], Some((regions[1], state)))
            }
            None => (entries_area, None),
        };
        let grey = || Style {
            fg: Color::DarkGray,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        };
        let white = || Style {
            fg: Color::White,
            ..grey()
        };
        let (entries_style, help_style) = match state.focussed {
            FocussedPane::Main => (white(), grey()),
            FocussedPane::Help => (grey(), white()),
        };
        Entries {
            tree: &tree,
            root: state.root,
            display: *display,
            sorting: state.sorting,
            selected: state.selected,
            border_style: entries_style,
            list: &mut draw_state.entries_list,
            is_focussed: if let FocussedPane::Main = state.focussed {
                true
            } else {
                false
            },
        }
        .draw(entries_area, buf);

        if let Some((help_area, state)) = help_area_state {
            HelpPane {
                state,
                border_style: help_style,
            }
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
