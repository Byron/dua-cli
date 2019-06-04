use crate::{
    interactive::{DisplayOptions, SortMode},
    sorted_entries,
    traverse::{Tree, TreeIndex},
};
use itertools::Itertools;
use std::iter::repeat;
use std::path::Path;
use tui::{
    buffer::Buffer,
    layout::{Corner, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, Text, Widget},
};

#[derive(Default)]
pub struct ListState {
    /// The index at which the list last started. Used for scrolling
    pub start_index: usize,
}

impl ListState {
    pub fn update(&mut self, selected: Option<usize>, height: usize) -> &mut Self {
        self.start_index = match selected {
            Some(pos) => match height as usize {
                h if self.start_index + h - 1 < pos => pos - h + 1,
                _ if self.start_index > pos => pos,
                _ => self.start_index,
            },
            None => 0,
        };
        self
    }
}

fn fill_background_to_right(mut s: String, entire_width: u16) -> String {
    match (s.len(), entire_width as usize) {
        (x, y) if x >= y => s,
        (x, y) => {
            s.extend(repeat(' ').take(y - x));
            s
        }
    }
}

pub struct Entries<'a, 'b> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub sorting: SortMode,
    pub selected: Option<TreeIndex>,
    pub list: &'b mut ListState,
}

impl<'a, 'b> Widget for Entries<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            tree,
            root,
            display,
            sorting,
            selected,
            list,
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
        let offset = list
            .update(
                selected.map(|selected| {
                    entries
                        .iter()
                        .find_position(|(idx, _)| *idx == selected)
                        .map(|(idx, _)| idx)
                        .unwrap_or(0)
                }),
                block.inner(area).height as usize,
            )
            .start_index;

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
                fill_background_to_right(
                    format!(
                        "{} | {:>5.02}% | {}{}",
                        display.byte_format.display(w.size),
                        (w.size as f64 / total as f64) * 100.0,
                        match path_of(*node_idx) {
                            ref p if p.is_dir() && !is_top(*root) => "/",
                            _ => " ",
                        },
                        w.name.to_string_lossy(),
                    ),
                    area.width,
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
