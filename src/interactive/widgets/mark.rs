use crate::interactive::{widgets::COLOR_MARKED_LIGHT, CursorDirection, EntryMarkMap};
use dua::traverse::TreeIndex;
use itertools::Itertools;
use std::borrow::Borrow;
use termion::{event::Key, event::Key::*};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Block,
    widgets::Borders,
    widgets::Text,
};
use tui_react::{List, ListProps};

pub struct MarkPane {
    list: List,
    selected: Option<TreeIndex>,
}

pub struct MarkPaneProps<'a> {
    pub border_style: Style,
    pub marked: &'a EntryMarkMap,
}

impl MarkPane {
    pub fn new(_marked: &EntryMarkMap) -> MarkPane {
        MarkPane {
            list: Default::default(),
            selected: None,
        }
    }

    pub fn key(&mut self, key: Key, marked: &EntryMarkMap) {
        match key {
            Ctrl('u') | PageUp => self.change_selection(CursorDirection::PageUp, marked),
            Char('k') | Up => self.change_selection(CursorDirection::Up, marked),
            Char('j') | Down => self.change_selection(CursorDirection::Down, marked),
            Ctrl('d') | PageDown => self.change_selection(CursorDirection::PageDown, marked),
            _ => {}
        };
    }

    fn change_selection(&mut self, _direction: CursorDirection, _marked: &EntryMarkMap) {}

    pub fn render<'a>(
        &mut self,
        props: impl Borrow<MarkPaneProps<'a>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let MarkPaneProps {
            border_style,
            marked,
        } = props.borrow();

        let block = Block::default()
            .title("Marked Entries")
            .border_style(*border_style)
            .borders(Borders::ALL);
        let entry_in_view = self
            .selected
            .and_then(|selected| {
                marked
                    .keys()
                    .find_position(|&&index| index == selected)
                    .map(|(pos, _)| pos)
            })
            .or_else(|| marked.keys().enumerate().last().map(|(pos, _)| pos));

        let selected = self.selected.clone();
        let entries = marked.iter().map(|(idx, v)| {
            let modifier = match selected {
                Some(selected) if *idx == selected => Modifier::BOLD,
                _ => Modifier::empty(),
            };
            let name = Text::Styled(
                v.path.to_string_lossy(),
                Style {
                    fg: COLOR_MARKED_LIGHT,
                    modifier,
                    ..Style::default()
                },
            );
            vec![name]
        });

        let props = ListProps {
            block: Some(block),
            entry_in_view,
        };
        self.list.render(props, entries, area, buf)
    }
}
