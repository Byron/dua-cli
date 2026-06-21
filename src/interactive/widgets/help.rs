use crate::interactive::CursorDirection;
use crate::interactive::widgets::Language;
use crate::interactive::widgets::tui_ext::{
    draw_text_nowrap_fn,
    util::{block_width, rect},
};
pub use crossterm::event::KeyCode::*;
use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
use std::{borrow::Borrow, cell::RefCell};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Default, Clone)]
pub struct HelpPane {
    pub scroll: u16,
    pub language: Language,
}

pub struct HelpPaneProps {
    pub border_style: Style,
    pub has_focus: bool,
    pub esc_navigates_back: bool,
}

fn margin(r: Rect, margin: u16) -> Rect {
    Rect {
        x: r.x + margin,
        y: r.y + margin,
        width: r.width - 2 * margin,
        height: r.height - 2 * margin,
    }
}

impl HelpPane {
    pub fn with_locale_from_env() -> Self {
        HelpPane {
            language: Language::from_env(),
            ..Default::default()
        }
    }

    pub fn process_events(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        match key.code {
            Char('H') => self.scroll_help(CursorDirection::ToTop),
            Char('G') => self.scroll_help(CursorDirection::ToBottom),
            Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_help(CursorDirection::PageUp)
            }
            Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_help(CursorDirection::PageDown)
            }
            PageUp => self.scroll_help(CursorDirection::PageUp),
            PageDown => self.scroll_help(CursorDirection::PageDown),
            Char('k') | Up => self.scroll_help(CursorDirection::Up),
            Char('j') | Down => self.scroll_help(CursorDirection::Down),
            _ => {}
        };
    }
    fn scroll_help(&mut self, direction: CursorDirection) {
        self.scroll = direction.move_cursor(self.scroll as usize) as u16;
    }

    pub fn render(&mut self, props: impl Borrow<HelpPaneProps>, area: Rect, buf: &mut Buffer) {
        let esc_navigates_back = props.borrow().esc_navigates_back;
        let t = self.language.help_text();
        let lines = {
            let lines = RefCell::new(Vec::<Line<'_>>::with_capacity(30));
            let add_newlines = |n| {
                for _ in 0..n {
                    lines.borrow_mut().push(Line::from(Span::raw("")))
                }
            };

            let spacer = || add_newlines(2);
            let title = |name: &str| {
                lines.borrow_mut().push(Line::from(Span::styled(
                    name.to_string(),
                    Style {
                        add_modifier: Modifier::BOLD | Modifier::UNDERLINED,
                        ..Default::default()
                    },
                )));
                add_newlines(1);
            };
            let hotkey = |keys, description, other_line: Option<&str>| {
                let separator_size = 3;
                let column_size = 11 + separator_size;
                lines.borrow_mut().push(Line::from(vec![
                    Span::styled(
                        format!(
                            "{:>column_size$}",
                            keys,
                            column_size = column_size - separator_size
                        ),
                        Style {
                            fg: Color::Green.into(),
                            ..Default::default()
                        },
                    ),
                    Span::from(format!(" => {description}")),
                ]));
                if let Some(second_line) = other_line {
                    lines.borrow_mut().push(Line::from(Span::from(format!(
                        "{:>column_size$}{}",
                        "",
                        second_line,
                        column_size = column_size + 1
                    ))));
                }
            };

            title(t.pane_control_title);
            {
                if esc_navigates_back {
                    hotkey("q", t.pane_q_quit, None);
                    hotkey("<Esc>", t.pane_esc_close, Some(t.pane_esc_close_2));
                } else {
                    hotkey("q/<Esc>", t.pane_qesc_close, Some(t.pane_qesc_close_2));
                }
                hotkey("<Tab>", t.pane_tab, Some(t.pane_tab_2));
                hotkey("?", t.pane_help_toggle, None);
                spacer();
            }
            title(t.nav_title);
            {
                hotkey("j/<Down>", t.nav_down, None);
                hotkey("k/<Up>", t.nav_up, None);
                hotkey("o/l/<Enter>", t.nav_descend, None);
                hotkey("<Right>", "^", None);
                hotkey("u/h/<Left>", t.nav_ascend, None);
                hotkey("<Backspace>", "^", None);
                hotkey("Ctrl + d", t.nav_down10, None);
                hotkey("<Page Down>", "^", None);
                hotkey("Ctrl + u", t.nav_up10, None);
                hotkey("<Page Up>", "^", None);
                hotkey("H/<Home>", t.nav_top, None);
                hotkey("G/<End>", t.nav_bottom, None);
                spacer();
            }
            title(t.disp_title);
            {
                hotkey("s", t.disp_sort_size, None);
                hotkey("m", t.disp_sort_mtime, None);
                hotkey("M", t.disp_show_mtime, Some(t.disp_show_mtime_2));
                hotkey("c", t.disp_sort_count, None);
                hotkey("C", t.disp_show_count, None);
                hotkey("n", t.disp_sort_name, None);
                hotkey("g/S", t.disp_cycle_bar, None);
                spacer();
            }
            title(t.oms_title);
            {
                hotkey("O", t.oms_open, None);
                hotkey("d", t.oms_toggle_down, None);
                hotkey("x", t.oms_mark_down, None);
                hotkey("<Space>", t.oms_toggle, None);
                hotkey("X", t.oms_mark_cleanup, None);
                hotkey("t", t.oms_toggle_cleanup, None);
                hotkey("I", t.oms_mark_gitignored, None);
                hotkey("i", t.oms_toggle_gitignored, None);
                hotkey("a", t.oms_toggle_all, None);
                hotkey("/", t.oms_search, Some(t.oms_search_2));
                hotkey("r", t.oms_refresh_one, None);
                hotkey("R", t.oms_refresh_all, None);
                spacer();
            }
            title(t.mark_title);
            {
                hotkey("x/d/<Space>", t.mark_remove, None);
                hotkey("a", t.mark_remove_all, None);
                hotkey("Ctrl + r", t.mark_delete, Some(t.mark_delete_2));
                #[cfg(feature = "trash-move")]
                hotkey("Ctrl + t", t.mark_trash, Some(t.mark_trash_2));
                spacer();
            }
            title(t.app_title);
            {
                hotkey("Ctrl + c", t.app_quit, None);
                spacer();
            }
            lines.into_inner()
        };

        let HelpPaneProps {
            border_style,
            has_focus,
            ..
        } = props.borrow();

        let title = t.block_title;
        let block = Block::default()
            .title(title)
            .border_style(*border_style)
            .borders(Borders::ALL);
        let inner_block_area = block.inner(area);
        block.render(area, buf);

        if *has_focus {
            let help_text = " . = o|.. = u ── ⇊ = Ctrl+d|↓ = j|⇈ = Ctrl+u|↑ = k ";
            let help_text_block_width = block_width(help_text);
            let bound = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            if block_width(title) + help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    help_text,
                    |_, _, _| Style::default(),
                );
            }
        }

        let area = margin(inner_block_area, 1);
        self.scroll = self
            .scroll
            .min(lines.len().saturating_sub(area.height as usize) as u16);
        Paragraph::new(Text::from(lines))
            .scroll((self.scroll, 0))
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(language: Language) -> String {
        let area = Rect::new(0, 0, 120, 80);
        let mut buf = Buffer::empty(area);
        HelpPane {
            language,
            ..Default::default()
        }
        .render(
            HelpPaneProps {
                border_style: Style::default(),
                has_focus: false,
                esc_navigates_back: false,
            },
            area,
            &mut buf,
        );
        buf.content.iter().map(|cell| cell.symbol()).collect()
    }

    #[test]
    fn english_is_the_default_rendering() {
        let text = rendered(Language::English);
        assert!(text.contains("Help"));
        assert!(text.contains("Navigation"));
        assert!(text.contains("Ctrl + c"));
    }

    #[test]
    fn japanese_replaces_the_english_strings() {
        let en = rendered(Language::English);
        let ja = rendered(Language::Japanese);
        assert_ne!(
            en, ja,
            "The Japanese rendering differs and no longer shows the English titles, while untranslated key names stay put"
        );
        assert!(!ja.contains("Help"));
        assert!(!ja.contains("Navigation"));
        assert!(!ja.contains("Display"));
        assert!(ja.contains("Ctrl + c"));

        // The backend pads wide glyphs with a trailing cell, so collapse whitespace before matching.
        let ja_collapsed: String = ja.split_whitespace().collect();
        assert!(
            ja_collapsed.contains("ヘルプ"),
            "The Japanese strings are actually rendered."
        );
        assert!(ja_collapsed.contains("ナビゲーション"));
    }
}
