use crate::{
    interactive::{widgets::DrawState, DisplayOptions, SortMode},
    sorted_entries,
    traverse::{Tree, TreeIndex},
};
use itertools::Itertools;
use std::path::Path;
use tui::{
    buffer::Buffer,
    layout::{Corner, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, Text, Widget},
};

pub struct Entries<'a, 'b> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub sorting: SortMode,
    pub selected: Option<TreeIndex>,
    pub state: &'b DrawState,
}

impl<'a, 'b> Widget for Entries<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            tree,
            root,
            display,
            sorting,
            selected,
            state: _,
        } = self;
        let is_top = |node_idx| {
            tree.neighbors_directed(node_idx, petgraph::Incoming)
                .next()
                .is_none()
        };
        let path_of = |node_idx| crate::common::path_of(tree, node_idx);

        let entries = sorted_entries(tree, *root, *sorting);
        let total: u64 = entries.iter().map(|(_, w)| w.size).sum();
        let title = match path_of(*root).to_string_lossy().to_string() {
            ref p if p.is_empty() => Path::new(".")
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::from(".")),
            p => p,
        };
        let title = format!(" {} ", title);
        let block = Block::default().borders(Borders::ALL).title(&title);
        let offset = match selected {
            Some(selected) => {
                let pos = entries
                    .iter()
                    .find_position(|(idx, _)| *idx == *selected)
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                match block.inner(area).height as usize {
                    h if pos >= h => pos - h + 1,
                    _ => 0,
                }
            }
            None => 0,
        };
        List::new(entries.iter().skip(offset).map(|(node_idx, w)| {
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
                    "{} | {:>5.02}% | {}{}",
                    display.byte_format.display(w.size),
                    (w.size as f64 / total as f64) * 100.0,
                    match path_of(*node_idx) {
                        ref p if p.is_dir() && !is_top(*root) => "/",
                        _ => " ",
                    },
                    w.name.to_string_lossy(),
                )
                .into(),
                style,
            )
        }))
        .block(block)
        .start_corner(Corner::TopLeft)
        .draw(area, buf);
    }
}
