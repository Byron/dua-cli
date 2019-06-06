use crate::interactive::{widgets::COLOR_MARKED_LIGHT, CursorDirection, EntryMarkMap, Handle};
use dua::traverse::TreeIndex;
use itertools::Itertools;
use std::borrow::Borrow;
use termion::{event::Key, event::Key::*};
use tui::{
    buffer::Buffer, layout::Rect, style::Style, widgets::Block, widgets::Borders, widgets::Text,
};
use tui_react::{List, ListProps};

#[derive(Default)]
pub struct MarkPane {
    pub list: List,
    pub selected: Option<TreeIndex>,
}

pub struct MarkPaneProps<'a> {
    pub border_style: Style,
    pub marked: &'a EntryMarkMap,
}

impl Handle for MarkPane {
    fn key(&mut self, key: Key) {
        match key {
            Ctrl('u') | PageUp => self.change_selection(CursorDirection::PageUp),
            Char('k') | Up => self.change_selection(CursorDirection::Up),
            Char('j') | Down => self.change_selection(CursorDirection::Down),
            Ctrl('d') | PageDown => self.change_selection(CursorDirection::PageDown),
            _ => {}
        };
    }
}

impl MarkPane {
    fn change_selection(&mut self, _direction: CursorDirection) {}
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
        let entry_in_view = self.selected.and_then(|idx| {
            marked
                .iter()
                .enumerate()
                .find_position(|(_pos, (&node_index, _))| node_index == idx)
                .map(|(pos, _)| pos)
        });

        let entries = marked.iter().map(|(_, v)| {
            let name = Text::Styled(
                v.path.to_string_lossy(),
                Style {
                    fg: COLOR_MARKED_LIGHT,
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
