use super::{DisplayOptions, Traversal, Tree, TreeIndex};
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

pub struct InitWindow<'a> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
}

fn get_size_or_panic(tree: &Tree, node_idx: TreeIndex) -> u64 {
    tree.node_weight(node_idx)
        .expect("node should always be retrievable with valid index")
        .size
}

impl<'a> Widget for InitWindow<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(256), Constraint::Length(1)].as_ref())
            .split(area);
        let (entries, footer) = (regions[0], regions[1]);
        Entries {
            tree: &self.traversal.tree,
            root: self.traversal.root_index,
            display: self.display,
        }
        .draw(entries, buf);

        let bg_color = Color::White;
        let text_color = Color::Black;
        let margin = 1;
        self.background(footer, buf, bg_color);
        buf.set_stringn(
            footer.x + margin,
            footer.y,
            format!(
                "Total disk usage: {}",
                format!(
                    "{}",
                    self.display.byte_format.display(get_size_or_panic(
                        &self.traversal.tree,
                        self.traversal.root_index
                    ))
                )
                .trim()
            ),
            (footer.width - margin) as usize,
            Style {
                fg: text_color,
                bg: bg_color,
                ..Default::default()
            },
        )
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
