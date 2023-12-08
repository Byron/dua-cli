use crate::interactive::{
    path_of,
    widgets::{entry_color, EntryMarkMap},
    DisplayOptions, EntryDataBundle, SortMode,
};
use chrono::DateTime;
use dua::traverse::{EntryData, Tree, TreeIndex};
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
    pub sort_mode: SortMode,
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
            sort_mode,
        } = props.borrow();
        let list = &mut self.list;

        let is_top = |node_idx| {
            tree.neighbors_directed(node_idx, petgraph::Incoming)
                .next()
                .is_none()
        };

        let total: u128 = entries.iter().map(|b| b.data.size).sum();
        let title = title(&current_path(tree, root), entries.len());
        let title_block = title_block(&title, border_style);
        let entry_in_view = entry_in_view(selected.as_ref(), entries);

        let props = ListProps {
            block: Some(title_block),
            entry_in_view,
        };
        let lines = entries.iter().map(
            |EntryDataBundle {
                 index: node_idx,
                 data: entry_data,
                 is_dir,
                 exists,
             }| {
                let is_marked = is_marked(marked.as_deref(), node_idx);
                let is_selected = is_selected(selected, node_idx);
                let fraction = entry_data.size as f32 / total as f32;
                let style = style(is_selected, is_focussed);
                let local_style = local_style(fraction, style);

                let mut columns = Vec::new();
                if should_show_mtime_column(sort_mode) {
                    columns.push(mtime_column(entry_data, sort_mode, style));
                }
                columns.push(bytes_column(display, entry_data, sort_mode, style));
                columns.push(percentage_column(display, fraction, local_style));
                columns.push(name_column(
                    entry_data,
                    is_dir,
                    is_top,
                    root,
                    area,
                    name_style(is_marked, *exists, *is_dir, style),
                ));

                columns_with_separators(columns, local_style)
            },
        );

        list.render(props, lines, area, buf);

        if *is_focussed {
            let bound = draw_top_right_help(area, &title, buf);
            draw_bottom_right_help(bound, buf);
        }
    }
}

fn entry_in_view(
    selected: Option<&petgraph::stable_graph::NodeIndex>,
    entries: &[EntryDataBundle],
) -> Option<usize> {
    selected.map(|selected| {
        entries
            .iter()
            .find_position(|b| b.index == *selected)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    })
}

fn title_block<'a>(title: &'a str, border_style: &'a Style) -> Block<'a> {
    Block::default()
        .title(title)
        .border_style(*border_style)
        .borders(Borders::ALL)
}

fn title(current_path: &str, item_count: usize) -> String {
    let title = format!(
        " {} ({} item{}) ",
        current_path,
        item_count,
        match item_count {
            1 => "",
            _ => "s",
        }
    );
    title
}

fn current_path(
    tree: &petgraph::stable_graph::StableGraph<EntryData, ()>,
    root: &petgraph::stable_graph::NodeIndex,
) -> String {
    match path_of(tree, *root).to_string_lossy().to_string() {
        ref p if p.is_empty() => Path::new(".")
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from(".")),
        p => p,
    }
}

