use crate::interactive::widgets::tui_ext::{
    draw_text_nowrap_fn,
    util::{block_width, rect},
};
use crate::interactive::{
    DisplayOptions,
    state::{AppState, Cursor, FocussedPane},
    widgets::{
        COLOR_MARKED, Entries, EntriesProps, Footer, FooterProps, GlobPane, GlobPaneProps, Header,
        HelpPane, HelpPaneProps, Language, MarkPane, MarkPaneProps,
    },
};
use Constraint::{Length, Max, Min, Percentage};
use FocussedPane::{Glob, Help, Main, Mark};
use std::borrow::Borrow;
use std::path::PathBuf;
use tui::buffer::Buffer;
use tui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Modifier,
    style::{Color, Style},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
    /// Keep open right-hand panes as a narrow shared sidebar while retaining their contents.
    pub right_panes_minimized: bool,
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
        let language = state.language;
        if let Some(pane) = self.mark.as_mut() {
            let has_focus = state.focussed == Mark;
            if pane.has_focus() != has_focus {
                pane.set_focus(has_focus);
            }
        }

        let (entries_style, help_style, mark_style, glob_style) = pane_border_style(state.focussed);
        let (header_area, content_area, footer_area) = main_window_layout(area);

        let safety_notice = mark_safety_notice(state.read_only, &config.keys, language);

        let header_bg_color =
            header_background_color(self.has_marks() && safety_notice.is_none(), state.focussed);
        Header::render(language, header_bg_color, header_area, buffer);

        let (entries_area, help_pane, mark_pane) = {
            let (left_pane, right_pane) =
                content_layout(content_area, self.right_panes_minimized, language);
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
            if self.right_panes_minimized {
                render_minimized_label(
                    &format!(
                        " {} {}",
                        pane.marked().len(),
                        language.ui_text().marked_label
                    ),
                    mark_area,
                    buffer,
                    mark_style,
                );
            } else {
                let props = MarkPaneProps {
                    border_style: mark_style,
                    format: display.byte_format,
                    root_total_size: *total_bytes,
                    keys: &config.keys,
                    safety_notice,
                    language,
                };
                pane.render(props, mark_area, buffer);
                if !matches!(state.focussed, Mark | Glob) {
                    render_collapse_hint(mark_area, buffer, &config.keys, language);
                }
            }
        }

        if let Some((help_area, pane)) = help_pane {
            if self.right_panes_minimized {
                render_minimized_label(
                    language.help_text().block_title,
                    help_area,
                    buffer,
                    help_style,
                );
            } else {
                let props = HelpPaneProps {
                    border_style: help_style,
                    has_focus: matches!(state.focussed, Help),
                    keys: &config.keys,
                    language,
                };
                pane.render(props, help_area, buffer);
                if !matches!(state.focussed, Help | Glob) {
                    render_collapse_hint(help_area, buffer, &config.keys, language);
                }
            }
        }

        let marked = self.mark.as_ref().map(|pane| pane.marked());
        let props = EntriesProps {
            current_path: current_path.clone(),
            display: *display,
            directory_suffix: config.directory_suffix,
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
            language,
        };
        self.entries.render(props, entries_area, buffer);

        if let Some((glob_area, pane)) = glob_pane {
            let props = GlobPaneProps {
                border_style: glob_style,
                has_focus: matches!(state.focussed, Glob),
                keys: &config.keys,
                language,
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
                language,
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

fn content_layout(content_area: Rect, minimized: bool, language: Language) -> (Rect, Rect) {
    let regions = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if minimized {
            let label_width = [
                language.help_text().block_title,
                language.ui_text().marked_label,
            ]
            .into_iter()
            .flat_map(|label| label.graphemes(true))
            .map(block_width)
            .max()
            .unwrap_or(1);
            [Min(0), Length(label_width + 2)]
        } else {
            [Percentage(50), Percentage(50)]
        })
        .split(content_area);
    (regions[0], regions[1])
}

fn render_minimized_label(label: &str, area: Rect, buffer: &mut Buffer, style: Style) {
    let area = area.intersection(buffer.area).inner(Margin::new(1, 0));
    if area.is_empty() {
        return;
    }
    let mut graphemes = label.graphemes(true).peekable();
    for y in area.y..area.bottom() {
        let Some(grapheme) = graphemes.next() else {
            break;
        };
        let clipped = grapheme.width() > usize::from(area.width)
            || (y + 1 == area.bottom() && graphemes.peek().is_some());
        buffer.set_stringn(
            area.x,
            y,
            if clipped { "…" } else { grapheme },
            usize::from(area.width),
            style,
        );
        if clipped {
            break;
        }
    }
}

fn render_collapse_hint(
    area: Rect,
    buffer: &mut Buffer,
    keys: &dua::KeysConfig,
    language: crate::interactive::widgets::Language,
) {
    if area.height < 2 || keys.toggle_right_panes.is_empty() {
        return;
    }
    let hint = format!(
        " {} = {} ",
        language.ui_text().toggle_collapse,
        keys.toggle_right_panes.primary(),
    );
    let width = block_width(&hint);
    if width <= area.width.saturating_sub(2) {
        let bottom = Rect {
            y: area.bottom() - 1,
            width: area.width - 1,
            height: 1,
            ..area
        };
        draw_text_nowrap_fn(
            rect::snap_to_right(bottom, width),
            buffer,
            &hint,
            |_, _, _| Style::default(),
        );
    }
}

fn mark_safety_notice(
    read_only: bool,
    keys: &dua::KeysConfig,
    language: crate::interactive::widgets::Language,
) -> Option<&'static str> {
    let t = language.ui_text();
    if read_only {
        Some(t.mark_snapshot_read_only)
    } else if keys.delete_marked.is_empty()
        && (!cfg!(feature = "trash-move") || keys.trash_marked.is_empty())
    {
        Some(t.mark_no_destructive_keys)
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

    fn render_window(window: &mut MainWindow, area: Rect, language: Language) -> Buffer {
        render_window_with_focus_and_keys(window, area, language, Main, &dua::Config::default())
    }

    fn render_window_with_focus_and_keys(
        window: &mut MainWindow,
        area: Rect,
        language: Language,
        focused: FocussedPane,
        config: &dua::Config,
    ) -> Buffer {
        let mut state = AppState::new(crate::snapshot_walk_options(), Vec::new(), None, false);
        state.language = language;
        state.received_events = true;
        state.focussed = focused;
        let mut buffer = Buffer::empty(area);
        window.render(
            MainWindowProps {
                current_path: PathBuf::from("."),
                entries_traversed: 0,
                total_bytes: 0,
                start: std::time::Instant::now(),
                elapsed: None,
                display: DisplayOptions::new(dua::ByteFormat::Bytes),
                state: &state,
                config,
            },
            area,
            &mut buffer,
            &mut Cursor::default(),
        );
        buffer
    }

    #[test]
    fn collapse_hints_only_show_on_unfocused_pane_borders() {
        let mut window = MainWindow {
            help: Some(HelpPane::default()),
            mark: Some(MarkPane::default()),
            ..Default::default()
        };
        for focused in [Main, Help, Mark, Help, Glob, Main] {
            let buffer = render_window_with_focus_and_keys(
                &mut window,
                Rect::new(0, 0, 120, 24),
                Language::English,
                focused,
                &dua::Config::default(),
            );
            let row = |y| {
                (60..120)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            };
            assert_eq!(
                row(11).contains("toggle-collapse = ]"),
                !matches!(focused, Help | Glob)
            );
            assert_eq!(
                row(22).contains("toggle-collapse = ]"),
                !matches!(focused, Mark | Glob)
            );
            assert_eq!(window.mark.as_ref().unwrap().has_focus(), focused == Mark);
        }
        for (binding, expected) in [("'ctrl+b'", true), ("[]", false)] {
            let config: dua::Config =
                toml::from_str(&format!("[keys]\ntoggle_right_panes = {binding}")).unwrap();
            let buffer = render_window_with_focus_and_keys(
                &mut window,
                Rect::new(0, 0, 120, 24),
                Language::English,
                Main,
                &config,
            );
            let row: String = (60..120).map(|x| buffer[(x, 22)].symbol()).collect();
            assert_eq!(row.contains("toggle-collapse = Ctrl + b"), expected);
            assert!(!row.contains("toggle-collapse = ]"));
            assert!(!row.contains("<unmapped>"));
        }
        let buffer = render_window(&mut window, Rect::new(0, 0, 40, 24), Language::English);
        let row: String = (20..40).map(|x| buffer[(x, 22)].symbol()).collect();
        assert!(row.chars().all(|c| matches!(c, '└' | '─' | '┘')));
        let buffer = render_window(&mut window, Rect::new(0, 0, 120, 24), Language::German);
        let row: String = (60..120).map(|x| buffer[(x, 22)].symbol()).collect();
        assert!(row.contains("ein-/ausklappen = ]"));
    }

    #[test]
    fn minimized_help_uses_three_column_sidebar_with_padding() {
        let mut window = MainWindow {
            help: Some(HelpPane::default()),
            right_panes_minimized: true,
            ..Default::default()
        };
        let buffer = render_window(&mut window, Rect::new(0, 0, 30, 10), Language::English);
        for (row, character) in ["H", "e", "l", "p"].into_iter().enumerate() {
            assert_eq!(buffer[(27, row as u16 + 1)].symbol(), " ");
            assert_eq!(buffer[(28, row as u16 + 1)].symbol(), character);
            assert_eq!(buffer[(29, row as u16 + 1)].symbol(), " ");
        }
        assert_eq!(
            buffer[(26, 2)].symbol(),
            "│",
            "entries reclaim the released columns"
        );
    }

    #[test]
    fn minimized_marked_and_shared_sidebars_use_available_height() {
        for help in [false, true] {
            let mut window = MainWindow {
                help: help.then(HelpPane::default),
                mark: Some(MarkPane::default()),
                right_panes_minimized: true,
                ..Default::default()
            };
            let buffer = render_window(&mut window, Rect::new(0, 0, 30, 10), Language::English);
            let (start, letters): (u16, &[_]) = if help {
                (5, &[" ", "0", " ", "…"])
            } else {
                (1, &[" ", "0", " ", "M", "a", "r", "k", "…"])
            };
            for (row, letter) in letters.iter().enumerate() {
                assert_eq!(buffer[(27, start + row as u16)].symbol(), " ");
                assert_eq!(buffer[(28, start + row as u16)].symbol(), *letter);
                assert_eq!(buffer[(29, start + row as u16)].symbol(), " ");
            }
            if help {
                assert_eq!(buffer[(28, 1)].symbol(), "H");
                assert_eq!(buffer[(28, 4)].symbol(), "p");
            }
        }
    }

    #[test]
    fn minimized_labels_render_wide_translations_and_intact_graphemes() {
        let mut help = MainWindow {
            help: Some(HelpPane::default()),
            right_panes_minimized: true,
            ..Default::default()
        };
        let buffer = render_window(&mut help, Rect::new(0, 0, 30, 10), Language::Japanese);
        for (row, grapheme) in ["ヘ", "ル", "プ"].into_iter().enumerate() {
            assert_eq!(buffer[(26, row as u16 + 1)].symbol(), " ");
            assert_eq!(buffer[(27, row as u16 + 1)].symbol(), grapheme);
            assert_eq!(buffer[(28, row as u16 + 1)].symbol(), " ");
            assert_eq!(buffer[(29, row as u16 + 1)].symbol(), " ");
        }
        let mut marked = MainWindow {
            mark: Some(MarkPane::default()),
            right_panes_minimized: true,
            ..Default::default()
        };
        let buffer = render_window(&mut marked, Rect::new(0, 0, 30, 10), Language::Chinese);
        for (row, grapheme) in [" ", "0", " ", "已", "标", "记"].into_iter().enumerate() {
            assert_eq!(buffer[(26, row as u16 + 1)].symbol(), " ");
            assert_eq!(buffer[(27, row as u16 + 1)].symbol(), grapheme);
            assert_eq!(buffer[(29, row as u16 + 1)].symbol(), " ");
        }

        let area = Rect::new(3, 4, 4, 3);
        let mut buffer = Buffer::empty(area);
        render_minimized_label("e\u{301}界!", area, &mut buffer, Style::default());
        assert_eq!(buffer[(4, 4)].symbol(), "e\u{301}");
        assert_eq!(buffer[(4, 5)].symbol(), "界");
        assert_eq!(buffer[(4, 6)].symbol(), "!");
        for row in 4..7 {
            assert_eq!(buffer[(3, row)].symbol(), " ");
            assert_eq!(buffer[(6, row)].symbol(), " ");
        }
    }

    #[test]
    fn minimized_layout_handles_tiny_and_wide_areas_without_reserving_absent_panes() {
        let (entries, sidebar) = content_layout(Rect::new(5, 6, 600, 14), true, Language::English);
        assert_eq!(entries, Rect::new(5, 6, 597, 14));
        assert_eq!(sidebar, Rect::new(602, 6, 3, 14));

        let mut absent = MainWindow::default();
        let area = Rect::new(0, 0, 30, 10);
        let expanded = render_window(&mut absent, area, Language::English);
        absent.right_panes_minimized = true;
        assert_eq!(
            render_window(&mut absent, area, Language::English),
            expanded
        );

        for width in 0..=4 {
            for height in 0..=5 {
                let mut window = MainWindow {
                    help: Some(HelpPane::default()),
                    mark: Some(MarkPane::default()),
                    glob: Some(GlobPane::default()),
                    right_panes_minimized: true,
                    ..Default::default()
                };
                render_window(
                    &mut window,
                    Rect::new(2, 3, width, height),
                    Language::Japanese,
                );
                assert!(window.help.is_some() && window.mark.is_some() && window.glob.is_some());
            }
        }

        let area = Rect::new(3, 4, 3, 2);
        let mut buffer = Buffer::empty(area);
        render_minimized_label("帮助", area, &mut buffer, Style::default());
        assert_eq!(buffer[(3, 4)].symbol(), " ");
        assert_eq!(buffer[(4, 4)].symbol(), "…");
        assert_eq!(buffer[(5, 4)].symbol(), " ");
        assert_eq!(buffer[(4, 5)].symbol(), " ");
    }

    #[test]
    fn marks_are_only_dangerous_when_a_destructive_action_is_available() {
        let config = dua::Config::default();
        assert!(mark_safety_notice(false, &config.keys, Language::English).is_none());
        assert_eq!(header_background_color(true, Mark), Color::LightRed);

        assert!(mark_safety_notice(true, &config.keys, Language::English).is_some());
        assert_eq!(header_background_color(false, Mark), Color::White);

        let config: dua::Config =
            toml::from_str("[keys]\ndelete_marked = []\ntrash_marked = []").expect("valid config");
        assert!(mark_safety_notice(false, &config.keys, Language::English).is_some());
    }
}
