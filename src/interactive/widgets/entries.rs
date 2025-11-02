use crate::interactive::widgets::COUNT;
use crate::interactive::{
    DisplayOptions, EntryDataBundle, SortMode,
    widgets::{EntryMarkMap, entry_color},
};
use chrono::DateTime;
use dua::traverse::TreeIndex;
use itertools::Itertools;
use std::borrow::{Borrow, Cow};
use std::collections::HashSet;
use std::time::SystemTime;
use tui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};
use tui_react::util::rect::line_bound;
use tui_react::{
    List, ListProps, draw_text_nowrap_fn,
    util::{block_width, rect},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub struct EntriesProps<'a> {
    pub current_path: String,
    pub display: DisplayOptions,
    pub selected: Option<TreeIndex>,
    pub entries: &'a [EntryDataBundle],
    pub marked: Option<&'a EntryMarkMap>,
    pub border_style: Style,
    pub is_focussed: bool,
    pub sort_mode: SortMode,
    pub show_columns: &'a HashSet<Column>,
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
            current_path,
            display,
            entries,
            selected,
            marked,
            border_style,
            is_focussed,
            sort_mode,
            show_columns,
        } = props.borrow();
        let list = &mut self.list;

        let total: u128 = entries.iter().map(|b| b.size).sum();
        let (recursive_item_count, item_size): (u64, u128) = entries
            .iter()
            .map(|f| (f.entry_count.unwrap_or(1), f.size))
            .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
            .unwrap_or_default();
        let title = title(
            current_path,
            entries.len(),
            recursive_item_count,
            *display,
            item_size,
        );
        let title_block = title_block(&title, *border_style);
        let inner_area = title_block.inner(area);
        let entry_in_view = entry_in_view(*selected, entries);

        let props = ListProps {
            block: Some(title_block),
            entry_in_view,
        };
        let mut scroll_offset = None;
        let lines = entries.iter().enumerate().map(|(idx, bundle)| {
            let node_idx = &bundle.index;
            let is_dir = &bundle.is_dir;
            let exists = &bundle.exists;
            let name = bundle.name.as_path();

            let is_marked = marked.map(|m| m.contains_key(node_idx)).unwrap_or(false);
            let is_selected = selected == &Some(*node_idx);
            if is_selected {
                scroll_offset = Some(idx);
            }
            let fraction = bundle.size as f32 / total as f32;
            let text_style = style(is_selected, *is_focussed);
            let percentage_style = percentage_style(fraction, text_style);

            let mut columns = Vec::new();
            if show_mtime_column(sort_mode, show_columns) {
                columns.push(mtime_column(
                    bundle.mtime,
                    column_style(Column::MTime, *sort_mode, text_style),
                ));
            }
            columns.push(bytes_column(
                *display,
                bundle.size,
                column_style(Column::Bytes, *sort_mode, text_style),
            ));
            columns.push(percentage_column(*display, fraction, percentage_style));
            if show_count_column(sort_mode, show_columns) {
                columns.push(count_column(
                    bundle.entry_count,
                    column_style(Column::Count, *sort_mode, text_style),
                ));
            }

            let available_width = inner_area.width.saturating_sub(
                columns_with_separators(columns.clone(), percentage_style, true)
                    .iter()
                    .map(|f| f.width() as u16)
                    .sum(),
            ) as usize;

            let name = shorten_input(
                name_with_prefix(name.to_string_lossy(), *is_dir),
                available_width,
            );
            let style = name_style(is_marked, *exists, *is_dir, text_style);
            columns.push(name_column(name, area, style));

            columns_with_separators(columns, percentage_style, false)
        });

        let line_count = lines.len();
        list.render(props, lines, area, buf);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state =
            ScrollbarState::new(line_count).position(scroll_offset.unwrap_or(list.offset));

        scrollbar.render(area.inner(&Margin::new(0, 1)), buf, &mut scrollbar_state);

        if *is_focussed {
            let bound = draw_top_right_help(area, &title, buf);
            draw_bottom_right_help(bound, buf);
        }
    }
}

