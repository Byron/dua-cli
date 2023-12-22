use crate::interactive::widgets::COUNT;
use crate::interactive::{
    widgets::{entry_color, EntryMarkMap},
    DisplayOptions, EntryDataBundle, SortMode,
};
use chrono::DateTime;
use dua::traverse::TreeIndex;
use itertools::Itertools;
use std::borrow::Borrow;
use std::ops::Deref;
use std::time::SystemTime;
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
    pub current_path: String,
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
            current_path,
            display,
            entries,
            selected,
            marked,
            border_style,
            is_focussed,
            sort_mode,
        } = props.borrow();
        let list = &mut self.list;

        let total: u128 = entries.iter().map(|b| b.size).sum();
        let (item_count, item_size): (u64, u128) = entries
            .iter()
            .map(|f| (f.entry_count.unwrap_or(1), f.size))
            .reduce(|a, b| (a.0 + b.0, a.1 + b.1))
            .unwrap_or_default();
        let title = title(current_path, item_count, *display, item_size);
        let title_block = title_block(&title, *border_style);
        let inner_area = title_block.inner(area);
        let entry_in_view = entry_in_view(*selected, entries);

        let props = ListProps {
            block: Some(title_block),
            entry_in_view,
        };
        let lines = entries.iter().map(|bundle| {
            let node_idx = &bundle.index;
            let is_dir = &bundle.is_dir;
            let exists = &bundle.exists;
            let name = bundle.name.as_path();

            let is_marked = marked.map(|m| m.contains_key(node_idx)).unwrap_or(false);
            let is_selected = selected.map_or(false, |idx| idx == *node_idx);
            let fraction = bundle.size as f32 / total as f32;
            let text_style = style(is_selected, *is_focussed);
            let percentage_style = percentage_style(fraction, text_style);

            let mut columns = Vec::new();
            if show_mtime_column(sort_mode) {
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
            if show_count_column(sort_mode) {
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

            let shorten_name = shorten_string_middle(
                name_with_prefix(name.to_string_lossy().deref(), *is_dir).as_str(),
                available_width,
            );

            columns.push(name_column(
                &shorten_name,
                area,
                name_style(is_marked, *exists, *is_dir, text_style),
            ));

            columns_with_separators(columns, percentage_style, false)
        });

        list.render(props, lines, area, buf);

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

fn title(current_path: &str, item_count: u64, display: DisplayOptions, size: u128) -> String {
    format!(
        " {} ({} item{}, {}) ",
        current_path,
        COUNT.format(item_count as f64),
        match item_count {
            1 => "",
            _ => "s",
        },
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
    Span::styled(format!("{:>20}", formatted_time), style)
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

fn name_column(name: &str, area: Rect, style: Style) -> Span<'static> {
    Span::styled(fill_background_to_right(name.into(), area.width), style)
}

fn name_with_prefix(name: &str, is_dir: bool) -> String {
    format!(
        "{prefix}{name}",
        prefix = if is_dir {
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
                ""
            } else {
                "/"
            }
        } else {
            " "
        }
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

#[derive(PartialEq)]
enum Column {
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

fn show_mtime_column(sort_mode: &SortMode) -> bool {
    matches!(
        sort_mode,
        SortMode::MTimeAscending | SortMode::MTimeDescending
    )
}

fn show_count_column(sort_mode: &SortMode) -> bool {
    matches!(
        sort_mode,
        SortMode::CountAscending | SortMode::CountDescending
    )
}

fn shorten_string_middle(input: &str, width: usize) -> String {
    let ellipsis = "...";
    let ellipsis_len = ellipsis.chars().count();

    if input.chars().count() <= width {
        return input.to_string();
    }

    if ellipsis.chars().count() > width {
        return "".to_string();
    }

    let chars_per_half = (width - ellipsis_len) / 2;

    let first_half: String = input.chars().take(chars_per_half).collect();
    let second_half_start = input.chars().count() - chars_per_half;
    let second_half: String = input.chars().skip(second_half_start).collect();

    first_half + ellipsis + &second_half
}

#[cfg(test)]
mod entries_test {
    use super::shorten_string_middle;

    #[test]
    fn test_shorten_string_middle() {
        assert_eq!(shorten_string_middle("12345678", 7), "12...78".to_string());
        assert_eq!(shorten_string_middle("12345678", 3), "...".to_string());
        assert_eq!(shorten_string_middle("12345678", 2), "".to_string());
    }
}
