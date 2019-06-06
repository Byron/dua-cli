use crate::interactive::{DisplayOptions, EntryDataBundle, EntryMarkMap};
use dua::traverse::{Tree, TreeIndex};
use itertools::Itertools;
use std::{borrow::Borrow, path::Path};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Text},
};
use tui_react::{fill_background_to_right, List, ListProps};

pub struct EntriesProps<'a> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub selected: Option<TreeIndex>,
    pub entries: &'a [EntryDataBundle],
    pub marked: &'a EntryMarkMap,
    pub border_style: Style,
    pub is_focussed: bool,
}

#[derive(Default)]
pub struct Entries {
    pub list: List,
}

impl Entries {
    pub fn render<'a>(
        &mut self,
        props: impl Borrow<EntriesProps<'a>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let EntriesProps {
            tree,
            root,
            display,
            entries,
            selected,
            marked,
            border_style,
            is_focussed,
        } = props.borrow();
        let list = &mut self.list;

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
        let block = Block::default()
            .title(&title)
            .border_style(*border_style)
            .borders(Borders::ALL);
        let entry_in_view = selected.map(|selected| {
            entries
                .iter()
                .find_position(|b| b.index == selected)
                .map(|(idx, _)| idx)
                .unwrap_or(0)
        });

        let props = ListProps {
            block: Some(block),
            entry_in_view,
        };
        let lines = entries.iter().map(
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

                let dark_yellow = Color::Rgb(176, 126, 0);
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
                        fg: match (!is_dir, exists, marked.contains_key(node_idx)) {
                            (true, true, false) if !is_selected => Color::DarkGray,
                            (true, true, false) => style.fg,
                            (false, true, false) => style.fg,

                            (true, true, true) => dark_yellow,
                            (false, true, true) => Color::Yellow,

                            // non-existing - always red!
                            (_, false, _) => Color::Red,
                        },
                        ..style
                    },
                );
                let column_segments = vec![bytes, percentage, name];
                column_segments
            },
        );

        list.render(props, lines, area, buf);
    }
}
