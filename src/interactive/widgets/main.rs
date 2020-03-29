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
                    ..
                },
            display,
            state,
        } = props.borrow();

        let (entries_style, help_style, mark_style) = {
            let grey = Style {
                fg: Color::DarkGray,
                bg: Color::Reset,
                modifier: Modifier::empty(),
            };
            let bold = Style {
                fg: Color::Rgb(230, 230, 230),
                modifier: Modifier::BOLD,
                ..grey
            };
            match state.focussed {
                Main => (bold, grey, grey),
                Help => (grey, bold, grey),
                Mark => (grey, grey, bold),
            }
        };

        let (header_area, entries_area, footer_area) = {
            let regions = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Length(1), Max(256), Length(1)].as_ref())
                .split(area);
            (regions[0], regions[1], regions[2])
        };
        {
            let marked = self.mark_pane.as_ref().map(|p| p.marked());
            let bg_color = match (marked.map_or(true, |m| m.is_empty()), state.focussed) {
                (false, FocussedPane::Mark) => Color::LightRed,
                (false, _) => COLOR_MARKED,
                (_, _) => Color::White,
            };
            Header.render(bg_color, header_area, buf);
        }
        let (entries_area, help_pane, mark_pane) = {
            let regions = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Percentage(50), Percentage(50)].as_ref())
                .split(entries_area);
            let (left_pane, right_pane) = (regions[0], regions[1]);
            match (&mut self.help_pane, &mut self.mark_pane) {
                (Some(ref mut pane), None) => (left_pane, Some((right_pane, pane)), None),
                (None, Some(ref mut pane)) => (left_pane, None, Some((right_pane, pane))),
                (Some(ref mut help), Some(ref mut mark)) => {
                    let regions = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Percentage(50), Percentage(50)].as_ref())
                        .split(right_pane);
                    (
                        left_pane,
                        Some((regions[0], help)),
                        Some((regions[1], mark)),
                    )
                }
                (None, None) => (entries_area, None, None),
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
                has_focus: if let Help = state.focussed {
                    true
                } else {
                    false
                },
            };
            pane.render(props, help_area, buf);
        }

        let marked = self.mark_pane.as_ref().map(|p| p.marked());
        let props = EntriesProps {
            tree: &tree,
            root: state.root,
            display: *display,
            entries: &state.entries,
            marked,
            selected: state.selected,
            border_style: entries_style,
            is_focussed: if let Main = state.focussed {
                true
            } else {
                false
            },
        };
        self.entries_pane.render(props, entries_area, buf);

        Footer.render(
            FooterProps {
                total_bytes: *total_bytes,
                format: display.byte_format,
                entries_traversed: *entries_traversed,
                message: state.message.clone(),
            },
            footer_area,
            buf,
        );
    }
}
