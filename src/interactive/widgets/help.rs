use tui::style::Color;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
};

#[derive(Copy, Clone)]
pub struct HelpPaneState;

pub struct HelpPane {
    pub state: HelpPaneState,
    pub border_style: Style,
}

impl Widget for HelpPane {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        fn title(name: &str) -> Text {
            Text::Styled(
                format!("{}\n\n", name).into(),
                Style {
                    modifier: Modifier::BOLD | Modifier::UNDERLINED,
                    ..Default::default()
                },
            )
        };
        fn hotkey(keys: &str, description: &str) -> [Text<'static>; 2] {
            [
                Text::Styled(
                    format!("{:>10}", keys).into(),
                    Style {
                        fg: Color::Green,
                        ..Default::default()
                    },
                ),
                Text::Styled(format!(" => {}\n", description).into(), Style::default()),
            ]
        };

        let mut block = Block::default()
            .title("Help")
            .border_style(self.border_style)
            .borders(Borders::ALL);
        block.draw(area, buf);
        let area = block.inner(area).inner(1);

        Paragraph::new(
            [title("Keys for Navigation")]
                .iter()
                .chain(hotkey("j", "move down an entry").iter())
                .chain(hotkey("k", "move up an entry").iter())
                .chain(hotkey("o", "descent into the selected directory").iter())
                .chain(hotkey("u", "move up one level into the parent directory").iter())
                .chain(hotkey("Ctrl + d", "move down 10 entries at once").iter())
                .chain(hotkey("Ctrl + u", "move up 10 entries at once").iter()),
        )
        .draw(area, buf);
    }
}
