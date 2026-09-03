use crate::interactive::CursorDirection;
use crate::interactive::widgets::Language;
use crate::interactive::widgets::tui_ext::{
    draw_text_nowrap_fn,
    util::{block_width, rect},
};
use crossterm::event::{KeyEvent, KeyEventKind};
use dua::KeysConfig;
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

pub struct HelpPaneProps<'a> {
    pub border_style: Style,
    pub has_focus: bool,
    pub keys: &'a KeysConfig,
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

    pub fn process_events(&mut self, key: KeyEvent, keys: &KeysConfig) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        let direction = if keys.move_to_top.matches(key) {
            CursorDirection::ToTop
        } else if keys.move_to_bottom.matches(key) {
            CursorDirection::ToBottom
        } else if keys.page_up.matches(key) {
            CursorDirection::PageUp
        } else if keys.page_down.matches(key) {
            CursorDirection::PageDown
        } else if keys.move_up.matches(key) {
            CursorDirection::Up
        } else if keys.move_down.matches(key) {
            CursorDirection::Down
        } else {
            return;
        };
        self.scroll_help(direction);
    }
    fn scroll_help(&mut self, direction: CursorDirection) {
        self.scroll = direction.move_cursor(self.scroll as usize) as u16;
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "scroll coordinates are bounded by terminal areas"
    )]
    pub fn render<'a>(
        &mut self,
        props: impl Borrow<HelpPaneProps<'a>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let props = props.borrow();
        let keys = props.keys;
        let t = self.language.help_text();
        let build_lines = || {
            let lines = RefCell::new(Vec::<Line<'_>>::with_capacity(30));
            let add_newlines = |n| {
                for _ in 0..n {
                    lines.borrow_mut().push(Line::from(Span::raw("")));
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
            let hotkey = |keys: String, description, other_line: Option<&str>| {
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
                if keys.esc_navigates_back {
                    hotkey(keys.quit.to_string(), t.pane_q_quit, None);
                    hotkey(
                        keys.close_pane.to_string(),
                        t.pane_esc_close,
                        Some(t.pane_esc_close_2),
                    );
                } else {
                    hotkey(
                        format!("{}/{}", keys.quit, keys.close_pane),
                        t.pane_qesc_close,
                        Some(t.pane_qesc_close_2),
                    );
                }
                hotkey(keys.cycle_panes.to_string(), t.pane_tab, Some(t.pane_tab_2));
                hotkey(keys.toggle_help.to_string(), t.pane_help_toggle, None);
                spacer();
            }
            title(t.nav_title);
            {
                hotkey(keys.move_down.to_string(), t.nav_down, None);
                hotkey(keys.move_up.to_string(), t.nav_up, None);
                hotkey(keys.descend.to_string(), t.nav_descend, None);
                hotkey(keys.ascend.to_string(), t.nav_ascend, None);
                hotkey(keys.page_down.to_string(), t.nav_down10, None);
                hotkey(keys.page_up.to_string(), t.nav_up10, None);
                hotkey(keys.move_to_top.to_string(), t.nav_top, None);
                hotkey(keys.move_to_bottom.to_string(), t.nav_bottom, None);
                spacer();
            }
            title(t.disp_title);
            {
                hotkey(keys.sort_by_size.to_string(), t.disp_sort_size, None);
                hotkey(keys.sort_by_mtime.to_string(), t.disp_sort_mtime, None);
                hotkey(
                    keys.cycle_mtime_mode.to_string(),
                    t.disp_show_mtime,
                    Some(t.disp_show_mtime_2),
                );
                hotkey(keys.sort_by_count.to_string(), t.disp_sort_count, None);
                hotkey(
                    keys.toggle_count_column.to_string(),
                    t.disp_show_count,
                    None,
                );
                hotkey(keys.sort_by_name.to_string(), t.disp_sort_name, None);
                hotkey(keys.cycle_visualization.to_string(), t.disp_cycle_bar, None);
                spacer();
            }
            title(t.oms_title);
            {
                hotkey(keys.open_entry.to_string(), t.oms_open, None);
                hotkey(
                    keys.toggle_mark_and_move_down.to_string(),
                    t.oms_toggle_down,
                    None,
                );
                hotkey(keys.mark_for_deletion.to_string(), t.oms_mark_down, None);
                hotkey(keys.toggle_mark.to_string(), t.oms_toggle, None);
                hotkey(keys.mark_cleanup.to_string(), t.oms_mark_cleanup, None);
                hotkey(keys.toggle_cleanup.to_string(), t.oms_toggle_cleanup, None);
                hotkey(keys.mark_gitignore.to_string(), t.oms_mark_gitignored, None);
                hotkey(
                    keys.toggle_gitignore.to_string(),
                    t.oms_toggle_gitignored,
                    None,
                );
                hotkey(keys.toggle_all.to_string(), t.oms_toggle_all, None);
                hotkey(
                    keys.open_search.to_string(),
                    t.oms_search,
                    Some(t.oms_search_2),
                );
                hotkey(keys.refresh_selected.to_string(), t.oms_refresh_one, None);
                hotkey(keys.refresh_all.to_string(), t.oms_refresh_all, None);
                spacer();
            }
            title(t.mark_title);
            {
                hotkey(keys.remove_mark.to_string(), t.mark_remove, None);
                hotkey(keys.remove_all_marks.to_string(), t.mark_remove_all, None);
                hotkey(
                    keys.delete_marked.to_string(),
                    t.mark_delete,
                    Some(t.mark_delete_2),
                );
                #[cfg(feature = "trash-move")]
                hotkey(
                    keys.trash_marked.to_string(),
                    t.mark_trash,
                    Some(t.mark_trash_2),
                );
                spacer();
            }
            title(t.app_title);
            {
                #[cfg(unix)]
                hotkey(keys.suspend.to_string(), t.app_suspend, None);
                hotkey(keys.repaint.to_string(), t.app_repaint, None);
                hotkey(keys.quit_immediately.to_string(), t.app_quit, None);
                spacer();
            }
            lines.into_inner()
        };
        let lines = build_lines();

        let border_style = props.border_style;
        let has_focus = props.has_focus;

        let title = t.block_title;
        let block = Block::default()
            .title(title)
            .border_style(border_style)
            .borders(Borders::ALL);
        let inner_block_area = block.inner(area);
        block.render(area, buf);

        if has_focus {
            let help_text = format!(
                " ⇊ = {}|↓ = {}|⇈ = {}|↑ = {} ",
                keys.page_down.primary(),
                keys.move_down.primary(),
                keys.page_up.primary(),
                keys.move_up.primary()
            );
            let help_text_block_width = block_width(&help_text);
            let bound = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            if block_width(title) + help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    &help_text,
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
    use crossterm::event::KeyCode;
    use tui::buffer::Cell;

    fn rendered(language: Language) -> String {
        rendered_with_keys(language, &KeysConfig::default())
    }

    fn rendered_with_keys(language: Language, keys: &KeysConfig) -> String {
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
                keys,
            },
            area,
            &mut buf,
        );
        buf.content.iter().map(Cell::symbol).collect()
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

    #[test]
    fn configured_keybindings_are_rendered() {
        let config: dua::Config = toml::from_str(
            r#"
            [keys]
            quit_immediately = ["alt+x"]
            suspend = ["alt+z"]
            repaint = ["ctrl+r"]
            descend = ["f"]
            delete_marked = []
            "#,
        )
        .expect("valid config");

        let text = rendered_with_keys(Language::English, &config.keys);
        assert!(text.contains("Alt + x"));
        #[cfg(unix)]
        assert!(
            text.contains("Alt + z => Suspend the application and return control to the shell.")
        );
        assert!(text.contains("Ctrl + r => Clear and repaint the screen."));
        assert!(text.contains("f => Descent"));
        assert!(text.contains("<unmapped> => Permanently delete all marked entries"));
        assert!(!text.contains("Ctrl + c"));
    }

    #[test]
    fn configured_keybindings_scroll_help() {
        let config: dua::Config = toml::from_str(
            r#"
            [keys]
            move_to_top = ["1"]
            move_to_bottom = ["2"]
            page_up = ["3"]
            page_down = ["4"]
            move_up = ["5"]
            move_down = ["6"]
            "#,
        )
        .expect("valid config");

        for (key, expected) in [
            ('1', 0),
            ('2', u16::MAX),
            ('3', 40),
            ('4', 60),
            ('5', 49),
            ('6', 51),
        ] {
            let mut pane = HelpPane {
                scroll: 50,
                ..Default::default()
            };
            pane.process_events(KeyCode::Char(key).into(), &config.keys);
            assert_eq!(pane.scroll, expected, "configured binding {key}");
        }
    }
}
