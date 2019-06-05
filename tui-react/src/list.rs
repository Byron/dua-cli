use super::BlockProps;
use std::borrow::Borrow;
use std::iter::repeat;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Text, Widget},
};

pub fn fill_background_to_right(mut s: String, entire_width: u16) -> String {
    match (s.len(), entire_width as usize) {
        (x, y) if x >= y => s,
        (x, y) => {
            s.extend(repeat(' ').take(y - x));
            s
        }
    }
}

#[derive(Default)] // TODO: remove Clone derive
pub struct ReactList {
    /// The index at which the list last started. Used for scrolling
    offset: usize,
}

impl ReactList {
    fn list_offset_for(&self, entry_in_view: Option<usize>, height: usize) -> usize {
        match entry_in_view {
            Some(pos) => match height as usize {
                h if self.offset + h - 1 < pos => pos - h + 1,
                _ if self.offset > pos => pos,
                _ => self.offset,
            },
            None => 0,
        }
    }
}

#[derive(Default)]
pub struct ReactListProps<'b> {
    pub block: Option<BlockProps<'b>>,
    pub entry_in_view: Option<usize>,
}

impl ReactList {
    pub fn render<'a, 't>(
        &mut self,
        props: impl Borrow<ReactListProps<'a>>,
        items: impl IntoIterator<Item = Vec<Text<'t>>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let ReactListProps {
            block,
            entry_in_view,
        } = props.borrow();

        let list_area = match block {
            Some(b) => {
                b.render(area, buf);
                b.inner(area)
            }
            None => area,
        };
        self.offset = self.list_offset_for(*entry_in_view, list_area.height as usize);

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        for (i, text_iterator) in items
            .into_iter()
            .skip(self.offset)
            .enumerate()
            .take(list_area.height as usize)
        {
            let (x, y) = (list_area.left(), list_area.top() + i as u16);
            Paragraph::new(text_iterator.iter()).draw(
                Rect {
                    x,
                    y,
                    width: list_area.width,
                    height: 1,
                },
                buf,
            );
        }
    }
}
