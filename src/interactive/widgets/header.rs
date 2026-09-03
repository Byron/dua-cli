use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};

use super::Language;

pub struct Header;

impl Header {
    pub fn render(language: Language, bg_color: Color, area: Rect, buf: &mut Buffer) {
        let t = language.ui_text();
        let standard = Style {
            fg: Color::Black.into(),
            bg: bg_color.into(),
            ..Default::default()
        };
        debug_assert_ne!(standard.bg, standard.fg);
        let modified = |text: &'static str, modifier| {
            Span::styled(
                text,
                Style {
                    add_modifier: modifier,
                    ..standard
                },
            )
        };
        let bold = |text: &'static str| modified(text, Modifier::BOLD);
        let italic = |text: &'static str| modified(text, Modifier::UNDERLINED);
        let text = |text: &'static str| Span::styled(text, standard);

        let spans = vec![
            bold(" D"),
            text("isk "),
            bold("U"),
            text("sage "),
            bold("A"),
            text("nalyzer v"),
            text(env!("CARGO_PKG_VERSION")),
            text("    "),
            italic(t.header_help_before_key),
            modified("?", Modifier::BOLD | Modifier::UNDERLINED),
            italic(t.header_help_after_key),
        ];
        Paragraph::new(Text::from(Line::from(spans)))
            .style(Style {
                bg: bg_color.into(),
                ..Default::default()
            })
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui::buffer::Cell;

    #[test]
    fn localizes_only_the_help_prompt() {
        let area = Rect::new(0, 0, 80, 1);
        let mut buffer = Buffer::empty(area);
        Header::render(Language::Chinese, Color::White, area, &mut buffer);
        let rendered: String = buffer.content.iter().map(Cell::symbol).collect();
        let rendered: String = rendered.split_whitespace().collect();

        assert!(rendered.contains("DiskUsageAnalyzer"));
        assert!(rendered.contains("按"));
        assert!(rendered.contains("查看帮助"));
        assert!(!rendered.contains("press"));
    }
}
