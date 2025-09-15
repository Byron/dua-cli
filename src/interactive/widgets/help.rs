use crate::interactive::CursorDirection;
pub use crosstermion::crossterm::event::KeyCode::*;
use crosstermion::crossterm::event::{KeyEventKind, KeyModifiers};
use crosstermion::input::Key;
use std::{borrow::Borrow, cell::RefCell};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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
        if key.kind == KeyEventKind::Release {
            return;
        }
        match key.code {
            Char('H') => self.scroll_help(CursorDirection::ToTop),
            Char('G') => self.scroll_help(CursorDirection::ToBottom),
            Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_help(CursorDirection::PageUp)
            }
            Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_help(CursorDirection::PageDown)
            }
            PageUp => self.scroll_help(CursorDirection::PageUp),
            PageDown => self.scroll_help(CursorDirection::PageDown),
            Char('k') | Up => self.scroll_help(CursorDirection::Up),
            Char('j') | Down => self.scroll_help(CursorDirection::Down),
            _ => {}
        };
    }
    fn scroll_help(&mut self, direction: CursorDirection) {
        self.scroll = direction.move_cursor(self.scroll as usize) as u16;
    }

    pub fn render(&mut self, props: impl Borrow<HelpPaneProps>, area: Rect, buf: &mut Buffer) {
        let lines = {
            let lines = RefCell::new(Vec::<Line<'_>>::with_capacity(30));
            let add_newlines = |n| {
                for _ in 0..n {
                    lines.borrow_mut().push(Line::from(Span::raw("")))
                }
            };

            let spacer = || add_newlines(2);
            let title = |name: &str| {
                lines.borrow_mut().push(Line::from(Span::styled(
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
                lines.borrow_mut().push(Line::from(vec![
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
                    Span::from(format!(" => {description}")),
                ]));
                if let Some(second_line) = other_line {
                    lines.borrow_mut().push(Line::from(Span::from(format!(
                        "{:>column_size$}{}",
                        "",
                        second_line,
                        column_size = column_size + 1
                    ))));
                }
            };

            title("Pane control");
            {
                hotkey(
                    "q/<Esc>",
                    "Close the current pane.",
                    Some("Closes the program if no pane is open."),
                );
                hotkey(
                    "<Tab>",
                    "Cycle between all open panes.",
                    Some("Activate 'Marked Items' pane to delete selected files."),
                );
                hotkey("?", "Show or hide this help pane.", None);
                spacer();
            }
            title("Navigation");
            {
                hotkey("j/<Down>", "Move down 1 entry.", None);
                hotkey("k/<Up>", "Move up 1 entry.", None);
                hotkey("o/l/<Enter>", "Descent into the selected directory.", None);
                hotkey("<Right>", "^", None);
                hotkey(
                    "u/h/<Left>",
                    "Ascent one level into the parent directory.",
                    None,
                );
                hotkey("<Backspace>", "^", None);
                hotkey("Ctrl + d", "Move down 10 entries.", None);
                hotkey("<Page Down>", "^", None);
                hotkey("Ctrl + u", "Move up 10 entries.", None);
                hotkey("<Page Up>", "^", None);
                hotkey("H/<Home>", "Move to the top of the list.", None);
                hotkey("G/<End>", "Move to the bottom of the list.", None);
                spacer();
            }
            title("Display");
            {
                hotkey("s", "Toggle sort by size descending/ascending.", None);
                hotkey(
                    "m",
                    "Toggle sort by modified time descending/ascending.",
                    None,
                );
                hotkey("M", "Show/hide modified time.", None);
                hotkey("c", "Toggle sort by entries descending/ascending.", None);
                hotkey("C", "Show/hide entry count.", None);
                hotkey("n", "Toggle sort by name ascending/descending.", None);
                hotkey(
                    "g/S",
                    "Cycle through percentage display and bar options.",
                    None,
                );
                spacer();
            }
            title("Open/Mark/Search");
            {
                hotkey(
                    "Shift + o",
                    "Open the selected entry with the associated program.",
                    None,
                );
                hotkey(
                    "d",
                    "Toggle the currently selected entry and move down.",
                    None,
                );
                hotkey(
                    "x",
                    "Mark the currently selected entry for deletion and move down.",
                    None,
                );
                hotkey("<Space>", "Toggle the currently selected entry.", None);
                hotkey("a", "Toggle all entries.", None);
                hotkey(
                    "/",
                    "Git-style glob search. Toggle case with 'I'.",
                    Some("Search starts from the current directory."),
                );
                hotkey("r", "Refresh only the selected entry.", None);
                hotkey("R", "Refresh all entries in the current view.", None);
                spacer();
            }
            title("Mark entries pane");
            {
                hotkey(
                    "x/d/<Space>",
                    "Remove the selected entry from the list.",
                    None,
                );
                hotkey("a", "Remove all entries from the list.", None);
                hotkey(
                    "Ctrl + r",
                    "Permanently delete all marked entries without prompt.",
                    Some("This operation cannot be undone!"),
                );
                #[cfg(feature = "trash-move")]
                hotkey(
                    "Ctrl + t",
                    "Move all marked entries to the trash bin.",
                    Some("The entries can be restored from the trash bin."),
                );
                spacer();
            }
            title("Application control");
            {
                hotkey(
                    "Ctrl + c",
                    "Close the application. No questions asked!",
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
            let help_text = " . = o|.. = u ── ⇊ = Ctrl+d|↓ = j|⇈ = Ctrl+u|↑ = k ";
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
