use crate::{
    interactive::react::Component,
    interactive::{
        react::{BlockProps, ReactList, ReactListProps},
        widgets::{fill_background_to_right, ListState},
        DisplayOptions, EntryDataBundle,
    },
};
use dua::traverse::{Tree, TreeIndex};
use itertools::Itertools;
use std::path::Path;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Borders, Text, Widget},
};

pub struct Entries<'a, 'b, T> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub selected: Option<TreeIndex>,
    pub list_state: &'b mut ListState,
    pub entries: &'a [EntryDataBundle],
    pub border_style: Style,
    pub is_focussed: bool,

    pub list: ReactList<'a, 'b, T>,
}

impl<'a, 'b, T> Widget for Entries<'a, 'b, T> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            tree,
            root,
            display,
            entries,
            selected,
            border_style,
            list_state,
            is_focussed,
            list,
        } = self;
        let is_top = |node_idx| {
            tree.neighbors_directed(node_idx, petgraph::Incoming)
                .next()
                .is_none()
        };

        let total: u64 = entries.iter().map(|b| b.data.size).sum();
        let title = match dua::path_of(tree, *root).to_string_lossy().to_string() {
            ref p if p.is_empty() => Path::new(".")
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::from(".")),
            p => p,
        };
        let title = format!(" {} ", title);
        let block = BlockProps {
            borders: Borders::ALL,
            border_style: *border_style,
            title: Some(&title),
            ..Default::default()
        };

        let offset = list_state
            .update(
                selected.map(|selected| {
                    entries
                        .iter()
                        .find_position(|b| b.index == selected)
                        .map(|(idx, _)| idx)
                        .unwrap_or(0)
                }),
                block.inner(area).height as usize,
            )
            .start_index;

        let props = ReactListProps {
            block: Some(block),
            items: entries
                .iter()
                .skip(offset)
                .map(
                    |EntryDataBundle {
                         index: node_idx,
                         data: w,
                         is_dir,
                         exists,
                     }| {
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
                                    prefix = if *is_dir && !is_top(*root) { "/" } else { " " }
                                ),
                                area.width,
                            )
                            .into(),
                            Style {
                                fg: match (!is_dir, exists) {
                                    (true, true) if !is_selected => Color::DarkGray,
                                    (true, true) => style.fg,
                                    (_, false) => Color::Red,
                                    (false, true) => style.fg,
                                },
                                ..style
                            },
                        );
                        let column_segments = vec![bytes, percentage, name];
                        column_segments
                    },
                )
                .collect(),
        };
        list.render(props, area, buf);
    }
}
