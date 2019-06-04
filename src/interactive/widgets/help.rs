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
                    modifier: Modifier::BOLD,
                    ..Default::default()
                },
            )
        };
        fn hotkey(keys: &str, description: &str) -> Text<'static> {
            Text::Styled(
                format!("{} => {}\n", keys, description).into(),
                Style {
                    ..Default::default()
                },
            )
        };

        let mut block = Block::default()
            .title("Help")
            .border_style(self.border_style)
            .borders(Borders::ALL);
        block.draw(area, buf);
        let area = block.inner(area).inner(1);

        Paragraph::new([title("Hotkeys"), hotkey("j", "move down")].iter()).draw(area, buf);
    }
}
