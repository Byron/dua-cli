use crate::interactive::widgets::COUNT;
use crate::interactive::widgets::tui_ext::{
    List, ListProps, draw_text_nowrap_fn,
    util::{block_width, rect, rect::line_bound},
};
use crate::interactive::{
    CursorDirection,
    app::tree_view::TreeView,
    fit_string_graphemes_with_ellipsis,
    widgets::{Language, entry_color},
};
use crossterm::event::{KeyEvent, KeyEventKind};
use dua::{ByteFormat, KeysConfig, traverse::TreeIndex};
use itertools::Itertools;
use std::{
    borrow::Borrow,
    collections::{BTreeMap, btree_map::Entry},
    path::PathBuf,
};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy)]
pub enum MarkMode {
    Delete,
    #[cfg(feature = "trash-move")]
    Trash,
}

pub type EntryMarkMap = BTreeMap<TreeIndex, EntryMark>;

#[derive(Default)]
pub struct EntryMark {
    pub size: u128,
    pub path: PathBuf,
    pub index: usize,
    pub num_errors_during_deletion: usize,
    pub is_dir: bool,
    pub entry_count: Option<u64>,
}

#[derive(Default)]
pub struct MarkPane {
    selected: Option<usize>,
    marked: EntryMarkMap,
    list: List,
    has_focus: bool,
    last_sorting_index: usize,
    total_size: u128,
    item_count: u64,
}

pub struct MarkPaneProps<'a> {
    pub border_style: Style,
    pub format: ByteFormat,
    pub root_total_size: u128,
    pub keys: &'a KeysConfig,
    pub safety_notice: Option<&'static str>,
    pub allow_changes: bool,
    pub language: Language,
}

