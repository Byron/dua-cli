use crate::interactive::{
    widgets::{
        Entries, EntriesProps, Footer, FooterProps, Header, HelpPane, HelpPaneProps, MarkPane,
        MarkPaneProps, COLOR_MARKED,
    },
    AppState, DisplayOptions, FocussedPane,
};
use dua::traverse::Traversal;
use std::borrow::Borrow;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    style::{Color, Style},
};
use Constraint::*;
use FocussedPane::*;

pub struct MainWindowProps<'a> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'a AppState,
}

#[derive(Default)]
pub struct MainWindow {
    pub help_pane: Option<HelpPane>,
    pub entries_pane: Entries,
    pub mark_pane: Option<MarkPane>,
}

impl MainWindow {
    pub fn render<'a>(
        &mut self,
        props: impl Borrow<MainWindowProps<'a>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let MainWindowProps {
            traversal:
                Traversal {
                    tree,
                    entries_traversed,
                    total_bytes,
                    start,
                    elapsed,
                    ..
                },
            display,
            state,
        } = props.borrow();

        let (entries_style, help_style, mark_style) = pane_border_style(state.focussed);
        let (header_area, content_area, footer_area) = main_window_layout(area);

        let header_bg_color = header_background_color(self.is_anything_marked(), state.focussed);
        Header.render(header_bg_color, header_area, buf);

        let (entries_area, help_pane, mark_pane) = {
            let (left_pane, right_pane) = content_layout(content_area);
            match (&mut self.help_pane, &mut self.mark_pane) {
                (Some(ref mut pane), None) => (left_pane, Some((right_pane, pane)), None),
                (None, Some(ref mut pane)) => (left_pane, None, Some((right_pane, pane))),
                (Some(ref mut help), Some(ref mut mark)) => {
                    let (top_area, bottom_area) = right_pane_layout(right_pane);
                    (left_pane, Some((top_area, help)), Some((bottom_area, mark)))
                }
                (None, None) => (content_area, None, None),
            }
        };

        if let Some((mark_area, pane)) = mark_pane {
            let props = MarkPaneProps {
                border_style: mark_style,
                format: display.byte_format,
            };
            pane.render(props, mark_area, buf);
        }

        if let Some((help_area, pane)) = help_pane {
            let props = HelpPaneProps {
                border_style: help_style,
                has_focus: matches!(state.focussed, Help),
            };
            pane.render(props, help_area, buf);
        }

        let marked = self.mark_pane.as_ref().map(|p| p.marked());
        let props = EntriesProps {
            tree,
            root: state.root,
            display: *display,
            entries: &state.entries,
            marked,
            selected: state.selected,
            border_style: entries_style,
            is_focussed: matches!(state.focussed, Main),
            sort_mode: state.sorting,
        };
        self.entries_pane.render(props, entries_area, buf);

        Footer.render(
            FooterProps {
                total_bytes: *total_bytes,
                format: display.byte_format,
                entries_traversed: *entries_traversed,
                message: state.message.clone(),
                traversal_start: *start,
                elapsed: *elapsed,
                sort_mode: state.sorting,
            },
            footer_area,
            buf,
        );
    }

    fn is_anything_marked(&self) -> bool {
        self.mark_pane
            .as_ref()
            .map(|p| p.marked())
            .map_or(true, |m| m.is_empty())
    }
}

fn right_pane_layout(right_pane: Rect) -> (Rect, Rect) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Percentage(50), Percentage(50)].as_ref())
        .split(right_pane);
    (regions[0], regions[1])
}

fn content_layout(content_area: Rect) -> (Rect, Rect) {
    let regions = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Percentage(50), Percentage(50)].as_ref())
        .split(content_area);
    (regions[0], regions[1])
}

fn header_background_color(is_marked: bool, focused_pane: FocussedPane) -> Color {
    match (is_marked, focused_pane) {
        (false, Mark) => Color::LightRed,
        (false, _) => COLOR_MARKED,
        (_, _) => Color::White,
    }
}

fn main_window_layout(area: Rect) -> (Rect, Rect, Rect) {
    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Length(1), Max(256), Length(1)].as_ref())
        .split(area);
    (regions[0], regions[1], regions[2])
}

fn pane_border_style(focused_pane: FocussedPane) -> (Style, Style, Style) {
    let grey = Style {
        fg: Color::DarkGray.into(),
        bg: Color::Reset.into(),
        add_modifier: Modifier::empty(),
        ..Style::default()
    };
    let bold = Style::default().add_modifier(Modifier::BOLD);
    match focused_pane {
        Main => (bold, grey, grey),
        Help => (grey, bold, grey),
        Mark => (grey, grey, bold),
    }
}
