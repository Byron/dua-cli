use crate::interactive::{
    widgets::{fill_background_to_right, List, ListState},
    DisplayOptions,
};
use dua::{
    sorted_entries,
    traverse::{Tree, TreeIndex},
    SortMode,
};
use itertools::Itertools;
use std::path::Path;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Text, Widget},
};

pub struct Entries<'a, 'b> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub sorting: SortMode,
    pub selected: Option<TreeIndex>,
    pub list: &'b mut ListState,
    pub border_style: Style,
    pub is_focussed: bool,
}

impl<'a, 'b> Widget for Entries<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            tree,
            root,
            display,
            sorting,
            selected,
            border_style,
            list,
            is_focussed,
        } = self;
        let is_top = |node_idx| {
            tree.neighbors_directed(node_idx, petgraph::Incoming)
                .next()
                .is_none()
        };
        let path_of = |node_idx| dua::path_of(tree, node_idx);

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
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(*border_style)
            .title(&title);
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

        List {
            block: Some(block),
            items: entries.iter().skip(offset).map(|(node_idx, w)| {
                let (is_selected, style) = match selected {
                    Some(idx) if *idx == *node_idx => (
                        true,
                        Style {
                            fg: Color::Black,
                            bg: if *is_focussed {
                                Color::White
                            } else {
                                Color::DarkGray
                            },
                            ..Default::default()
                        },
                    ),
                    _ => (
                        false,
                        Style {
                            fg: Color::White,
                            bg: Color::Reset,
                            ..Default::default()
                        },
                    ),
                };
                let path = path_of(*node_idx);

                let bytes = Text::Styled(
                    format!(
                        "{:>byte_column_width$}",
                        display.byte_format.display(w.size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
                        byte_column_width = display.byte_format.width()
                    )
                    .into(),
                    Style {
                        fg: match (is_selected, *is_focussed) {
                            (true, true) => Color::DarkGray,
                            (true, false) => Color::Black,
                            _ => Color::Green,
                        },
                        ..style
                    },
                );
                let percentage = Text::Styled(
                    format!(
                        " |{}| ",
                        display.byte_vis.display(w.size as f32 / total as f32)
                    )
                    .into(),
                    style,
                );
                let name = Text::Styled(
                    fill_background_to_right(
                        format!(
                            "{prefix}{}",
                            w.name.to_string_lossy(),
                            prefix = if path.is_dir() && !is_top(*root) {
                                "/"
                            } else {
                                " "
                            }
                        ),
                        area.width,
                    )
                    .into(),
                    Style {
                        fg: if path.is_dir() {
                            style.fg
                        } else {
                            if is_selected {
                                style.fg
                            } else {
                                Color::DarkGray
                            }
                        },
                        ..style
                    },
                );
                let column_segments = vec![bytes, percentage, name];
                column_segments
            }),
        }
        .draw(area, buf);
    }
}
