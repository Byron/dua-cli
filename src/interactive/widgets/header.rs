use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};

pub struct Header;

impl Header {
    pub fn render(&self, bg_color: Color, area: Rect, buf: &mut Buffer) {
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
            italic("(press "),
            modified("?", Modifier::BOLD | Modifier::UNDERLINED),
            italic(" for help)"),
        ];
        Paragraph::new(Text::from(Line::from(spans)))
            .style(Style {
                bg: bg_color.into(),
                ..Default::default()
            })
            .render(area, buf);
    }
}
