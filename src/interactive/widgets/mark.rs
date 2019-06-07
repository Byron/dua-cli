use crate::interactive::{widgets::COLOR_MARKED_LIGHT, CursorDirection};
use dua::traverse::{Tree, TreeIndex};
use dua::{path_of, ByteFormat};
use itertools::Itertools;
use std::collections::btree_map::Entry;
use std::{borrow::Borrow, collections::BTreeMap, path::PathBuf};
use termion::{event::Key, event::Key::*};
use tui::style::Color;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Block,
    widgets::Borders,
    widgets::Text,
};
use tui_react::{List, ListProps};

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
    pub fn key(&mut self, key: Key) {
        match key {
            Ctrl('u') | PageUp => self.change_selection(CursorDirection::PageUp),
            Char('k') | Up => self.change_selection(CursorDirection::Up),
            Char('j') | Down => self.change_selection(CursorDirection::Down),
            Ctrl('d') | PageDown => self.change_selection(CursorDirection::PageDown),
            _ => {}
        };
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
        let block = Block::default()
            .title(&title)
            .border_style(*border_style)
            .borders(Borders::ALL);
        let entry_in_view = match self.selected {
            Some(s) => Some(s),
            None => {
                self.list.offset = 0;
                Some(marked.len().saturating_sub(1))
            }
        };
        let selected = self.selected;
        let entries = marked
            .values()
            .sorted_by_key(|v| &v.index)
            .enumerate()
            .map(|(idx, v)| {
                let modifier = match selected {
                    Some(selected) if idx == selected => Modifier::BOLD,
                    _ => Modifier::empty(),
                };
                let path = format!(" {}", v.path.display());
                let path_len = path.len();
                let path = Text::Styled(
                    path.into(),
                    Style {
                        fg: COLOR_MARKED_LIGHT,
                        modifier,
                        ..Style::default()
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
                        fg: Color::Green,
                        ..Default::default()
                    },
                );
                let spacer = Text::Raw(
                    format!(
                        "{:-space$}",
                        "",
                        space = (area.width as usize)
                            .saturating_sub(path_len)
                            .saturating_sub(format.total_width())
                    )
                    .into(),
                );
                vec![path, spacer, bytes]
            });

        let props = ListProps {
            block: Some(block),
            entry_in_view,
        };
        self.list.render(props, entries, area, buf)
    }
}
