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

pub struct FooterProps {
    pub total_bytes: u128,
    pub entries_traversed: u64,
    pub traversal_start: std::time::Instant,
    pub elapsed: Option<std::time::Duration>,
    pub format: ByteFormat,
    pub message: Option<String>,
    pub sort_mode: SortMode,
    pub pending_exit: bool,
    pub esc_navigates_back: bool,
}

impl Footer {
    pub fn render(&self, props: impl Borrow<FooterProps>, area: Rect, buf: &mut Buffer) {
        let FooterProps {
            total_bytes,
            entries_traversed,
            elapsed,
            traversal_start,
            format,
            message,
            sort_mode,
            pending_exit,
            esc_navigates_back,
        } = props.borrow();

        if *pending_exit {
            let exit_msg = if *esc_navigates_back {
                "Press q again to exit..."
            } else {
                "Press esc or q again to exit..."
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
            Span::from(format!(
                "Sort mode: {}  Total disk usage: {}  Processed {} entries {progress}  ",
                sort_mode_label(*sort_mode),
                format.display(*total_bytes),
                entries_traversed,
                progress = match elapsed {
                    Some(elapsed) => format!("in {:.02}s", elapsed.as_secs_f32()),
                    None => {
                        let elapsed = traversal_start.elapsed();
                        let rate = if elapsed.is_zero() {
                            0.0
                        } else {
                            *entries_traversed as f32 / elapsed.as_secs_f32()
                        };
                        format!("in {:.0}s ({:.0}/s)", elapsed.as_secs_f32(), rate)
                    }
                }
            ))
            .into(),
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
    use super::sort_mode_label;
    use crate::interactive::{MTimeSort, SortMode};

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
