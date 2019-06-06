use crate::{
    interactive::{widgets::COLOR_MARKED_DARK, EntryMarkMap},
    ByteFormat,
};
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

pub struct FooterProps<'a> {
    pub total_bytes: Option<u64>,
    pub entries_traversed: u64,
    pub format: ByteFormat,
    pub marked: Option<&'a EntryMarkMap>,
    pub message: Option<String>,
}

impl Footer {
    pub fn render<'a>(&self, props: impl Borrow<FooterProps<'a>>, area: Rect, buf: &mut Buffer) {
        let FooterProps {
            total_bytes,
            entries_traversed,
            format,
            marked,
            message,
        } = props.borrow();

        let bg_color = Color::White;
        let text_color = Color::Black;
        let lines = [
            Some(Text::Raw(
                format!(
                    " Total disk usage: {}  Entries: {}   ",
                    match total_bytes {
                        Some(b) => format!("{}", format.display(*b)).to_owned(),
                        None => "-".to_owned(),
                    },
                    entries_traversed,
                )
                .into(),
            )),
            marked.and_then(|marked| match marked.is_empty() {
                true => None,
                false => Some(Text::Styled(
                    format!(
                        "Marked {} items ({}) ",
                        marked.len(),
                        format.display(marked.iter().map(|(_k, v)| v.size).sum::<u64>())
                    )
                    .into(),
                    Style {
                        fg: COLOR_MARKED_DARK,
                        bg: bg_color,
                        modifier: Modifier::BOLD | Modifier::RAPID_BLINK,
                    },
                )),
            }),
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
