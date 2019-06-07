use crate::{
    interactive::widgets::{COLOR_BYTESIZE_SELECTED, COLOR_MARKED_LIGHT},
    interactive::{fit_string_graphemes_with_ellipsis, CursorDirection},
};
use dua::{
    path_of,
    traverse::{Tree, TreeIndex},
    ByteFormat,
};
use itertools::Itertools;
use std::{borrow::Borrow, collections::btree_map::Entry, collections::BTreeMap, path::PathBuf};
use termion::{event::Key, event::Key::*};
use tui::{
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::Block,
    widgets::Borders,
    widgets::Text,
    widgets::{Paragraph, Widget},
};
use tui_react::{List, ListProps};
use unicode_segmentation::UnicodeSegmentation;

pub type EntryMarkMap = BTreeMap<TreeIndex, EntryMark>;
pub struct EntryMark {
    pub size: u64,
    pub path: PathBuf,
    pub index: usize,
}

#[derive(Default)]
pub struct MarkPane {
    selected: Option<usize>,
    marked: EntryMarkMap,
    list: List,
    has_focus: bool,
    last_sorting_index: usize,
}

pub struct MarkPaneProps {
    pub border_style: Style,
    pub format: ByteFormat,
}

impl MarkPane {
    #[cfg(test)]
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }
    pub fn set_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
        if has_focus {
            self.selected = Some(self.marked.len().saturating_sub(1));
        } else {
            self.selected = None
        }
    }
    pub fn toggle_index(mut self, index: TreeIndex, tree: &Tree) -> Option<Self> {
        match self.marked.entry(index) {
            Entry::Vacant(entry) => {
                if let Some(e) = tree.node_weight(index) {
                    let sorting_index = self.last_sorting_index + 1;
                    self.last_sorting_index = sorting_index;
                    entry.insert(EntryMark {
                        size: e.size,
                        path: path_of(tree, index),
                        index: sorting_index,
                    });
                }
            }
            Entry::Occupied(entry) => {
                entry.remove();
            }
        };
        if self.marked.is_empty() {
            None
        } else {
            Some(self)
        }
    }
    pub fn marked(&self) -> &EntryMarkMap {
        &self.marked
    }
    pub fn key(mut self, key: Key) -> Option<Self> {
        match key {
            Char('d') | Char(' ') => return self.remove_selected_and_move_down(),
            Ctrl('u') | PageUp => self.change_selection(CursorDirection::PageUp),
            Char('k') | Up => self.change_selection(CursorDirection::Up),
            Char('j') | Down => self.change_selection(CursorDirection::Down),
            Ctrl('d') | PageDown => self.change_selection(CursorDirection::PageDown),
            _ => {}
        };
        Some(self)
    }

    fn remove_selected_and_move_down(mut self) -> Option<Self> {
        if let Some(selected) = self.selected {
            let (idx, se_len) = {
                let sorted_entries: Vec<_> = self
                    .marked
                    .iter()
                    .sorted_by_key(|(_, v)| &v.index)
                    .collect();
                (
                    sorted_entries.get(selected).map(|(k, _)| *k.to_owned()),
                    sorted_entries.len(),
                )
            };
            if let Some(idx) = idx {
                self.marked.remove(&idx);
                if se_len.saturating_sub(1) == 0 {
                    return None;
                }
                self.selected = Some(selected.saturating_sub(1));
            }
            Some(self)
        } else {
            Some(self)
        }
    }

    fn change_selection(&mut self, direction: CursorDirection) {
        self.selected = self.selected.map(|selected| {
            direction
                .move_cursor(selected)
                .min(self.marked.len().saturating_sub(1))
        });
    }

    pub fn render(&mut self, props: impl Borrow<MarkPaneProps>, area: Rect, buf: &mut Buffer) {
        let MarkPaneProps {
            border_style,
            format,
        } = props.borrow();

        let marked: &_ = &self.marked;
        let title = format!(
            "Marked {} items ({}) ",
            marked.len(),
            format.display(marked.iter().map(|(_k, v)| v.size).sum::<u64>())
        );
        let selected = self.selected;
        let entries = marked
            .values()
            .sorted_by_key(|v| &v.index)
            .enumerate()
            .map(|(idx, v)| {
                let (default_style, is_selected) = match selected {
                    Some(selected) if idx == selected => (
                        Style {
                            bg: Color::White,
                            ..Default::default()
                        },
                        true,
                    ),
                    _ => (Style::default(), false),
                };
                let (path, path_len) = {
                    let path = format!(" {}  ", v.path.display());
                    let num_path_graphemes = path.graphemes(true).count();
                    match num_path_graphemes + format.total_width() {
                        n if n > area.width as usize => {
                            let desired_size = num_path_graphemes - (n - area.width as usize);
                            fit_string_graphemes_with_ellipsis(
                                path,
                                num_path_graphemes,
                                desired_size,
                            )
                        }
                        _ => (path, num_path_graphemes),
                    }
                };
                let path = Text::Styled(
                    path.into(),
                    Style {
                        fg: if is_selected {
                            Color::Black
                        } else {
                            COLOR_MARKED_LIGHT
                        },
                        ..default_style
                    },
                );
                let bytes = Text::Styled(
                    format!(
                        "{:>byte_column_width$}",
                        format.display(v.size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
                        byte_column_width = format.width()
                    )
                    .into(),
                    Style {
                        fg: if is_selected {
                            COLOR_BYTESIZE_SELECTED
                        } else {
                            Color::Green
                        },
                        ..default_style
                    },
                );
                let spacer = Text::Styled(
                    format!(
                        "{:-space$}",
                        "",
                        space = (area.width as usize)
                            .saturating_sub(path_len)
                            .saturating_sub(format.total_width())
                    )
                    .into(),
                    default_style,
                );
                vec![path, spacer, bytes]
            });

        let entry_in_view = match self.selected {
            Some(s) => Some(s),
            None => {
                self.list.offset = 0;
                Some(marked.len().saturating_sub(1))
            }
        };
        let mut block = Block::default()
            .title(&title)
            .border_style(*border_style)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        block.draw(area, buf);

        let list_area = if self.has_focus {
            let (help_line_area, list_area) = {
                let help_at_bottom =
                    selected.unwrap_or(0) >= inner_area.height.saturating_sub(1) as usize / 2;
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

                match help_at_bottom {
                    true => (regions[1], regions[0]),
                    false => (regions[0], regions[1]),
                }
            };

            let default_style = Style {
                fg: Color::Black,
                bg: Color::White,
                modifier: Modifier::BOLD,
                ..Default::default()
            };
            Paragraph::new(
                [
                    Text::Styled(
                        " Ctrl + Shift + r".into(),
                        Style {
                            fg: Color::Red,
                            modifier: default_style.modifier | Modifier::RAPID_BLINK,
                            ..default_style
                        },
                    ),
                    Text::Styled(
                        " deletes listed entries from disk without prompt".into(),
                        default_style,
                    ),
                ]
                .iter(),
            )
            .style(Style {
                bg: Color::White,
                ..Style::default()
            })
            .draw(help_line_area, buf);
            list_area
        } else {
            inner_area
        };

        let props = ListProps {
            block: None,
            entry_in_view,
        };
        self.list.render(props, entries, list_area, buf)
    }
}
