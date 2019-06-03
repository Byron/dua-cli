use super::{DisplayOptions, Traversal, Tree, TreeIndex};
use crate::{sorted_entries, ByteFormat};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::{
    buffer::Buffer,
    layout::{Corner, Rect},
    widgets::{Block, Borders, List, Text, Widget},
};

pub struct Entries<'a> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub sorting: SortMode,
    pub selected: Option<TreeIndex>,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    SizeDescending,
    SizeAscending,
}

impl SortMode {
    pub fn toggle_size(&mut self) {
        use SortMode::*;
        *self = match self {
            SizeAscending => SizeDescending,
            SizeDescending => SizeAscending,
        }
    }
}

impl Default for SortMode {
    fn default() -> Self {
        SortMode::SizeDescending
    }
}

pub struct DisplayState {
    pub root: TreeIndex,
    pub selected: Option<TreeIndex>,
    pub sorting: SortMode,
}

pub struct MainWindow<'a, 'b> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'b DisplayState,
}

pub struct Footer {
    pub total_bytes: Option<u64>,
    pub entries_traversed: u64,
    pub format: ByteFormat,
}

impl Widget for Footer {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        assert!(area.height == 1, "The footer must be a line");
        let bg_color = Color::White;
        let text_color = Color::Black;
        let margin = 1;
        self.background(area, buf, bg_color);
        buf.set_stringn(
            area.x + margin,
            area.y,
            format!(
                "Total disk usage: {}  Entries: {}",
                match self.total_bytes {
                    Some(b) => format!("{}", self.format.display(b)).trim().to_owned(),
                    None => "-".to_owned(),
                },
                self.entries_traversed
            ),
            (area.width - margin) as usize,
            Style {
                fg: text_color,
                bg: bg_color,
                ..Default::default()
            },
        )
    }
}
impl<'a, 'b> Widget for MainWindow<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            traversal:
                Traversal {
                    tree,
                    entries_traversed,
                    total_bytes,
                    ..
                },
            display,
            state,
        } = self;
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(256), Constraint::Length(1)].as_ref())
            .split(area);
        let (entries, footer) = (regions[0], regions[1]);
        Entries {
            tree: &tree,
            root: state.root,
            display: *display,
            sorting: state.sorting,
            selected: state.selected,
        }
        .draw(entries, buf);

        Footer {
            total_bytes: *total_bytes,
            entries_traversed: *entries_traversed,
            format: display.byte_format,
        }
        .draw(footer, buf);
    }
}

impl<'a> Widget for Entries<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            tree,
            root,
            display,
            sorting,
            selected,
        } = self;
        let entries = sorted_entries(tree, *root, *sorting);
        let total: u64 = entries.iter().map(|(_, w)| w.size).sum();
        List::new(entries.iter().map(|(node_idx, w)| {
            let style = match selected {
                Some(idx) if *idx == *node_idx => Style {
                    fg: Color::Black,
                    bg: Color::White,
                    ..Default::default()
                },
                _ => Style {
                    fg: Color::White,
                    bg: Color::Reset,
                    ..Default::default()
                },
            };
            Text::Styled(
                format!(
                    "{} | {:.02}% | {}",
                    display.byte_format.display(w.size),
                    (w.size as f64 / total as f64) * 100.0,
                    w.name.to_string_lossy()
                )
                .into(),
                style,
            )
        }))
        .block(Block::default().borders(Borders::ALL).title("Entries"))
        .start_corner(Corner::TopLeft)
        .draw(area, buf);
    }
}
