use crate::interactive::widgets::COUNT;
use crate::interactive::widgets::tui_ext::util::rect::line_bound;
use crate::interactive::widgets::tui_ext::{
    List, ListProps, draw_text_nowrap_fn,
    util::{block_width, rect},
};
use crate::interactive::{
    DisplayOptions, EntryDataBundle, SortMode,
    widgets::{EntryMarkMap, entry_color},
};
use chrono::DateTime;
use dua::traverse::TreeIndex;
use itertools::Itertools;
use std::borrow::{Borrow, Cow};
use std::collections::{BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use tui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Inputs used to render the entries pane.
pub struct EntriesProps<'a> {
    /// Path shown in the entries pane title.
    pub current_path: PathBuf,
    /// Size display mode used for byte and percentage columns.
    pub display: DisplayOptions,
    /// Currently selected tree entry, if one is selected.
    pub selected: Option<TreeIndex>,
    /// Entries to display in the pane, already sorted for the current view.
    pub entries: &'a [EntryDataBundle],
    /// Entries currently marked for action, if marking is active.
    pub marked: Option<&'a EntryMarkMap>,
    /// Entry indices that match known cleanup-directory names, if enabled.
    pub cleanup_candidates: Option<&'a BTreeSet<TreeIndex>>,
    /// Entry indices ignored by the current git repository, if enabled.
    pub gitignored_entries: Option<&'a BTreeSet<TreeIndex>>,
    /// Border style for the entries pane.
    pub border_style: Style,
    /// Whether this pane currently owns keyboard focus.
    pub is_focussed: bool,
    /// Active sort mode, used for column visibility and highlighting.
    pub sort_mode: SortMode,
    /// Columns explicitly enabled in addition to columns implied by sorting.
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
            cleanup_candidates,
            gitignored_entries,
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
        let title_width_inside_borders = area.width.saturating_sub(2) as usize;
        let title = title(
            current_path,
            entries.len(),
            recursive_item_count,
            *display,
            item_size,
            title_width_inside_borders,
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
            let is_cleanup_candidate = cleanup_candidates.is_some_and(|c| c.contains(node_idx));
            let is_gitignored = gitignored_entries.is_some_and(|g| g.contains(node_idx));
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
            let style = name_style(
                is_marked,
                is_cleanup_candidate,
                is_gitignored,
                *exists,
                *is_dir,
                text_style,
            );
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

/// Build the entries pane title within `title_width_cells` terminal cells.
///
/// `title_width_cells` is the available display width for the title text itself,
/// after excluding the surrounding block borders.
fn title(
    current_path: &Path,
    item_count: usize,
    recursive_item_count: u64,
    display: DisplayOptions,
    size: u128,
    title_width_cells: usize,
) -> String {
    let statistics = format!(
        "({item_count} visible, {} total, {})",
        COUNT.format(recursive_item_count as f64),
        display.byte_format.display(size)
    );
    let title = format!(" {path} {statistics} ", path = current_path.display());
    if title.width() <= title_width_cells {
        return title;
    }

    path_title(current_path, title_width_cells)
}

fn path_title(current_path: &Path, width: usize) -> String {
    let padding = 2;
    match width {
        0 => String::new(),
        width if width <= padding => {
            shorten_input(current_path.to_string_lossy(), width).into_owned()
        }
        width => {
            let path = compact_path(current_path, width - padding);
            let title = format!(" {path} ");
            if title.width() <= width {
                title
            } else {
                shorten_input(Cow::Owned(title), width).into_owned()
            }
        }
    }
}

/// Shorten a display path to fit `width` cells by preserving path structure when possible.
///
/// Unlike `shorten_input`, this understands `Path` components and prefers replacing
/// a consecutive run of middle components with a `[N]` marker. Root and prefix
/// components are preserved; only ordinary path components are candidates for removal.
///
/// If the path cannot be parsed into components, or no component-removal candidate fits,
/// this falls back to `shorten_input`.
///
/// `width` is the maximum display width in terminal cells, not bytes or characters.
///
/// - `/a/b/c/d` at width 7 -> `/a[2]d`
/// - `a/b/c/d` at width 6 -> `a[2]d`
/// - `12345678` at width 3 -> `1…8`
fn compact_path(current_path: &Path, width: usize) -> Cow<'_, str> {
    let current_path_display = current_path.to_string_lossy();
    if current_path_display.width() <= width {
        return current_path_display;
    }

    let Some(path) = DisplayPath::new(current_path) else {
        return shorten_input(current_path_display, width);
    };

    let keep_ends = true;
    path.compact_by_removing_components(width, keep_ends)
        .or_else(|| path.compact_by_removing_components(width, !keep_ends))
        .map(Cow::Owned)
        .unwrap_or_else(|| shorten_input(current_path_display, width))
}

/// Parsed path data reused while evaluating component-removal candidates.
///
/// This keeps root/prefix text, removable components, component display widths,
/// and total path width together so each compaction candidate can be scored
/// without repeatedly walking the path or recalculating unchanged widths.
struct DisplayPath<'a> {
    prefix: String,
    separator: char,
    components: Vec<Cow<'a, str>>,
    component_width_prefix_sum: Vec<usize>,
    width: usize,
}