fn entry_in_view(
    selected: Option<petgraph::stable_graph::NodeIndex>,
    entries: &[EntryDataBundle],
) -> Option<usize> {
    selected.map(|selected| {
        entries
            .iter()
            .find_position(|b| b.index == selected)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    })
}

fn title_block(title: &str, border_style: Style) -> Block<'_> {
    Block::default()
        .title(title)
        .border_style(border_style)
        .borders(Borders::ALL)
}

fn title(
    current_path: &str,
    item_count: usize,
    recursive_item_count: u64,
    display: DisplayOptions,
    size: u128,
) -> String {
    format!(
        " {} ({item_count} visible, {} total, {}) ",
        current_path,
        COUNT.format(recursive_item_count as f64),
        display.byte_format.display(size)
    )
}

fn draw_bottom_right_help(bound: Rect, buf: &mut Buffer) {
    let bound = line_bound(bound, bound.height.saturating_sub(1) as usize);
    let help_text = " mark-move = d | mark-toggle = space | toggle-all = a ";
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

fn style(is_selected: bool, is_focussed: bool) -> Style {
    let mut style = Style::default();
    if is_selected {
        style.add_modifier.insert(Modifier::REVERSED);
    }
    if is_focussed & is_selected {
        style.add_modifier.insert(Modifier::BOLD);
    }
    style
}

fn percentage_style(fraction: f32, style: Style) -> Style {
    let avoid_big_reversed_bar = fraction > 0.9;
    if avoid_big_reversed_bar {
        style.remove_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

fn columns_with_separators(
    columns: Vec<Span<'_>>,
    style: Style,
    insert_last_separator: bool,
) -> Vec<Span<'_>> {
    let mut columns_with_separators = Vec::new();
    let column_count = columns.len();
    for (idx, column) in columns.into_iter().enumerate() {
        columns_with_separators.push(column);
        if insert_last_separator || idx != column_count - 1 {
            columns_with_separators.push(Span::styled(" | ", style))
        }
    }
    columns_with_separators
}

fn mtime_column(entry_mtime: SystemTime, style: Style) -> Span<'static> {
    let datetime = DateTime::<chrono::Utc>::from(entry_mtime);
    let formatted_time = datetime.format("%d/%m/%Y %H:%M:%S").to_string();
    Span::styled(format!("{formatted_time:>20}"), style)
}

fn count_column(entry_count: Option<u64>, style: Style) -> Span<'static> {
    Span::styled(
        format!(
            "{:>4}",
            match entry_count {
                Some(count) => {
                    COUNT.format(count as f64)
                }
                None => "".to_string(),
            }
        ),
        style,
    )
}