impl MarkPane {
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }
    pub fn set_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
        if has_focus {
            self.selected = Some(self.marked.len().saturating_sub(1));
        } else {
            self.selected = None;
        }
    }
    pub fn toggle_index(
        mut self,
        index: TreeIndex,
        tree_view: &TreeView<'_>,
        is_dir: bool,
        toggle: bool,
    ) -> Option<Self> {
        match self.marked.entry(index) {
            Entry::Vacant(entry) => {
                if let Some(e) = tree_view.tree().entry(index) {
                    let sorting_index = self.last_sorting_index + 1;
                    self.last_sorting_index = sorting_index;
                    entry.insert(EntryMark {
                        size: e.size,
                        path: tree_view.path_of(index),
                        index: sorting_index,
                        num_errors_during_deletion: 0,
                        is_dir,
                        entry_count: e.entry_count,
                    });
                }
            }
            Entry::Occupied(entry) => {
                if toggle {
                    entry.remove();
                }
            }
        }
        if self.marked.is_empty() {
            None
        } else {
            (self.total_size, self.item_count) = calculate_size_and_count(&self.marked);
            Some(self)
        }
    }
    pub fn marked(&self) -> &EntryMarkMap {
        &self.marked
    }
    pub fn is_empty(&self) -> bool {
        self.marked.is_empty()
    }
    pub fn total_size(&self) -> u128 {
        self.total_size
    }
    pub fn reconcile(&mut self, tree_view: &TreeView<'_>) {
        let selected_index = self
            .selected
            .and_then(|position| self.tree_index_by_list_position(position));
        self.marked.retain(|index, mark| {
            let Some(entry) = tree_view.tree().entry(*index) else {
                return false;
            };
            if tree_view.path_of(*index) != mark.path {
                return false;
            }
            mark.size = entry.size;
            mark.entry_count = entry.entry_count;
            true
        });
        (self.total_size, self.item_count) = calculate_size_and_count(&self.marked);
        self.selected = self.selected.and_then(|position| {
            if self.marked.is_empty() {
                return None;
            }
            Some(
                self.marked_sorted_by_index()
                    .iter()
                    .position(|(index, _)| Some(**index) == selected_index)
                    .unwrap_or_else(|| position.min(self.marked.len() - 1)),
            )
        });
        self.list.offset = self.list.offset.min(self.marked.len().saturating_sub(1));
    }
    pub fn set_deletion_error(&mut self, index: TreeIndex, errors: usize) {
        if let Some(mark) = self.marked.get_mut(&index) {
            mark.num_errors_during_deletion = errors;
        }
    }
    pub fn into_paths(self) -> impl Iterator<Item = PathBuf> {
        self.marked.into_values().map(|v| v.path)
    }
    pub fn process_events(
        mut self,
        key: KeyEvent,
        keys: &KeysConfig,
        allow_changes: bool,
    ) -> Option<(Self, Option<MarkMode>)> {
        let action = None;
        if key.kind == KeyEventKind::Release {
            return Some((self, action));
        }
        if allow_changes && keys.delete_marked.matches(key) {
            return Some(self.prepare_deletion(MarkMode::Delete));
        }
        #[cfg(feature = "trash-move")]
        if allow_changes && keys.trash_marked.matches(key) {
            return Some(self.prepare_deletion(MarkMode::Trash));
        }
        if allow_changes && keys.remove_all_marks.matches(key) {
            return None;
        }
        if keys.move_to_top.matches(key) {
            self.change_selection(CursorDirection::ToTop);
        } else if keys.move_to_bottom.matches(key) {
            self.change_selection(CursorDirection::ToBottom);
        } else if keys.page_up.matches(key) {
            self.change_selection(CursorDirection::PageUp);
        } else if keys.page_down.matches(key) {
            self.change_selection(CursorDirection::PageDown);
        } else if keys.move_up.matches(key) {
            self.change_selection(CursorDirection::Up);
        } else if keys.move_down.matches(key) {
            self.change_selection(CursorDirection::Down);
        } else if allow_changes && keys.remove_mark.matches(key) {
            return self.remove_selected().map(|s| (s, action));
        }
        Some((self, action))
    }

    fn prepare_deletion(mut self, mark: MarkMode) -> (Self, Option<MarkMode>) {
        for entry in self.marked.values_mut() {
            entry.num_errors_during_deletion = 0;
        }
        self.selected = Some(0);
        (self, Some(mark))
    }
    fn remove_selected(mut self) -> Option<Self> {
        if let Some(mut selected) = self.selected {
            let idx = self.tree_index_by_list_position(selected);
            let se_len = self.marked.len();
            if let Some(idx) = idx {
                self.marked.remove(&idx);
                (self.total_size, self.item_count) = calculate_size_and_count(&self.marked);
                let new_len = se_len.saturating_sub(1);
                if new_len == 0 {
                    return None;
                }
                if new_len == selected {
                    selected = selected.saturating_sub(1);
                }
                self.selected = Some(selected);
            }
        }
        Some(self)
    }

    fn tree_index_by_list_position(&self, selected: usize) -> Option<TreeIndex> {
        self.marked_sorted_by_index()
            .get(selected)
            .map(|(k, _)| *k.to_owned())
    }

    fn marked_sorted_by_index(&self) -> Vec<(&TreeIndex, &EntryMark)> {
        self.marked
            .iter()
            .sorted_by_key(|(_, v)| &v.index)
            .collect()
    }

    fn change_selection(&mut self, direction: CursorDirection) {
        self.selected = self.selected.map(|selected| {
            direction
                .move_cursor(selected)
                .min(self.marked.len().saturating_sub(1))
        });
    }

    pub fn render<'a>(
        &mut self,
        props: impl Borrow<MarkPaneProps<'a>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let MarkPaneProps {
            border_style,
            format,
            root_total_size,
            keys,
            safety_notice,
            allow_changes,
            language,
        } = props.borrow();

        let marked: &_ = &self.marked;
        let percentage = if *root_total_size == 0 {
            0.0
        } else {
            self.total_size as f64 / *root_total_size as f64 * 100.0
        };
        let title = language.marked_title(
            &COUNT.format(self.item_count as f64),
            &format.display(self.total_size).to_string(),
            percentage,
            &format.display(*root_total_size).to_string(),
        );
        let selected = self.selected;
        let has_focus = self.has_focus;
        let entries = marked.values().sorted_by_key(|v| &v.index).enumerate().map(
            |(idx, v): (usize, &EntryMark)| {
                let base_style = match selected {
                    Some(selected) if idx == selected => {
                        let mut modifier = Modifier::REVERSED;
                        if has_focus {
                            modifier.insert(Modifier::BOLD);
                        }
                        Style {
                            add_modifier: modifier,
                            ..Default::default()
                        }
                    }
                    _ => Style::default(),
                };
                let (path, path_len) = {
                    let path = format!(
                        " {}  {}",
                        v.path.display(),
                        if v.num_errors_during_deletion != 0 {
                            language.deletion_errors(v.num_errors_during_deletion)
                        } else {
                            String::new()
                        }
                    );
                    let num_path_graphemes = path.graphemes(true).count();
                    match num_path_graphemes + format.total_width() {
                        n if n > area.width as usize => {
                            let desired_size =
                                num_path_graphemes.saturating_sub(n - area.width as usize);
                            fit_string_graphemes_with_ellipsis(
                                path,
                                num_path_graphemes,
                                desired_size,
                            )
                        }
                        _ => (path, num_path_graphemes),
                    }
                };
                let fg_path = entry_color(None, !v.is_dir, true);
                let path = Span::styled(
                    path,
                    Style {
                        fg: fg_path,
                        ..base_style
                    },
                );
                let bytes = Span::styled(
                    format!(
                        "{:>byte_column_width$} ",
                        format.display(v.size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
                        byte_column_width = format.width()
                    ),
                    Style {
                        fg: Color::Green.into(),
                        ..base_style
                    },
                );
                let spacer = Span::styled(
                    format!(
                        "{:-space$}",
                        "",
                        space = (area.width as usize)
                            .saturating_sub(path_len)
                            .saturating_sub(format.total_width())
                    ),
                    Style {
                        fg: fg_path,
                        ..base_style
                    },
                );
                vec![path, spacer, bytes]
            },
        );

        let entry_in_view = if let Some(s) = self.selected {
            Some(s)
        } else {
            self.list.offset = 0;
            Some(marked.len().saturating_sub(1))
        };
        let block = Block::default()
            .title(title.as_str())
            .border_style(*border_style)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let list_area = if self.has_focus {
            let (help_line_area, list_area) = {
                let help_at_bottom = selected.unwrap_or(0).saturating_sub(self.list.offset)
                    >= inner_area.height.saturating_sub(1) as usize / 2;
                let constraints = {
                    let mut c = vec![Constraint::Length(1), Constraint::Max(256)];
                    if help_at_bottom {
                        c.reverse();
                    }
                    c
                };
                let regions = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(inner_area);

                if help_at_bottom {
                    (regions[1], regions[0])
                } else {
                    (regions[0], regions[1])
                }
            };

            let default_style = Style {
                fg: Color::Black.into(),
                bg: Color::Yellow.into(),
                add_modifier: Modifier::BOLD,
                sub_modifier: Modifier::empty(),
                ..Style::default()
            };
            let t = language.ui_text();
            if let Some(notice) = safety_notice {
                Paragraph::new(*notice).render(help_line_area, buf);
            } else {
                Paragraph::new(Text::from(Line::from(vec![
                    #[cfg(feature = "trash-move")]
                    Span::styled(
                        format!(" {} ", keys.trash_marked),
                        Style {
                            fg: Color::White.into(),
                            bg: Color::Black.into(),
                            ..default_style
                        },
                    ),
                    #[cfg(feature = "trash-move")]
                    Span::styled(t.mark_to_trash_or, default_style),
                    Span::styled(
                        format!(" {} ", keys.delete_marked),
                        Style {
                            fg: Color::LightRed.into(),
                            bg: Color::Black.into(),
                            add_modifier: default_style.add_modifier | Modifier::RAPID_BLINK,
                            ..default_style
                        },
                    ),
                    Span::styled(t.mark_to_delete, default_style),
                ])))
                .style(default_style)
                .render(help_line_area, buf);
            }
            list_area
        } else {
            inner_area
        };

        let line_count = marked.len();
        let props = ListProps {
            block: None,
            entry_in_view,
        };
        self.list.render(props, entries, list_area, buf);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        let mut scrollbar_state =
            ScrollbarState::new(line_count).position(selected.unwrap_or(self.list.offset));

        scrollbar.render(
            {
                let mut scrollbar_area = list_area;
                // The list has no blocks, so we need to increase
                // the render area for scrollbar to make sure it
                // will be drawn on the border.
                scrollbar_area.width += 1;
                scrollbar_area
            },
            buf,
            &mut scrollbar_state,
        );

        if has_focus {
            let t = language.ui_text();
            let help_text = format!(
                " ⇊ = {}|↓ = {}|⇈ = {}|↑ = {} ",
                keys.page_down.primary(),
                keys.move_down.primary(),
                keys.page_up.primary(),
                keys.move_up.primary(),
            );
            let help_text_block_width = block_width(&help_text);
            let bound = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            if block_width(&title) + help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    &help_text,
                    |_, _, _| Style::default(),
                );
            }
            let bound = line_bound(bound, bound.height.saturating_sub(1) as usize);
            let help_text = format!(
                " {} = {} | {} = {}",
                t.mark_toggle, keys.remove_mark, t.mark_remove_all, keys.remove_all_marks
            );
            let help_text_block_width = block_width(&help_text);
            if *allow_changes && help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    &help_text,
                    |_, _, _| Style::default(),
                );
            }
        }
    }
}

