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

use crate::interactive::{MTimeSort, SortMode};

pub struct Footer;

pub struct FooterProps<'a> {
    pub total_bytes: u128,
    pub traversal_stats: Option<(u64, std::time::Instant, Option<std::time::Duration>)>,
    pub format: ByteFormat,
    pub message: Option<String>,
    pub sort_mode: SortMode,
    pub pending_exit: bool,
    pub keys: &'a dua::KeysConfig,
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
        } = props.borrow();

        if *pending_exit {
            let exit_msg = if keys.esc_navigates_back {
                format!("Press {} again to exit...", keys.quit)
            } else {
                format!(
                    "Press {} or {} again to exit...",
                    keys.close_pane, keys.quit
                )
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

        let spans = vec![
            Some(Span::from(format!(
                "Sort mode: {}  Total disk usage: {}  ",
                sort_mode_label(*sort_mode),
                format.display(*total_bytes),
            ))),
            traversal_stats
                .as_ref()
                .map(|(entries_traversed, traversal_start, elapsed)| {
                    let progress = if let Some(elapsed) = elapsed {
                        format!("in {:.02}s", elapsed.as_secs_f32())
                    } else {
                        let elapsed = traversal_start.elapsed();
                        let rate = if elapsed.is_zero() {
                            0.0
                        } else {
                            *entries_traversed as f32 / elapsed.as_secs_f32()
                        };
                        format!("in {:.0}s ({:.0}/s)", elapsed.as_secs_f32(), rate)
                    };
                    Span::from(format!(
                        "Processed {entries_traversed} entries {progress}  "
                    ))
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

fn sort_mode_label(sort_mode: SortMode) -> String {
    use SortMode::*;
    match sort_mode {
        SizeAscending => "size, small first".into(),
        SizeDescending => "size, large first".into(),
        MTimeAscending(sort) => modified_sort_label("old first", sort),
        MTimeDescending(sort) => modified_sort_label("new first", sort),
        CountAscending => "items, few first".into(),
        CountDescending => "items, most first".into(),
        NameAscending => "name, A-Z".into(),
        NameDescending => "name, Z-A".into(),
    }
}

fn modified_sort_label(order: &'static str, mtime_sort: MTimeSort) -> String {
    match mtime_sort_label(mtime_sort) {
        Some(label) => format!("mtime, {order} ({label})"),
        None => format!("mtime, {order}"),
    }
}

fn mtime_sort_label(mtime_sort: MTimeSort) -> Option<&'static str> {
    match mtime_sort {
        MTimeSort::Entry => None,
        MTimeSort::RecursiveChildrenNewest => Some("deep newest"),
        MTimeSort::RecursiveChildrenOldest => Some("deep oldest"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Footer, FooterProps, sort_mode_label};
    use crate::interactive::{MTimeSort, SortMode};
    use dua::{ByteFormat, KeysConfig};
    use std::time::{Duration, Instant};
    use tui::{
        buffer::{Buffer, Cell},
        layout::Rect,
    };

    fn rendered_footer(show_traversal_stats: bool) -> String {
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
            },
            area,
            &mut buffer,
        );
        buffer.content.iter().map(Cell::symbol).collect()
    }

    #[test]
    fn completed_traversal_stats_can_be_hidden() {
        let with_stats = rendered_footer(true);
        let without_stats = rendered_footer(false);
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
    fn modified_sort_label_includes_effective_mtime_mode() {
        assert_eq!(
            sort_mode_label(SortMode::MTimeDescending(MTimeSort::Entry)),
            "mtime, new first"
        );
        assert_eq!(
            sort_mode_label(SortMode::MTimeDescending(
                MTimeSort::RecursiveChildrenNewest
            )),
            "mtime, new first (deep newest)"
        );
        assert_eq!(
            sort_mode_label(SortMode::MTimeAscending(MTimeSort::RecursiveChildrenOldest)),
            "mtime, old first (deep oldest)"
        );
    }

    #[test]
    fn non_modified_sort_labels_describe_what_comes_first() {
        assert_eq!(
            sort_mode_label(SortMode::SizeDescending),
            "size, large first"
        );
        assert_eq!(
            sort_mode_label(SortMode::CountAscending),
            "items, few first"
        );
        assert_eq!(sort_mode_label(SortMode::NameAscending), "name, A-Z");
    }
}
