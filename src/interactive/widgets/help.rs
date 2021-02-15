use crate::interactive::CursorDirection;
use crosstermion::{input::Key, input::Key::*};
use std::{borrow::Borrow, cell::RefCell};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_react::{
    draw_text_nowrap_fn,
    util::{block_width, rect},
};

#[derive(Default, Clone)]
pub struct HelpPane {
    pub scroll: u16,
}

pub struct HelpPaneProps {
    pub border_style: Style,
    pub has_focus: bool,
}

fn margin(r: Rect, margin: u16) -> Rect {
    Rect {
        x: r.x + margin,
        y: r.y + margin,
        width: r.width - 2 * margin,
        height: r.height - 2 * margin,
    }
}

impl HelpPane {
    pub fn process_events(&mut self, key: Key) {
        match key {
            Char('H') => self.scroll_help(CursorDirection::ToTop),
            Char('G') => self.scroll_help(CursorDirection::ToBottom),
            Ctrl('u') | PageUp => self.scroll_help(CursorDirection::PageUp),
            Char('k') | Up => self.scroll_help(CursorDirection::Up),
            Char('j') | Down => self.scroll_help(CursorDirection::Down),
            Ctrl('d') | PageDown => self.scroll_help(CursorDirection::PageDown),
            _ => {}
        };
    }
    fn scroll_help(&mut self, direction: CursorDirection) {
        self.scroll = direction.move_cursor(self.scroll as usize) as u16;
    }

    pub fn render(&mut self, props: impl Borrow<HelpPaneProps>, area: Rect, buf: &mut Buffer) {
        let lines = {
            let lines = RefCell::new(Vec::<Spans>::with_capacity(30));
            let add_newlines = |n| {
                for _ in 0..n {
                    lines.borrow_mut().push(Spans::from(Span::raw("")))
                }
            };

            let spacer = || add_newlines(2);
            let title = |name: &str| {
                lines.borrow_mut().push(Spans::from(Span::styled(
                    name.to_string(),
                    Style {
                        add_modifier: Modifier::BOLD | Modifier::UNDERLINED,
                        ..Default::default()
                    },
                )));
                add_newlines(1);
            };
            let hotkey = |keys, description, other_line: Option<&str>| {
                let separator_size = 3;
                let column_size = 11 + separator_size;
                lines.borrow_mut().push(Spans::from(vec![
                    Span::styled(
                        format!(
                            "{:>column_size$}",
                            keys,
                            column_size = column_size - separator_size
                        ),
                        Style {
                            fg: Color::Green.into(),
                            ..Default::default()
                        },
                    ),
                    Span::from(format!(" => {}", description)),
                ]));
                if let Some(second_line) = other_line {
                    lines.borrow_mut().push(Spans::from(Span::from(format!(
                        "{:>column_size$}{}",
                        "",
                        second_line,
                        column_size = column_size + 1
                    ))));
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
                hotkey("o/l/<enter>", "descent into the selected directory", None);
                hotkey("<right>", "^", None);
                hotkey(
                    "u/h/<left>",
                    "ascent one level into the parent directory",
                    None,
                );
                hotkey("<backspace>", "^", None);
                hotkey("Ctrl + d", "move down 10 entries at once", None);
                hotkey("<Page Down>", "^", None);
                hotkey("Ctrl + u", "move up 10 entries at once", None);
                hotkey("<Page Up>", "^", None);
                hotkey("H", "Move to the top of the entries list", None);
                hotkey("G", "Move to the bottomw of the entries list", None);
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
                hotkey(
                    "x",
                    "Mark for the currently selected entry for deletion and move down",
                    None,
                );
                hotkey("<space bar>", "Toggle the currently selected entry", None);
                spacer();
            }
            title("Keys in the Mark pane");
            {
                hotkey(
                    "x/d/<space>",
                    "remove the selected entry from the list",
                    None,
                );
                hotkey(
                    "Ctrl + r",
                    "Permanently delete all marked entries without prompt!",
                    Some("This operation cannot be undone!"),
                );
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
            lines.into_inner()
        };

        let HelpPaneProps {
            border_style,
            has_focus,
        } = props.borrow();

        let title = "Help";
        let block = Block::default()
            .title(title)
            .border_style(*border_style)
            .borders(Borders::ALL);
        let inner_block_area = block.inner(area);
        block.render(area, buf);

        if *has_focus {
            let help_text = " . = o|.. = u || ⇊ = CTRL+d|↓ = j|⇈ = CTRL+u|↑ = k ";
            let help_text_block_width = block_width(help_text);
            let bound = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            if block_width(title) + help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    help_text,
                    |_, _, _| Style::default(),
                );
            }
        }

        let area = margin(inner_block_area, 1);
        self.scroll = self
            .scroll
            .min(lines.len().saturating_sub(area.height as usize) as u16);
        Paragraph::new(Text::from(lines))
            .scroll((self.scroll, 0))
            .render(area, buf);
    }
}
