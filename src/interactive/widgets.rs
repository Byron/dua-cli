use super::{Tree, TreeIndex};
use petgraph::Direction;
use tui::buffer::Buffer;
use tui::layout::{Corner, Rect};
use tui::widgets::{Block, Borders, List, Text, Widget};

pub struct Entries<'a> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
}

impl<'a> Widget for Entries<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self { tree, root } = self;
        List::new(
            tree.neighbors_directed(*root, Direction::Outgoing)
                .filter_map(|w| {
                    tree.node_weight(w).map(|w| {
                        Text::Raw(
                            format!("{} | ----- | {}", w.size, w.name.to_string_lossy()).into(),
                        )
                    })
                }),
        )
        .block(Block::default().borders(Borders::ALL).title("Entries"))
        .start_corner(Corner::TopLeft)
        .draw(area, buf);
    }
}
