use tui::style::Color;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
};

#[derive(Default, Copy, Clone)]
pub struct HelpPaneState {
    pub scroll: u16,
}

pub struct HelpPane {
    pub state: HelpPaneState,
    pub border_style: Style,
}

impl Widget for HelpPane {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        fn spacer() -> [Text<'static>; 1] {
            [Text::Raw("\n\n".into())]
        };
        fn title(name: &str) -> [Text<'static>; 1] {
            [Text::Styled(
                format!("{}\n\n", name).into(),
                Style {
                    modifier: Modifier::BOLD | Modifier::UNDERLINED,
                    ..Default::default()
                },
            )]
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
            title("Keys for pane control")
                .iter()
                .chain(
                    hotkey(
                        "q",
                        "close the current pain. Closes the application if no pane is open.",
                    )
                    .iter(),
                )
                .chain(hotkey("<tab>", "Cycle between all open panes").iter())
                .chain(hotkey("?", "Show the help pane").iter())
                .chain(spacer().iter())
                .chain(
                    title("Keys for Navigation")
                        .iter()
                        .chain(hotkey("j", "move down an entry").iter())
                        .chain(hotkey("k", "move up an entry").iter())
                        .chain(hotkey("o", "descent into the selected directory").iter())
                        .chain(hotkey("u", "ascent one level into the parent directory").iter())
                        .chain(hotkey("Ctrl + d", "move down 10 entries at once").iter())
                        .chain(hotkey("Ctrl + u", "move up 10 entries at once").iter())
                        .chain(spacer().iter()),
                )
                .chain(
                    title("Keys for sorting")
                        .iter()
                        .chain(hotkey("s", "toggle sort by size ascending/descending").iter())
                        .chain(spacer().iter()),
                )
                .chain(
                    title("Keys for entry operations")
                        .iter()
                        .chain(
                            hotkey("Shift + o", "Open the entry with the associated program")
                                .iter(),
                        )
                        .chain(spacer().iter()),
                )
                .chain(
                    title("Keys for application control")
                        .iter()
                        .chain(
                            hotkey("Ctrl + c", "close the application. No questions asked!").iter(),
                        )
                        .chain(spacer().iter()),
                ),
        )
        .scroll(self.state.scroll)
        .draw(area, buf);
    }
}