impl<'a> DisplayPath<'a> {
    fn new(path: &'a Path) -> Option<Self> {
        let separator = std::path::MAIN_SEPARATOR;
        let mut prefix = String::new();
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(prefix_component) => {
                    prefix.push_str(&prefix_component.as_os_str().to_string_lossy());
                }
                Component::RootDir => {
                    prefix.push(separator);
                }
                Component::CurDir => components.push(Cow::Borrowed(".")),
                Component::ParentDir => components.push(Cow::Borrowed("..")),
                Component::Normal(component) => {
                    components.push(component.to_string_lossy());
                }
            }
        }
        if components.len() < 2 {
            return None;
        }

        let mut component_width_prefix_sum = Vec::with_capacity(components.len() + 1);
        component_width_prefix_sum.push(0);
        for component in &components {
            component_width_prefix_sum
                .push(component_width_prefix_sum.last().copied().unwrap() + component.width());
        }
        let width = prefix.width()
            + component_width_prefix_sum.last().copied().unwrap()
            + separator.width().unwrap_or(1) * (components.len() - 1);

        Some(DisplayPath {
            prefix,
            separator,
            components,
            component_width_prefix_sum,
            width,
        })
    }

    /// Return the best path candidate that fits `width` by replacing components with `[N]`.
    ///
    /// Root and prefix components are always kept. When `keep_ends` is true, the
    /// first and last removable components are also kept visible and only a middle
    /// run may be replaced. When false, the removed run may include either
    /// removable end, which is a fallback for very narrow widths.
    fn compact_by_removing_components(&self, width: usize, keep_ends: bool) -> Option<String> {
        let component_count = self.components.len();
        let path_center = component_count as isize - 1;
        let (first_start, last_start) = if keep_ends {
            (1, component_count.checked_sub(2)?)
        } else {
            (0, component_count.checked_sub(1)?)
        };
        if last_start == 0 {
            return None;
        }

        let mut best = None;
        for start in first_start..=last_start {
            let first_end = start + 1;
            let last_end = if keep_ends {
                component_count - 1
            } else {
                component_count
            };
            if first_end > last_end || self.candidate_width(start, last_end) > width {
                continue;
            }

            // Find the shortest removable run that fits. For a fixed `start`,
            // increasing `end` removes more components, so candidate width is
            // monotonic non-increasing.
            let mut lower = first_end;
            let mut upper = last_end;
            while lower < upper {
                let middle = lower + (upper - lower) / 2;
                if self.candidate_width(start, middle) <= width {
                    upper = middle;
                } else {
                    lower = middle + 1;
                }
            }

            let end = lower;
            let removed_components = end - start;
            let removed_center = (start * 2 + removed_components - 1) as isize;
            let center_distance = (removed_center - path_center).abs();
            let candidate_width = self.candidate_width(start, end);
            if best.as_ref().is_none_or(
                |(_, _, best_removed_components, best_distance, best_width)| {
                    (removed_components, center_distance, candidate_width)
                        < (*best_removed_components, *best_distance, *best_width)
                },
            ) {
                best = Some((
                    start,
                    end,
                    removed_components,
                    center_distance,
                    candidate_width,
                ));
            }
        }

        best.map(|(start, end, _, _, _)| self.candidate(start, end))
    }

    fn candidate_width(&self, start: usize, end: usize) -> usize {
        fn marker_width(removed_components: usize) -> usize {
            2 + removed_components.ilog10() as usize + 1
        }

        let removed_components = end - start;
        let removed_component_width =
            self.component_width_prefix_sum[end] - self.component_width_prefix_sum[start];
        let removed_separator_width = self.separator.width().unwrap_or(1)
            * ((removed_components - 1)
                + usize::from(start > 0)
                + usize::from(end < self.components.len()));
        self.width - removed_component_width - removed_separator_width
            + marker_width(removed_components)
    }

    fn candidate(&self, start: usize, end: usize) -> String {
        fn push_components(out: &mut String, separator: char, components: &[Cow<'_, str>]) {
            for (idx, component) in components.iter().enumerate() {
                if idx > 0 {
                    out.push(separator);
                }
                out.push_str(component);
            }
        }

        let mut out = String::new();
        out.push_str(&self.prefix);
        push_components(&mut out, self.separator, &self.components[..start]);
        let removed_components = end - start;
        out.push_str(&format!("[{removed_components}]"));
        push_components(&mut out, self.separator, &self.components[end..]);
        out
    }
}

