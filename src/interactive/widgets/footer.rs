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

use crate::interactive::SortMode;

pub struct Footer;

pub struct FooterProps {
    pub total_bytes: u128,
    pub entries_traversed: u64,
    pub traversal_start: std::time::Instant,
    pub elapsed: Option<std::time::Duration>,
    pub format: ByteFormat,
    pub message: Option<String>,
    pub sort_mode: SortMode,
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
        } = props.borrow();

        let spans = vec![
            Span::from(format!(
                "Sort mode: {}  Total disk usage: {}  Processed {} entries {progress}  ",
                match sort_mode {
                    SortMode::SizeAscending => "size ascending",
                    SortMode::SizeDescending => "size descending",
                    SortMode::MTimeAscending => "modified ascending",
                    SortMode::MTimeDescending => "modified descending",
                    SortMode::CountAscending => "items ascending",
                    SortMode::CountDescending => "items descending",
                    SortMode::NameAscending => "name ascending",
                    SortMode::NameDescending => "name descending",
                },
                format.display(*total_bytes),
                entries_traversed,
                progress = match elapsed {
                    Some(elapsed) => format!("in {:.02}s", elapsed.as_secs_f32()),
                    None => {
                        let elapsed = traversal_start.elapsed();
                        format!(
                            "in {:.0}s ({:.0}/s)",
                            elapsed.as_secs_f32(),
                            *entries_traversed as f32 / elapsed.as_secs_f32()
                        )
                    }
                }
            ))
            .into(),
            message.as_ref().map(|m| {
                Span::styled(
                    m,
                    Style {
                        fg: Color::Red.into(),
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
