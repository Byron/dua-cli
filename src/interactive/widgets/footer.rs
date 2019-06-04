use crate::ByteFormat;
use tui::widgets::{Paragraph, Text};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    style::{Color, Style},
    widgets::Widget,
};

pub struct Footer {
    pub total_bytes: Option<u64>,
    pub entries_traversed: u64,
    pub format: ByteFormat,
    pub message: Option<String>,
}

impl Widget for Footer {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        assert_eq!(area.height, 1, "The footer must be a line");
        let bg_color = Color::White;
        let text_color = Color::Black;
        let lines = [
            Some(Text::Raw(
                format!(
                    " Total disk usage: {}  Entries: {}   ",
                    match self.total_bytes {
                        Some(b) => format!("{}", self.format.display(b)).trim().to_owned(),
                        None => "-".to_owned(),
                    },
                    self.entries_traversed,
                )
                .into(),
            )),
            self.message.as_ref().map(|m| {
                Text::Styled(
                    m.into(),
                    Style {
                        fg: Color::Blue,
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
