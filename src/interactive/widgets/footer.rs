use crate::ByteFormat;
use std::borrow::Borrow;
use tui::widgets::{Paragraph, Text};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    style::{Color, Style},
    widgets::Widget,
};
use tui_react::ToplevelComponent;

pub struct ReactFooter;

pub struct ReactFooterProps {
    pub total_bytes: Option<u64>,
    pub entries_traversed: u64,
    pub format: ByteFormat,
    pub message: Option<String>,
}

impl ToplevelComponent for ReactFooter {
    type Props = ReactFooterProps;

    fn render(&mut self, props: impl Borrow<Self::Props>, area: Rect, buf: &mut Buffer) {
        let ReactFooterProps {
            total_bytes,
            entries_traversed,
            format,
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