fn draw_bottom_right_help(bound: Rect, buf: &mut Buffer) {
    let bound = line_bound(bound, bound.height.saturating_sub(1) as usize);
    let mut help_text = " mark-move = d | mark-toggle = space | cleanup = X".to_string();
    help_text.push_str(" | gitignore = I");
    help_text.push_str(" | all = a ");

    let help_text_block_width = block_width(&help_text);
    if help_text_block_width <= bound.width {
        draw_text_nowrap_fn(
            rect::snap_to_right(bound, help_text_block_width),
            buf,
            &help_text,
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

fn name_style(
    is_marked: bool,
    is_cleanup_candidate: bool,
    is_gitignored: bool,
    exists: bool,
    is_dir: bool,
    style: Style,
) -> Style {
    let mut style = style;
    let fg = if !exists {
        // non-existing - always red!
        Some(Color::Red)
    } else if is_cleanup_candidate && !is_marked {
        Some(Color::Magenta)
    } else {
        entry_color(style.fg, !is_dir, is_marked)
    };
    if is_gitignored && !is_marked && exists {
        style.add_modifier.insert(Modifier::DIM);
    }
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
            | (SortMode::MTimeAscending(_) | SortMode::MTimeDescending(_), Column::MTime)
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
        SortMode::MTimeAscending(_) | SortMode::MTimeDescending(_)
    ) || show_columns.contains(&Column::MTime)
}

fn show_count_column(sort_mode: &SortMode, show_columns: &HashSet<Column>) -> bool {
    matches!(
        sort_mode,
        SortMode::CountAscending | SortMode::CountDescending
    ) || show_columns.contains(&Column::Count)
}

/// Shorten arbitrary text by keeping terminal-cell-budgeted pieces from the start and end.
///
/// This is the path-agnostic fallback used when structured compaction via
/// `compact_path` is not possible. It does not understand separators or path
/// prefixes; it only inserts an ellipsis between the retained leading and trailing
/// graphemes.
///
/// `width` is the maximum display width in terminal cells, not bytes or characters.
///
/// # Examples
///
/// - `12345678` at width 7 -> `123…678`
/// - `12345678` at width 3 -> `1…8`
/// - `12345678` at width 2 -> `…`
/// - `你好😁你好` at width 5 -> `你…好`
fn shorten_input(input: Cow<'_, str>, width: usize) -> Cow<'_, str> {
    const ELLIPSIS: char = '…';
    const EXTENDED: bool = true;

    if input.width() <= width {
        return input;
    }

    let ellipsis_width = ELLIPSIS.width().unwrap_or(1);
    if ellipsis_width > width {
        return Cow::Borrowed("");
    }
    if width == ellipsis_width {
        return Cow::Owned(ELLIPSIS.to_string());
    }

    let retained_width = width - ellipsis_width;
    if retained_width < 2 {
        return Cow::Owned(ELLIPSIS.to_string());
    }

    let prefix_width = retained_width / 2 + retained_width % 2;
    let suffix_width = retained_width / 2;
    let graphemes: Vec<_> = input.graphemes(EXTENDED).collect();
    let prefix_end = grapheme_boundary_fitting_width(
        graphemes
            .iter()
            .enumerate()
            .map(|(idx, grapheme)| (idx + 1, *grapheme)),
        0,
        prefix_width,
    );
    let suffix_start = grapheme_boundary_fitting_width(
        (prefix_end..graphemes.len())
            .rev()
            .map(|idx| (idx, graphemes[idx])),
        graphemes.len(),
        suffix_width,
    );
    let split_budget_fits_no_graphemes = prefix_end == 0 && suffix_start == graphemes.len();
    let (prefix_end, suffix_start) = if split_budget_fits_no_graphemes {
        (
            grapheme_boundary_fitting_width(
                graphemes
                    .iter()
                    .enumerate()
                    .map(|(idx, grapheme)| (idx + 1, *grapheme)),
                0,
                retained_width,
            ),
            graphemes.len(),
        )
    } else {
        (prefix_end, suffix_start)
    };

    let mut out = String::new();
    out.extend(graphemes[..prefix_end].iter().copied());
    out.push(ELLIPSIS);
    out.extend(graphemes[suffix_start..].iter().copied());

    Cow::Owned(out)
}

fn grapheme_boundary_fitting_width<'a>(
    graphemes: impl IntoIterator<Item = (usize, &'a str)>,
    initial_boundary: usize,
    width: usize,
) -> usize {
    let mut used_width = 0;
    let mut boundary = initial_boundary;
    for (next_boundary, grapheme) in graphemes {
        let grapheme_width = grapheme.width();
        if used_width + grapheme_width > width {
            break;
        }
        used_width += grapheme_width;
        boundary = next_boundary;
    }
    boundary
}

#[cfg(test)]
mod entries_test {
    use std::collections::HashSet;
    use std::path::Path;

    use super::{name_style, shorten_input, show_mtime_column, title as entry_title};
    use crate::interactive::widgets::Column;
    use crate::interactive::{MTimeSort, SortMode};
    use dua::ByteFormat;
    use tui::style::{Color, Modifier, Style};
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn test_shorten_string_middle() {
        let numbers = "12345678";
        let graphemes = "你好😁世界";
        for (input, target_width, expected) in [
            (numbers, 8, numbers),
            (numbers, 7, "123…678"),
            (numbers, 3, "1…8"),
            (numbers, 2, "…"),
            (numbers, 1, "…"),
            (numbers, 0, ""),
            (graphemes, 0, ""),
            (graphemes, 1, "…"),
            (graphemes, 3, "你…"),
            (graphemes, 4, "你…"),
            (graphemes, 5, "你…界"),
            (graphemes, 6, "你…界"),
            (graphemes, 7, "你…界"),
            (graphemes, 8, "你好…界"),
            (graphemes, 9, "你好…世界"),
            (graphemes, 10, "你好😁世界"),
        ] {
            let actual = shorten_input(input.into(), target_width);
            assert_eq!(actual, expected);
            assert!(actual.as_ref().width() <= target_width);
        }
    }

    #[test]
    fn title_drops_statistics_before_shortening_path() {
        let path = "a/b/c";
        assert_eq!(title(path, path.len() + 2), " a/b/c ");
    }

    #[test]
    fn title_shows_stats_if_possible() {
        assert_eq!(
            title("项目/资料", 42),
            " 项目/资料 (4 visible, 43 total, 1.42 GB) "
        );
    }

    #[test]
    fn title_drops_statistics_for_wide_unicode_path() {
        assert_eq!(title("项目/资料", 41), " 项目/资料 ");
    }

    #[test]
    fn title_degrades_path_by_removing_fewest_consecutive_components() {
        assert_eq!(title("a/b/c/d", 8), " a[2]d ");
    }

    #[test]
    fn title_degrades_unicode_path_components_by_display_width() {
        assert_eq!(title("项目/😀😀😀😀/资料/📦", 12), " 项目[2]📦 ");
    }

    #[test]
    #[cfg(unix)]
    fn title_preserves_root_dir() {
        assert_eq!(title("/a/b/c/d", 9), " /a[2]d ");
    }

    #[test]
    #[cfg(windows)]
    fn title_degrades_backslash_separated_paths() {
        assert_eq!(title("a\\b\\c\\d", 8), " a[2]d ");
    }

    #[test]
    #[cfg(unix)]
    fn title_keeps_unix_backslashes_as_component_text() {
        assert_eq!(title("foo\\bar/baz/qux", 15), " foo\\bar[1]qux ");
    }

    #[test]
    fn title_treats_drive_like_text_as_relative_component() {
        assert_eq!(title("C:\\long/bar/baz/qux", 17), " C:\\long[2]qux ");
    }

    #[test]
    #[cfg(unix)]
    fn path_degradation_selects_the_shortest_sufficient_run() {
        assert_eq!(
            title("short/very-long-component/tiny/end", 19),
            " short[1]tiny/end "
        );
    }

    fn title(path: impl AsRef<Path>, title_width_cells: usize) -> String {
        let display = crate::interactive::DisplayOptions::new(ByteFormat::Metric);
        entry_title(
            path.as_ref(),
            4,
            43,
            display,
            1_420_000_000,
            title_width_cells,
        )
    }

    #[test]
    fn sorting_by_mtime_shows_column_like_count_sorting() {
        let mut show_columns = HashSet::new();
        assert!(
            show_mtime_column(&SortMode::MTimeDescending(MTimeSort::Entry), &show_columns,),
            "mtime sorting shows the mtime column even when it is not explicitly enabled",
        );

        show_columns.insert(Column::MTime);
        assert!(
            show_mtime_column(&SortMode::SizeDescending, &show_columns,),
            "explicitly enabling the mtime column shows it for non-mtime sorts",
        );
    }

    #[test]
    fn name_style_prioritizes_missing_gitignored_and_cleanup_states() {
        let style = Style::default();
        let is_marked = false;
        let is_cleanup_candidate = true;
        let is_gitignored = true;
        let exists = true;
        let is_dir = true;

        assert_eq!(
            name_style(
                is_marked,
                is_cleanup_candidate,
                is_gitignored,
                !exists,
                is_dir,
                style
            )
            .fg,
            Some(Color::Red),
            "missing entries stay red"
        );

        let gitignored = name_style(
            is_marked,
            !is_cleanup_candidate,
            is_gitignored,
            exists,
            is_dir,
            style,
        );
        assert_eq!(
            gitignored.fg,
            Some(Color::Cyan),
            "gitignored entries keep the regular directory color"
        );
        assert!(gitignored.add_modifier.contains(Modifier::DIM));

        let cleanup = name_style(
            is_marked,
            is_cleanup_candidate,
            !is_gitignored,
            exists,
            is_dir,
            style,
        );
        assert_eq!(
            cleanup.fg,
            Some(Color::Magenta),
            "cleanup candidates use a distinct foreground color"
        );
        assert!(
            !cleanup.add_modifier.contains(Modifier::DIM),
            "cleanup candidates are colored without dimming unless they are gitignored"
        );
    }
}
