use dua::ByteFormat;
use std::borrow::Borrow;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Paragraph, Widget},
};

pub struct Footer;

pub struct FooterProps {
    pub total_bytes: Option<u128>,
    pub entries_traversed: u64,
    pub format: ByteFormat,
    pub message: Option<String>,
}

impl Footer {
    pub fn render(&self, props: impl Borrow<FooterProps>, area: Rect, buf: &mut Buffer) {
        let FooterProps {
            total_bytes,
            entries_traversed,
            format,
            message,
        } = props.borrow();

        let spans = vec![
            Span::from(format!(
                " Total disk usage: {}  Entries: {}   ",
                match total_bytes {
                    Some(b) => format!("{}", format.display(*b)),
                    None => "-".to_owned(),
                },
                entries_traversed,
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
        Paragraph::new(Text::from(Spans::from(
            spans.into_iter().flatten().collect::<Vec<_>>(),
        )))
        .style(Style::default().add_modifier(Modifier::REVERSED))
        .render(area, buf);
    }
}