pub fn calculate_size_and_count(marked: &EntryMarkMap) -> (u128, u64) {
    let entries: Vec<&EntryMark> = marked
        .values()
        .sorted_by(|a, b| Ord::cmp(&a.path, &b.path))
        .collect();

    let mut size = 0u128;
    let mut item_count = 0u64;
    for (idx, entry) in entries.iter().enumerate() {
        let mut is_subdirectory = false;
        for other in &entries[0..idx] {
            if other.is_dir && entry.path.starts_with(&other.path) {
                is_subdirectory = true;
                break;
            }
        }
        if !is_subdirectory {
            size += entry.size;
            item_count += entry.entry_count.unwrap_or(1);
        }
    }
    (size, item_count)
}

#[cfg(test)]
mod mark_pane_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::buffer::Cell;

    #[test]
    fn title_shows_percentage_of_root_size() {
        let area = Rect::new(0, 0, 80, 4);
        let mut buffer = Buffer::empty(area);
        MarkPane {
            total_size: 312_400_000,
            item_count: 20_000,
            ..Default::default()
        }
        .render(
            MarkPaneProps {
                border_style: Style::default(),
                format: ByteFormat::Metric,
                keys: &KeysConfig::default(),
                root_total_size: 1_000_000_000,
                safety_notice: None,
                allow_changes: true,
                language: Language::English,
            },
            area,
            &mut buffer,
        );

        insta::assert_debug_snapshot!(
            buffer,
            "marked item count, size and percentage of root size",
            @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 80, height: 4 },
            content: [
                "┌Marked 20K items (312.40 MB, 31.24% of 1.00 GB) ──────────────────────────────┐",
                "│                                                                              │",
                "│                                                                              │",
                "└──────────────────────────────────────────────────────────────────────────────┘",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#
        );
    }

    #[test]
    fn unmapped_delete_key_is_named() {
        let area = Rect::new(0, 0, 80, 4);
        let mut buffer = Buffer::empty(area);
        let config: dua::Config =
            toml::from_str("[keys]\ndelete_marked = []").expect("valid config");

        MarkPane {
            has_focus: true,
            ..Default::default()
        }
        .render(
            MarkPaneProps {
                border_style: Style::default(),
                format: ByteFormat::Metric,
                keys: &config.keys,
                root_total_size: 0,
                safety_notice: None,
                allow_changes: true,
                language: Language::English,
            },
            area,
            &mut buffer,
        );

        #[cfg(feature = "trash-move")]
        insta::assert_debug_snapshot!(
            buffer,
            "unmapped permanent-delete key alongside trash support",
            @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 80, height: 4 },
            content: [
                "┌Marked 0 items (0  B, 0.00% of 0  B) ── ⇊ = Ctrl + d|↓ = j|⇈ = Ctrl + u|↑ = k ┐",
                "│                                                                              │",
                "│ Ctrl + t  to trash or  <unmapped>  to delete without prompt                  │",
                "└─────────────────────────────────── mark-toggle = x/d/<Space> | remove-all = a┘",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 1, y: 2, fg: White, bg: Black, underline: Reset, modifier: BOLD,
                x: 11, y: 2, fg: Black, bg: Yellow, underline: Reset, modifier: BOLD,
                x: 24, y: 2, fg: LightRed, bg: Black, underline: Reset, modifier: BOLD | RAPID_BLINK,
                x: 36, y: 2, fg: Black, bg: Yellow, underline: Reset, modifier: BOLD,
                x: 79, y: 2, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#
        );
        #[cfg(not(feature = "trash-move"))]
        insta::assert_debug_snapshot!(
            buffer,
            "unmapped permanent-delete key without trash support",
            @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 80, height: 4 },
            content: [
                "┌Marked 0 items (0  B, 0.00% of 0  B) ── ⇊ = Ctrl + d|↓ = j|⇈ = Ctrl + u|↑ = k ┐",
                "│                                                                              │",
                "│ <unmapped>  to delete without prompt                                         │",
                "└─────────────────────────────────── mark-toggle = x/d/<Space> | remove-all = a┘",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 1, y: 2, fg: LightRed, bg: Black, underline: Reset, modifier: BOLD | RAPID_BLINK,
                x: 13, y: 2, fg: Black, bg: Yellow, underline: Reset, modifier: BOLD,
                x: 79, y: 2, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#
        );
    }

    #[test]
    fn safety_notice_replaces_the_danger_prompt() {
        let area = Rect::new(0, 0, 80, 4);
        let mut buffer = Buffer::empty(area);

        MarkPane {
            has_focus: true,
            ..Default::default()
        }
        .render(
            MarkPaneProps {
                border_style: Style::default(),
                format: ByteFormat::Metric,
                keys: &KeysConfig::default(),
                root_total_size: 0,
                safety_notice: Some(" Snapshot is read-only; marked entries cannot be deleted "),
                allow_changes: true,
                language: Language::English,
            },
            area,
            &mut buffer,
        );

        insta::assert_debug_snapshot!(
            buffer,
            "read-only notice replaces destructive prompt and styling",
            @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 80, height: 4 },
            content: [
                "┌Marked 0 items (0  B, 0.00% of 0  B) ── ⇊ = Ctrl + d|↓ = j|⇈ = Ctrl + u|↑ = k ┐",
                "│                                                                              │",
                "│ Snapshot is read-only; marked entries cannot be deleted                      │",
                "└─────────────────────────────────── mark-toggle = x/d/<Space> | remove-all = a┘",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#
        );
    }

    #[test]
    fn title_prompt_and_actions_follow_the_selected_language() {
        let area = Rect::new(0, 0, 120, 4);
        let mut buffer = Buffer::empty(area);

        MarkPane {
            has_focus: true,
            ..Default::default()
        }
        .render(
            MarkPaneProps {
                border_style: Style::default(),
                format: ByteFormat::Metric,
                keys: &KeysConfig::default(),
                root_total_size: 0,
                safety_notice: None,
                allow_changes: true,
                language: Language::Korean,
            },
            area,
            &mut buffer,
        );

        let rendered: String = buffer.content.iter().map(Cell::symbol).collect();
        let rendered: String = rendered.split_whitespace().collect();
        assert!(rendered.contains("표시된항목0개"));
        assert!(rendered.contains("확인없이삭제"));
        assert!(rendered.contains("표시전환"));
        assert!(rendered.contains("모두해제"));
        assert!(!rendered.contains("Marked"));
    }

    #[test]
    fn process_events_uses_configured_keybindings() {
        let config: dua::Config = toml::from_str(
            r#"
            [keys]
            delete_marked = ["z"]
            "#,
        )
        .expect("valid config");

        assert!(matches!(
            MarkPane::default().process_events(KeyCode::Char('z').into(), &config.keys, true),
            Some((_, Some(MarkMode::Delete)))
        ));
        assert!(matches!(
            MarkPane::default().process_events(
                KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
                &config.keys,
                true
            ),
            Some((_, None))
        ));
    }

    #[test]
    fn busy_pane_preserves_marks_and_errors_but_allows_navigation() {
        let keys = KeysConfig::default();
        let mut pane = MarkPane {
            selected: Some(0),
            has_focus: true,
            marked: [
                (
                    TreeIndex::new(1),
                    EntryMark {
                        index: 1,
                        path: PathBuf::from("one"),
                        num_errors_during_deletion: 3,
                        ..Default::default()
                    },
                ),
                (
                    TreeIndex::new(2),
                    EntryMark {
                        index: 2,
                        path: PathBuf::from("two"),
                        ..Default::default()
                    },
                ),
            ]
            .into(),
            ..Default::default()
        };
        for key in [
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
            KeyCode::Char('a').into(),
            KeyCode::Char('x').into(),
            KeyCode::Char('d').into(),
            KeyCode::Char(' ').into(),
        ] {
            let (next, action) = pane.process_events(key, &keys, false).expect("marks stay");
            pane = next;
            assert!(action.is_none());
            assert_eq!(pane.marked.len(), 2);
            assert_eq!(pane.selected, Some(0));
            assert_eq!(
                pane.marked[&TreeIndex::new(1)].num_errors_during_deletion,
                3
            );
        }
        let (mut pane, action) = pane
            .process_events(KeyCode::Char('j').into(), &keys, false)
            .expect("navigation keeps marks");
        assert!(action.is_none());
        assert_eq!(pane.selected, Some(1));

        let area = Rect::new(0, 0, 120, 5);
        let mut buffer = Buffer::empty(area);
        pane.render(
            MarkPaneProps {
                border_style: Style::default(),
                format: ByteFormat::Metric,
                keys: &keys,
                root_total_size: 0,
                safety_notice: Some(Language::English.ui_text().deletion_running),
                allow_changes: false,
                language: Language::English,
            },
            area,
            &mut buffer,
        );
        let rendered: String = buffer.content.iter().map(Cell::symbol).collect();
        assert!(rendered.contains("Deletion in progress; changes are disabled"));
        assert!(!rendered.contains("mark-toggle"));
        assert!(!rendered.contains("remove-all"));
        assert!(!rendered.contains("to delete without prompt"));
    }

    #[test]
    fn reconcile_updates_remaining_sizes_and_preserves_selection_identity() {
        use dua::traverse::{EntryData, Traversal};

        let mut traversal = Traversal::new();
        let root = traversal.root_index;
        let deleted = traversal
            .tree
            .add_child(root, "deleted", EntryData::default());
        let survivor = traversal.tree.add_child(
            root,
            "survivor",
            EntryData {
                size: 40,
                entry_count: Some(2),
                is_dir: true,
                ..Default::default()
            },
        );
        let stale = traversal
            .tree
            .add_child(root, "stale", EntryData::default());
        let view = TreeView {
            traversal: &mut traversal,
            scope: None,
            glob_tree_root: None,
            glob_matches: None,
        };
        let mut pane = MarkPane::default();
        for index in [deleted, survivor, stale] {
            pane = pane
                .toggle_index(index, &view, index == survivor, true)
                .unwrap();
        }
        pane.selected = Some(1);
        pane.set_deletion_error(survivor, 2);
        view.traversal.tree.remove_subtree(deleted);
        view.traversal.tree.remove_subtree(stale);
        let replacement = view
            .traversal
            .tree
            .add_child(root, "replacement", EntryData::default());
        assert_eq!(replacement, stale, "exercise a recycled tree index");
        view.traversal.tree.update(survivor, |data| {
            data.size = 28;
            data.entry_count = Some(1);
        });

        pane.reconcile(&view);
        assert_eq!(pane.marked.len(), 1);
        assert_eq!(pane.selected, Some(0));
        assert_eq!(pane.tree_index_by_list_position(0), Some(survivor));
        assert_eq!(pane.total_size(), 28);
        assert_eq!(pane.item_count, 1);
        assert_eq!(pane.marked[&survivor].num_errors_during_deletion, 2);

        view.traversal.tree.remove_subtree(survivor);
        pane.reconcile(&view);
        assert!(pane.is_empty());
        assert_eq!(pane.selected, None);
        assert_eq!((pane.total_size(), pane.item_count), (0, 0));
    }

    #[test]
    fn test_calculate_size() {
        let mut marked = EntryMarkMap::new();

        marked.insert(
            TreeIndex::new(0),
            EntryMark {
                size: 2,
                path: PathBuf::from("root/test1"),
                ..Default::default()
            },
        );
        marked.insert(
            TreeIndex::new(1),
            EntryMark {
                size: 10,
                path: PathBuf::from("root"),
                is_dir: true,
                entry_count: Some(2),
                ..Default::default()
            },
        );
        marked.insert(
            TreeIndex::new(2),
            EntryMark {
                size: 5,
                path: PathBuf::from("root1"),
                ..Default::default()
            },
        );
        marked.insert(
            TreeIndex::new(3),
            EntryMark {
                size: 2,
                path: PathBuf::from("root/test2"),
                ..Default::default()
            },
        );

        assert_eq!(calculate_size_and_count(&marked), (15u128, 3u64));
    }
}
