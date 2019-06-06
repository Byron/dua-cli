use crate::interactive::{
    widgets::{Entries, EntriesProps, Footer, FooterProps, Header, HelpPane, HelpPaneProps},
    AppState, DisplayOptions, FocussedPane,
};
use dua::traverse::Traversal;
use std::borrow::Borrow;
use tui::style::{Color, Style};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    widgets::Widget,
};
use tui_react::ToplevelComponent;

pub struct MainWindowProps<'a> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'a AppState,
}

#[derive(Default)]
pub struct MainWindow {
    pub help_pane: Option<HelpPane>,
    pub entries_pane: Entries,
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
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Max(256),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);
        let (header_area, entries_area, footer_area) = (regions[0], regions[1], regions[2]);
        let (entries_area, help_pane) = match self.help_pane {
            Some(ref mut pane) => {
                let regions = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(entries_area);
                (regions[0], Some((regions[1], pane)))
            }
            None => (entries_area, None),
        };
        let grey = Style {
            fg: Color::DarkGray,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        };
        let white = Style {
            fg: Color::White,
            ..grey
        };
        let (entries_style, help_style) = match state.focussed {
            FocussedPane::Main => (white, grey),
            FocussedPane::Help => (grey, white),
        };

        Header.draw(header_area, buf);
        let props = EntriesProps {
            tree: &tree,
            root: state.root,
            display: *display,
            entries: &state.entries,
            selected: state.selected,
            border_style: entries_style,
            is_focussed: if let FocussedPane::Main = state.focussed {
                true
            } else {
                false
            },
        };
        self.entries_pane.render(props, entries_area, buf);

        if let Some((help_area, pane)) = help_pane {
            let props = HelpPaneProps {
                border_style: help_style,
            };
            pane.render(props, help_area, buf);
        }

        Footer.render(
            FooterProps {
                total_bytes: *total_bytes,
                entries_traversed: *entries_traversed,
                format: display.byte_format,
                message: state.message.clone(),
            },
            footer_area,
            buf,
        );
    }
}
