use crate::interactive::{
    DisplayOptions,
    state::{AppState, Cursor, FocussedPane},
    widgets::{
        COLOR_MARKED, Entries, EntriesProps, Footer, FooterProps, GlobPane, GlobPaneProps, Header,
        HelpPane, HelpPaneProps, MarkPane, MarkPaneProps,
    },
};
use Constraint::{Length, Max, Percentage};
use FocussedPane::{Glob, Help, Main, Mark};
use std::borrow::Borrow;
use std::path::PathBuf;
use tui::buffer::Buffer;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    style::{Color, Style},
};

pub struct MainWindowProps<'a> {
    pub current_path: PathBuf,
    pub entries_traversed: u64,
    pub total_bytes: u128,
    pub start: std::time::Instant,
    pub elapsed: Option<std::time::Duration>,
    pub display: DisplayOptions,
    pub state: &'a AppState,
    pub config: &'a dua::Config,
}

#[derive(Default)]
pub struct MainWindow {
    pub help: Option<HelpPane>,
    pub entries: Entries,
    pub mark: Option<MarkPane>,
    pub glob: Option<GlobPane>,
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
            config,
        } = props.borrow();

        let (entries_style, help_style, mark_style, glob_style) = pane_border_style(state.focussed);
        let (header_area, content_area, footer_area) = main_window_layout(area);

        let safety_notice = mark_safety_notice(state.read_only, &config.keys);

        let header_bg_color =
            header_background_color(self.has_marks() && safety_notice.is_none(), state.focussed);
        Header::render(header_bg_color, header_area, buffer);

        let (entries_area, help_pane, mark_pane) = {
            let (left_pane, right_pane) = content_layout(content_area);
            match (&mut self.help, &mut self.mark) {
                (Some(pane), None) => (left_pane, Some((right_pane, pane)), None),
                (None, Some(pane)) => (left_pane, None, Some((right_pane, pane))),
                (Some(help), Some(mark)) => {
                    let (top_area, bottom_area) = right_pane_layout(right_pane);
                    (left_pane, Some((top_area, help)), Some((bottom_area, mark)))
                }
                (None, None) => (content_area, None, None),
            }
        };

        let (entries_area, glob_pane) = match &mut self.glob {
            Some(glob_pane) => {
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
                root_total_size: *total_bytes,
                keys: &config.keys,
                safety_notice,
            };
            pane.render(props, mark_area, buffer);
        }

        if let Some((help_area, pane)) = help_pane {
            let props = HelpPaneProps {
                border_style: help_style,
                has_focus: matches!(state.focussed, Help),
                keys: &config.keys,
            };
            pane.render(props, help_area, buffer);
        }

        let marked = self.mark.as_ref().map(|pane| pane.marked());
        let props = EntriesProps {
            current_path: current_path.clone(),
            display: *display,
            entries: &state.entries,
            marked,
            cleanup_candidates: state.cleanup_candidates.as_ref(),
            gitignored_entries: state.gitignored_entries.as_ref(),
            selected: state.navigation().selected,
            border_style: entries_style,
            is_focussed: matches!(state.focussed, Main),
            sort_mode: state.sorting,
            show_columns: &state.show_columns,
            keys: &config.keys,
        };
        self.entries.render(props, entries_area, buffer);

        if let Some((glob_area, pane)) = glob_pane {
            let props = GlobPaneProps {
                border_style: glob_style,
                has_focus: matches!(state.focussed, Glob),
                keys: &config.keys,
            };
            pane.render(props, glob_area, buffer, cursor);
        }

        Footer::render(
            FooterProps {
                total_bytes: *total_bytes,
                format: display.byte_format,
                message: state.message.clone(),
                traversal_stats: (state.scan.is_some() || !state.received_events).then_some((
                    *entries_traversed,
                    *start,
                    *elapsed,
                )),
                sort_mode: state.sorting,
                pending_exit: state.pending_exit,
                keys: &config.keys,
            },
            footer_area,
            buffer,
        );
    }

    fn has_marks(&self) -> bool {
        self.mark
            .as_ref()
            .map(|pane| pane.marked())
            .is_some_and(|marked| !marked.is_empty())
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

fn mark_safety_notice(read_only: bool, keys: &dua::KeysConfig) -> Option<&'static str> {
    if read_only {
        Some(" Snapshot is read-only; marked entries cannot be deleted ")
    } else if keys.delete_marked.is_empty()
        && (!cfg!(feature = "trash-move") || keys.trash_marked.is_empty())
    {
        Some(" No destructive keys are mapped; marked entries are safe ")
    } else {
        None
    }
}

fn header_background_color(has_dangerous_marks: bool, focused_pane: FocussedPane) -> Color {
    match (has_dangerous_marks, focused_pane) {
        (true, Mark) => Color::LightRed,
        (true, _) => COLOR_MARKED,
        (false, _) => Color::White,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_are_only_dangerous_when_a_destructive_action_is_available() {
        let config = dua::Config::default();
        assert!(mark_safety_notice(false, &config.keys).is_none());
        assert_eq!(header_background_color(true, Mark), Color::LightRed);

        assert!(mark_safety_notice(true, &config.keys).is_some());
        assert_eq!(header_background_color(false, Mark), Color::White);

        let config: dua::Config =
            toml::from_str("[keys]\ndelete_marked = []\ntrash_marked = []").expect("valid config");
        assert!(mark_safety_notice(false, &config.keys).is_some());
    }
}
