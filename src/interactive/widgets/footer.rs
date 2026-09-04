use dua::ByteFormat;
use std::borrow::Borrow;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};

use crate::interactive::{MTimeSort, SortMode, widgets::Language};

pub struct Footer;

pub struct FooterProps<'a> {
    pub total_bytes: u128,
    pub traversal_stats: Option<(u64, std::time::Instant, Option<std::time::Duration>)>,
    pub format: ByteFormat,
    pub message: Option<String>,
    pub sort_mode: SortMode,
    pub pending_exit: bool,
    pub keys: &'a dua::KeysConfig,
    pub language: Language,
}

impl Footer {
    pub fn render<'a>(props: impl Borrow<FooterProps<'a>>, area: Rect, buf: &mut Buffer) {
        let FooterProps {
            total_bytes,
            traversal_stats,
            format,
            message,
            sort_mode,
            pending_exit,
            keys,
            language,
        } = props.borrow();

        if *pending_exit {
            let quit = keys.quit.to_string();
            let exit_msg = if keys.esc_navigates_back {
                language.exit_message(&quit, None)
            } else {
                let close = keys.close_pane.to_string();
                language.exit_message(&quit, Some(&close))
            };
            Paragraph::new(Text::from(exit_msg))
                .style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .render(area, buf);
            return;
        }

        let t = language.ui_text();
        let spans = vec![
            Some(Span::from(format!(
                "{}: {}  {}: {}  ",
                t.footer_sort_mode,
                sort_mode_label(*sort_mode, *language),
                t.footer_total_disk_usage,
                format.display(*total_bytes),
            ))),
            traversal_stats
                .as_ref()
                .map(|(entries_traversed, traversal_start, elapsed)| {
                    let progress = if let Some(elapsed) = elapsed {
                        format!("{:.02}s", elapsed.as_secs_f32())
                    } else {
                        let elapsed = traversal_start.elapsed();
                        let rate = if elapsed.is_zero() {
                            0.0
                        } else {
                            *entries_traversed as f32 / elapsed.as_secs_f32()
                        };
                        format!("{:.0}s ({:.0}/s)", elapsed.as_secs_f32(), rate)
                    };
                    Span::from(language.footer_progress(*entries_traversed, &progress))
                }),
            message.as_ref().map(|m| {
                Span::styled(
                    m,
                    Style {
                        fg: Color::Yellow.into(),
                        bg: Color::Reset.into(),
                        add_modifier: Modifier::BOLD | Modifier::RAPID_BLINK,
                        ..Style::default()
                    },
                )
            }),
        ];
        Paragraph::new(Text::from(Line::from(
            spans.into_iter().flatten().collect::<Vec<_>>(),
        )))
        .style(Style::default().add_modifier(Modifier::REVERSED))
        .render(area, buf);
    }
}

fn sort_mode_label(sort_mode: SortMode, language: Language) -> String {
    use SortMode::*;
    let t = language.ui_text();
    match sort_mode {
        SizeAscending => t.sort_size_ascending.into(),
        SizeDescending => t.sort_size_descending.into(),
        MTimeAscending(sort) => modified_sort_label(t.sort_mtime_ascending, sort, language),
        MTimeDescending(sort) => modified_sort_label(t.sort_mtime_descending, sort, language),
        CountAscending => t.sort_count_ascending.into(),
        CountDescending => t.sort_count_descending.into(),
        NameAscending => t.sort_name_ascending.into(),
        NameDescending => t.sort_name_descending.into(),
    }
}

fn modified_sort_label(label: &str, mtime_sort: MTimeSort, language: Language) -> String {
    match mtime_sort_label(mtime_sort, language) {
        Some(deep_label) => format!("{label} ({deep_label})"),
        None => label.into(),
    }
}

fn mtime_sort_label(mtime_sort: MTimeSort, language: Language) -> Option<&'static str> {
    let t = language.ui_text();
    match mtime_sort {
        MTimeSort::Entry => None,
        MTimeSort::RecursiveChildrenNewest => Some(t.sort_deep_newest),
        MTimeSort::RecursiveChildrenOldest => Some(t.sort_deep_oldest),
    }
}

#[cfg(test)]
mod tests {
    use super::{Footer, FooterProps, sort_mode_label};
    use crate::interactive::{MTimeSort, SortMode, widgets::Language};
    use dua::{ByteFormat, KeysConfig};
    use std::time::{Duration, Instant};
    use tui::{
        buffer::{Buffer, Cell},
        layout::Rect,
    };

    fn rendered_footer(show_traversal_stats: bool, language: Language) -> String {
        let area = Rect::new(0, 0, 160, 1);
        let mut buffer = Buffer::empty(area);
        Footer::render(
            FooterProps {
                total_bytes: 42,
                traversal_stats: show_traversal_stats
                    .then(|| (7, Instant::now(), Some(Duration::from_millis(230)))),
                format: ByteFormat::Metric,
                message: Some("ready".into()),
                sort_mode: SortMode::SizeDescending,
                pending_exit: false,
                keys: &KeysConfig::default(),
                language,
            },
            area,
            &mut buffer,
        );
        buffer.content.iter().map(Cell::symbol).collect()
    }

    #[test]
    fn completed_traversal_stats_can_be_hidden() {
        let with_stats = rendered_footer(true, Language::English);
        let without_stats = rendered_footer(false, Language::English);
        insta::assert_debug_snapshot!(
            (with_stats.trim_end(), without_stats.trim_end()),
            "completed traversal statistics shown, then hidden",
            @r#"
        (
            "Sort mode: size, large first  Total disk usage: 42  B  Processed 7 entries in 0.23s  ready",
            "Sort mode: size, large first  Total disk usage: 42  B  ready",
        )
        "#
        );
    }

    #[test]
    fn footer_uses_the_selected_language() {
        let rendered = rendered_footer(true, Language::German);
        assert!(rendered.contains("Sortierung: Größe, groß zuerst"));
        assert!(rendered.contains("Gesamte Speicherbelegung"));
        assert!(rendered.contains("7 Einträge in 0.23s verarbeitet"));
        assert!(!rendered.contains("Sort mode"));
    }

    #[test]
    fn modified_sort_label_includes_effective_mtime_mode() {
        assert_eq!(
            sort_mode_label(
                SortMode::MTimeDescending(MTimeSort::Entry),
                Language::English
            ),
            "mtime, new first"
        );
        assert_eq!(
            sort_mode_label(
                SortMode::MTimeDescending(MTimeSort::RecursiveChildrenNewest),
                Language::English
            ),
            "mtime, new first (deep newest)"
        );
        assert_eq!(
            sort_mode_label(
                SortMode::MTimeAscending(MTimeSort::RecursiveChildrenOldest),
                Language::English
            ),
            "mtime, old first (deep oldest)"
        );
    }

    #[test]
    fn non_modified_sort_labels_describe_what_comes_first() {
        assert_eq!(
            sort_mode_label(SortMode::SizeDescending, Language::English),
            "size, large first"
        );
        assert_eq!(
            sort_mode_label(SortMode::CountAscending, Language::English),
            "items, few first"
        );
        assert_eq!(
            sort_mode_label(SortMode::NameAscending, Language::English),
            "name, A-Z"
        );
    }
}
