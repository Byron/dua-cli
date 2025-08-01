use crate::interactive::{
    state::{AppState, Cursor, FocussedPane},
    widgets::{
        Entries, EntriesProps, Footer, FooterProps, GlobPane, GlobPaneProps, Header, HelpPane,
        HelpPaneProps, MarkPane, MarkPaneProps, COLOR_MARKED,
    },
    DisplayOptions,
};
use std::borrow::Borrow;
use tui::buffer::Buffer;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    style::{Color, Style},
};
use Constraint::*;
use FocussedPane::*;

pub struct MainWindowProps<'a> {
    pub current_path: String,
    pub entries_traversed: u64,
    pub total_bytes: u128,
    pub start: std::time::Instant,
    pub elapsed: Option<std::time::Duration>,
    pub display: DisplayOptions,
    pub state: &'a AppState,
}

#[derive(Default)]
pub struct MainWindow {
    pub help_pane: Option<HelpPane>,
    pub entries_pane: Entries,
    pub mark_pane: Option<MarkPane>,
    pub glob_pane: Option<GlobPane>,
}

impl MainWindow {
    pub fn render<'a>(
        &mut self,
        props: impl Borrow<MainWindowProps<'a>>,
        area: Rect,
        buffer: &mut Buffer,
        cursor: &mut Cursor,
    ) {
        let MainWindowProps {
            current_path,
            entries_traversed,
            total_bytes,
            start,
            elapsed,
            display,
            state,
        } = props.borrow();

        let (entries_style, help_style, mark_style, glob_style) = pane_border_style(state.focussed);
        let (header_area, content_area, footer_area) = main_window_layout(area);

        let header_bg_color = header_background_color(self.is_anything_marked(), state.focussed);
        Header.render(header_bg_color, header_area, buffer);

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

        let (entries_area, glob_pane) = match &mut self.glob_pane {
            Some(ref mut glob_pane) => {
                let regions = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Max(256), Length(3)].as_ref())
                    .split(entries_area);
                (regions[0], Some((regions[1], glob_pane)))
            }
            None => (entries_area, None),
        };

        if let Some((mark_area, pane)) = mark_pane {
            let props = MarkPaneProps {
                border_style: mark_style,
                format: display.byte_format,
            };
            pane.render(props, mark_area, buffer);
        }

        if let Some((help_area, pane)) = help_pane {
            let props = HelpPaneProps {
                border_style: help_style,
                has_focus: matches!(state.focussed, Help),
            };
            pane.render(props, help_area, buffer);
        }

        let marked = self.mark_pane.as_ref().map(|p| p.marked());
        let props = EntriesProps {
            current_path: current_path.clone(),
            display: *display,
            entries: &state.entries,
            marked,
            selected: state.navigation().selected,
            border_style: entries_style,
            is_focussed: matches!(state.focussed, Main),
            sort_mode: state.sorting,
            show_columns: &state.show_columns,
        };
        self.entries_pane.render(props, entries_area, buffer);

        if let Some((glob_area, pane)) = glob_pane {
            let props = GlobPaneProps {
                border_style: glob_style,
                has_focus: matches!(state.focussed, Glob),
            };
            pane.render(props, glob_area, buffer, cursor);
        }

        Footer.render(
            FooterProps {
                total_bytes: *total_bytes,
                format: display.byte_format,
                entries_traversed: *entries_traversed,
                message: state.message.clone(),
                traversal_start: *start,
                elapsed: *elapsed,
                sort_mode: state.sorting,
                pending_exit: state.pending_exit,
            },
            footer_area,
            buffer,
        );
    }

    fn is_anything_marked(&self) -> bool {
        self.mark_pane
            .as_ref()
            .map(|p| p.marked())
            .is_none_or(|m| m.is_empty())
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

fn pane_border_style(focused_pane: FocussedPane) -> (Style, Style, Style, Style) {
    let grey = Style {
        fg: Color::DarkGray.into(),
        bg: Color::Reset.into(),
        add_modifier: Modifier::empty(),
        ..Style::default()
    };
    let bold = Style::default().add_modifier(Modifier::BOLD);
    match focused_pane {
        Main => (bold, grey, grey, grey),
        Help => (grey, bold, grey, grey),
        Mark => (grey, grey, bold, grey),
        Glob => (grey, grey, grey, bold),
    }
}
