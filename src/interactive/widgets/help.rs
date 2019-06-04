use std::cell::{Cell, RefCell};
use tui::style::Color;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
};

#[derive(Default, Copy, Clone)]
pub struct HelpPaneState;

pub struct HelpPane {
    pub state: HelpPaneState,
    pub scroll: u16,
    pub border_style: Style,
}

impl Widget for HelpPane {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let (texts, num_lines) = {
            let num_lines = Cell::new(0u16);
            let count = |n| num_lines.set(num_lines.get() + n);
            let lines = RefCell::new(Vec::with_capacity(30));

            let spacer = || {
                count(2);
                lines.borrow_mut().push(Text::Raw("\n\n".into()));
            };
            let title = |name| {
                count(2);
                lines.borrow_mut().push(Text::Styled(
                    format!("{}\n\n", name).into(),
                    Style {
                        modifier: Modifier::BOLD | Modifier::UNDERLINED,
                        ..Default::default()
                    },
                ));
            };
            let hotkey = |keys, description| {
                count(1);
                lines.borrow_mut().push(Text::Styled(
                    format!("{:>10}", keys).into(),
                    Style {
                        fg: Color::Green,
                        ..Default::default()
                    },
                ));
                lines.borrow_mut().push(Text::Styled(
                    format!(" => {}\n", description).into(),
                    Style::default(),
                ));
            };

            title("Keys for pane control");
            {
                hotkey(
                    "q",
                    "close the current pain. Closes the application if no pane is open.",
                );
                hotkey("<tab>", "Cycle between all open panes");
                hotkey("?", "Show the help pane");
                spacer();
            }
            title("Keys for Navigation");
            {
                hotkey("j", "move down an entry");
                hotkey("k", "move up an entry");
                hotkey("o", "descent into the selected directory");
                hotkey("u", "ascent one level into the parent directory");
                hotkey("Ctrl + d", "move down 10 entries at once");
                hotkey("Ctrl + u", "move up 10 entries at once");
                spacer();
            }
            title("Keys for sorting");
            {
                hotkey("s", "toggle sort by size ascending/descending");
                spacer();
            }
            title("Keys for entry operations");
            {
                hotkey("Shift + o", "Open the entry with the associated program");
                spacer();
            }
            title("Keys for application control");
            {
                hotkey("Ctrl + c", "close the application. No questions asked!");
                spacer();
            }
            (lines.into_inner(), num_lines.get())
        };

        let mut block = Block::default()
            .title("Help")
            .border_style(self.border_style)
            .borders(Borders::ALL);
        block.draw(area, buf);

        let area = block.inner(area).inner(1);
        self.scroll = self.scroll.min(num_lines.saturating_sub(area.height));
        Paragraph::new(texts.iter())
            .scroll(self.scroll)
            .draw(area, buf);
    }
}
