use crate::ByteFormat;
use std::borrow::Borrow;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    style::{Color, Style},
    widgets::Widget,
    widgets::{Paragraph, Text},
};
pub struct Footer;

pub struct FooterProps {
    pub total_bytes: Option<u64>,
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

        let bg_color = Color::White;
        let text_color = Color::Black;
        let lines = [
            Text::Raw(
                format!(
                    " Total disk usage: {}  Entries: {}   ",
                    match total_bytes {
                        Some(b) => format!("{}", format.display(*b)).to_owned(),
                        None => "-".to_owned(),
                    },
                    entries_traversed,
                )
                .into(),
            )
            .into(),
            message.as_ref().map(|m| {
                Text::Styled(
                    m.into(),
                    Style {
                        fg: Color::Red,
                        bg: bg_color,
                        modifier: Modifier::BOLD | Modifier::RAPID_BLINK,
                    },
                )
            }),
        ];
        Paragraph::new(lines.iter().filter_map(|x| x.as_ref()))
            .style(Style {
                fg: text_color,
                bg: bg_color,
                ..Default::default()
            })
            .draw(area, buf);
    }
}
