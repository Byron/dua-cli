use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use tui::style::Color;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
};

#[derive(Default, Clone)]
pub struct HelpPane {
    pub scroll: u16,
}

pub struct HelpPaneProps {
    pub border_style: Style,
}

impl HelpPane {
    pub fn render(&mut self, props: impl Borrow<HelpPaneProps>, area: Rect, buf: &mut Buffer) {
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
            let hotkey = |keys, description, other_line: Option<&str>| {
                let separator_size = 3;
                let column_size = 11 + separator_size;
                count(1 + other_line.iter().count() as u16);
                lines.borrow_mut().push(Text::Styled(
                    format!(
                        "{:>column_size$}",
                        keys,
                        column_size = column_size - separator_size
                    )
                    .into(),
                    Style {
                        fg: Color::Green,
                        ..Default::default()
                    },
                ));
                lines.borrow_mut().push(Text::Styled(
                    format!(" => {}\n", description).into(),
                    Style::default(),
                ));
                if let Some(second_line) = other_line {
                    lines.borrow_mut().push(Text::Styled(
                        format!(
                            "{:>column_size$}{}\n",
                            "",
                            second_line,
                            column_size = column_size + 1
                        )
                        .into(),
                        Style::default(),
                    ));
                }
            };

            title("Keys for pane control");
            {
                hotkey(
                    "q/<ESC>",
                    "Close the current pane. Closes the program if no",
                    Some("pane is open"),
                );
                hotkey("<tab>", "Cycle between all open panes", None);
                hotkey("?", "Show or hide the help pane", None);
                spacer();
            }
            title("Keys for Navigation");
            {
                hotkey("j/<down>", "move down an entry", None);
                hotkey("k/<up>", "move up an entry", None);
                hotkey("o/<enter>", "descent into the selected directory", None);
                hotkey("u", "ascent one level into the parent directory", None);
                hotkey("<backspace>", "^", None);
                hotkey("Ctrl + d", "move down 10 entries at once", None);
                hotkey("<Page Down>", "^", None);
                hotkey("Ctrl + u", "move up 10 entries at once", None);
                hotkey("<Page Up>", "^", None);
                spacer();
            }
            title("Keys for display");
            {
                hotkey("s", "toggle sort by size ascending/descending", None);
                hotkey(
                    "g",
                    "cycle through percentage display and bar options",
                    None,
                );
                spacer();
            }
            title("Keys for entry operations");
            {
                hotkey(
                    "Shift + o",
                    "Open the entry with the associated program",
                    None,
                );
                hotkey(
                    "d",
                    "Toggle the currently selected entry and move down",
                    None,
                );
                hotkey("<space bar>", "Toggle the currently selected entry", None);
                spacer();
            }
            title("Keys for application control");
            {
                hotkey(
                    "Ctrl + c",
                    "close the application. No questions asked!",
                    None,
                );
                spacer();
            }
            (lines.into_inner(), num_lines.get())
        };

        let HelpPaneProps { border_style } = props.borrow();

        let mut block = Block::default()
            .title("Help")
            .border_style(*border_style)
            .borders(Borders::ALL);
        block.draw(area, buf);

        let area = block.inner(area).inner(1);
        self.scroll = self.scroll.min(num_lines.saturating_sub(area.height));
        Paragraph::new(texts.iter())
            .scroll(self.scroll)
            .draw(area, buf);
    }
}
