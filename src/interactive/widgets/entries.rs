use crate::interactive::{
    path_of,
    widgets::{entry_color, EntryMarkMap},
    DisplayOptions, EntryDataBundle,
};
use dua::traverse::{Tree, TreeIndex};
use itertools::Itertools;
use std::{borrow::Borrow, path::Path};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
};
use tui_react::util::rect::line_bound;
use tui_react::{
    draw_text_nowrap_fn, fill_background_to_right,
    util::{block_width, rect},
    List, ListProps,
};

pub struct EntriesProps<'a> {
    pub tree: &'a Tree,
    pub root: TreeIndex,
    pub display: DisplayOptions,
    pub selected: Option<TreeIndex>,
    pub entries: &'a [EntryDataBundle],
    pub marked: Option<&'a EntryMarkMap>,
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

        let total: u128 = entries.iter().map(|b| b.data.size).sum();
        let title = match path_of(tree, *root).to_string_lossy().to_string() {
            ref p if p.is_empty() => Path::new(".")
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::from(".")),
            p => p,
        };
        let title = format!(
            " {} ({} item{}) ",
            title,
            entries.len(),
            match entries.len() {
                1 => "",
                _ => "s",
            }
        );
        let block = Block::default()
            .title(title.as_str())
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
                let mut style = Style::default();
                let is_selected = if let Some(idx) = selected {
                    *idx == *node_idx
                } else {
                    false
                };
                if is_selected {
                    style.add_modifier.insert(Modifier::REVERSED);
                }
                if *is_focussed & is_selected {
                    style.add_modifier.insert(Modifier::BOLD);
                }

                let bytes = Span::styled(
                    format!(
                        "{:>byte_column_width$}",
                        display.byte_format.display(w.size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
                        byte_column_width = display.byte_format.width()
                    ),
                    Style {
                        fg: Color::Green.into(),
                        ..style
                    },
                );
                let percentage = Span::styled(
                    format!(
                        " |{}| ",
                        display.byte_vis.display(w.size as f32 / total as f32)
                    ),
                    style,
                );

                let name = Span::styled(
                    fill_background_to_right(
                        format!(
                            "{prefix}{}",
                            w.name.to_string_lossy(),
                            prefix = if *is_dir && !is_top(*root) { "/" } else { " " }
                        ),
                        area.width,
                    ),
                    {
                        let is_marked = marked.map(|m| m.contains_key(node_idx)).unwrap_or(false);
                        let fg = if !exists {
                            // non-existing - always red!
                            Some(Color::Red)
                        } else {
                            entry_color(style.fg, !*is_dir, is_marked)
                        };
                        Style { fg, ..style }
                    },
                );
                vec![bytes, percentage, name]
            },
        );

        list.render(props, lines, area, buf);

        if *is_focussed {
            let help_text = " . = o|.. = u ── ⇊ = CTRL+d|↓ = j|⇈ = CTRL+u|↑ = k ";
            let help_text_block_width = block_width(help_text);
            let bound = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            if block_width(&title) + help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    help_text,
                    |_, _, _| Style::default(),
                );
            }
            let bound = line_bound(bound, bound.height.saturating_sub(1) as usize);
            let help_text = " mark-move = d | mark-toggle = space ";
            let help_text_block_width = block_width(help_text);
            if help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    help_text,
                    |_, _, _| Style::default(),
                );
            }
        }
    }
}
