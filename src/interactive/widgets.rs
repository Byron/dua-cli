use super::{DisplayOptions, Traversal, Tree, TreeIndex};
use crate::ByteFormat;
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
}

pub struct MainWindow<'a> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
}

pub struct Footer {
    pub total_bytes: u64,
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
                format!("{}", self.format.display(self.total_bytes)).trim(),
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

fn get_size_or_panic(tree: &Tree, node_idx: TreeIndex) -> u64 {
    tree.node_weight(node_idx)
        .expect("node should always be retrievable with valid index")
        .size
}

impl<'a> Widget for MainWindow<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            traversal:
                Traversal {
                    tree,
                    root_index,
                    entries_traversed,
                    ..
                },
            display,
        } = *self;
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(256), Constraint::Length(1)].as_ref())
            .split(area);
        let (entries, footer) = (regions[0], regions[1]);
        Entries {
            tree: &tree,
            root: *root_index,
            display: display,
        }
        .draw(entries, buf);

        Footer {
            total_bytes: get_size_or_panic(&tree, *root_index),
            entries_traversed: *entries_traversed,
            format: display.byte_format,
        }
        .draw(footer, buf);
    }
}

impl<'a> Widget for Entries<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        use petgraph::Direction;
        let Self {
            tree,
            root,
            display,
        } = self;
        List::new(
            tree.neighbors_directed(*root, Direction::Outgoing)
                .filter_map(|w| {
                    tree.node_weight(w).map(|w| {
                        Text::Raw(
                            format!(
                                "{} | ----- | {}",
                                display.byte_format.display(w.size),
                                w.name.to_string_lossy()
                            )
                            .into(),
                        )
                    })
                }),
        )
        .block(Block::default().borders(Borders::ALL).title("Entries"))
        .start_corner(Corner::TopLeft)
        .draw(area, buf);
    }
}