fn draw_bottom_right_help(bound: Rect, buf: &mut Buffer) {
    let bound = line_bound(bound, bound.height.saturating_sub(1) as usize);
    let help_text = " mark-move = d | mark-toggle = space | toggle-all = a";
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

fn draw_top_right_help(area: Rect, title: &str, buf: &mut Buffer) -> Rect {
    let help_text = " . = o|.. = u ── ⇊ = Ctrl+d|↓ = j|⇈ = Ctrl+u|↑ = k ";
    let help_text_block_width = block_width(help_text);
    let bound = Rect {
        width: area.width.saturating_sub(1),
        ..area
    };
    if block_width(title) + help_text_block_width <= bound.width {
        draw_text_nowrap_fn(
            rect::snap_to_right(bound, help_text_block_width),
            buf,
            help_text,
            |_, _, _| Style::default(),
        );
    }
    bound
}

fn is_marked(
    marked: Option<
        &std::collections::BTreeMap<petgraph::stable_graph::NodeIndex, super::EntryMark>,
    >,
    node_idx: &petgraph::stable_graph::NodeIndex,
) -> bool {
    marked.map(|m| m.contains_key(node_idx)).unwrap_or(false)
}

fn is_selected(
    selected: &Option<petgraph::stable_graph::NodeIndex>,
    node_idx: &petgraph::stable_graph::NodeIndex,
) -> bool {
    if let Some(idx) = selected {
        *idx == *node_idx
    } else {
        false
    }
}

fn style(is_selected: bool, is_focussed: &bool) -> Style {
    let mut style = Style::default();
    if is_selected {
        style.add_modifier.insert(Modifier::REVERSED);
    }
    if *is_focussed & is_selected {
        style.add_modifier.insert(Modifier::BOLD);
    }
    style
}

fn local_style(fraction: f32, style: Style) -> Style {
    let should_avoid_showing_a_big_reversed_bar = fraction > 0.9;
    if should_avoid_showing_a_big_reversed_bar {
        style.remove_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

fn columns_with_separators(columns: Vec<Span<'_>>, style: Style) -> Vec<Span<'_>> {
    let mut columns_with_separators = Vec::new();
    let column_count = columns.len();
    for (idx, column) in columns.into_iter().enumerate() {
        columns_with_separators.push(column);
        if idx != column_count - 1 {
            columns_with_separators.push(Span::styled(" | ", style))
        }
    }
    columns_with_separators
}

fn mtime_column<'a>(entry_data: &'a EntryData, sort_mode: &'a SortMode, style: Style) -> Span<'a> {
    let datetime = DateTime::<chrono::Utc>::from(entry_data.mtime);
    let formatted_time = datetime.format("%d/%m/%Y %H:%M:%S").to_string();
    Span::styled(
        format!("{:>20}", formatted_time),
        Style {
            fg: match sort_mode {
                SortMode::SizeAscending | SortMode::SizeDescending => style.fg,
                SortMode::MTimeAscending | SortMode::MTimeDescending => Color::Green.into(),
            },
            ..style
        },
    )
}

fn name_column<'a>(
    entry_data: &'a dua::traverse::EntryData,
    is_dir: &'a bool,
    is_top: impl Fn(petgraph::stable_graph::NodeIndex) -> bool,
    root: &'a petgraph::stable_graph::NodeIndex,
    area: Rect,
    style: Style,
) -> Span<'a> {
    Span::styled(
        fill_background_to_right(
            format!(
                "{prefix}{}",
                entry_data.name.to_string_lossy(),
                prefix = if *is_dir && !is_top(*root) { "/" } else { " " }
            ),
            area.width,
        ),
        style,
    )
}

fn name_style(is_marked: bool, exists: bool, is_dir: bool, style: Style) -> Style {
    let fg = if !exists {
        // non-existing - always red!
        Some(Color::Red)
    } else {
        entry_color(style.fg, !is_dir, is_marked)
    };
    Style { fg, ..style }
}

fn percentage_column(display: &DisplayOptions, fraction: f32, style: Style) -> Span {
    Span::styled(format!("{}", display.byte_vis.display(fraction)), style)
}

fn bytes_column<'a>(
    display: &'a DisplayOptions,
    entry_data: &'a dua::traverse::EntryData,
    sort_mode: &'a SortMode,
    style: Style,
) -> Span<'a> {
    Span::styled(
        format!(
            "{:>byte_column_width$}",
            display.byte_format.display(entry_data.size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
            byte_column_width = display.byte_format.width()
        ),
        Style {
            fg: match sort_mode {
                SortMode::SizeAscending | SortMode::SizeDescending => Color::Green.into(),
                SortMode::MTimeAscending | SortMode::MTimeDescending => style.fg,
            },
            ..style
        },
    )
}

fn should_show_mtime_column(sort_mode: &SortMode) -> bool {
    matches!(
        sort_mode,
        SortMode::MTimeAscending | SortMode::MTimeDescending
    )
}