fn name_column(name: Cow<'_, str>, area: Rect, style: Style) -> Span<'_> {
    Span::styled(fill_background_to_right(name, area.width), style)
}

fn fill_background_to_right(mut s: Cow<'_, str>, entire_width: u16) -> Cow<'_, str> {
    match (s.len(), entire_width as usize) {
        (x, y) if x >= y => s,
        (x, y) => {
            s.to_mut().extend(std::iter::repeat_n(' ', y - x));
            s
        }
    }
}

fn name_with_prefix(mut name: Cow<'_, str>, is_dir: bool) -> Cow<'_, str> {
    let prefix = if is_dir {
        // Note that these names never happen on non-root items, so this is a root-item special case.
        // It was necessary since we can't trust the 'actual' root anymore as it might be the CWD or
        // `main()` cwd' into the one path that was provided by the user.
        // The idea was to keep explicit roots as specified without adjustment, which works with this
        // logic unless somebody provides `name` as is, then we will prefix it which is a little confusing.
        // Overall, this logic makes the folder display more consistent.
        if name == "."
            || name == ".."
            || name.starts_with('/')
            || name.starts_with("./")
            || name.starts_with("../")
        {
            None
        } else {
            Some("/")
        }
    } else {
        Some(" ")
    };
    match prefix {
        None => name,
        Some(prefix) => {
            name.to_mut().insert_str(0, prefix);
            name
        }
    }
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

fn percentage_column(display: DisplayOptions, fraction: f32, style: Style) -> Span<'static> {
    Span::styled(format!("{}", display.byte_vis.display(fraction)), style)
}

fn bytes_column(display: DisplayOptions, entry_size: u128, style: Style) -> Span<'static> {
    Span::styled(
        format!(
            "{:>byte_column_width$}",
            display.byte_format.display(entry_size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
            byte_column_width = display.byte_format.width()
        ),
        style,
    )
}

#[derive(PartialEq, Eq, Hash)]
pub enum Column {
    Bytes,
    MTime,
    Count,
}

fn column_style(column: Column, sort_mode: SortMode, style: Style) -> Style {
    Style {
        fg: match (sort_mode, column) {
            (SortMode::SizeAscending | SortMode::SizeDescending, Column::Bytes)
            | (SortMode::MTimeAscending | SortMode::MTimeDescending, Column::MTime)
            | (SortMode::CountAscending | SortMode::CountDescending, Column::Count) => {
                Color::Green.into()
            }
            _ => style.fg,
        },
        ..style
    }
}

fn show_mtime_column(sort_mode: &SortMode, show_columns: &HashSet<Column>) -> bool {
    matches!(
        sort_mode,
        SortMode::MTimeAscending | SortMode::MTimeDescending
    ) || show_columns.contains(&Column::MTime)
}

fn show_count_column(sort_mode: &SortMode, show_columns: &HashSet<Column>) -> bool {
    matches!(
        sort_mode,
        SortMode::CountAscending | SortMode::CountDescending
    ) || show_columns.contains(&Column::Count)
}

/// Note that this implementation isn't correct as `width` is the amount of blocks to display,
/// which is not what we are actually counting when adding graphemes to the output string.
fn shorten_input(input: Cow<'_, str>, width: usize) -> Cow<'_, str> {
    const ELLIPSIS: char = '…';
    const ELLIPSIS_LEN: usize = 1;
    const EXTENDED: bool = true;

    let total_count = input.width();
    if total_count <= width {
        return input;
    }

    if ELLIPSIS_LEN > width {
        return Cow::Borrowed("");
    }

    let graphemes_per_half = (width - ELLIPSIS_LEN) / 2;

    let mut out = String::with_capacity(width);
    let mut g = input.graphemes(EXTENDED);

    out.extend(g.by_ref().take(graphemes_per_half));
    out.push(ELLIPSIS);
    out.extend(g.skip(total_count - graphemes_per_half * 2));

    Cow::Owned(out)
}

#[cfg(test)]
mod entries_test {
    use super::shorten_input;

    #[test]
    fn test_shorten_string_middle() {
        let numbers = "12345678";
        let graphemes = "你好😁你好";
        for (input, target_length, expected) in [
            (numbers, 8, numbers),
            (numbers, 7, "123…678"),
            (numbers, 3, "1…8"),
            (numbers, 2, "…"),
            (numbers, 1, "…"),
            (numbers, 0, ""),
            // multi-block strings are handled incorrectly, but at least it doesn't crash.
            (graphemes, 0, ""),
            (graphemes, 1, "…"),
            (graphemes, 3, "你…"),
            (graphemes, 4, "你…"),
            (graphemes, 5, "你好…"),
            (graphemes, 6, "你好…"),
            (graphemes, 7, "你好😁…"),
            (graphemes, 8, "你好😁…"),
            (graphemes, 9, "你好😁你…"),
            (graphemes, 10, "你好😁你好"),
        ] {
            assert_eq!(shorten_input(input.into(), target_length), expected);
        }
    }
}
