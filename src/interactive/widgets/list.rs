use std::iter::repeat;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Text, Widget},
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

#[derive(Default, Clone)] // TODO: remove Clone derive
pub struct ListState {
    /// The index at which the list last started. Used for scrolling
    pub start_index: usize,
}

impl ListState {
    pub fn update(&mut self, selected: Option<usize>, height: usize) -> &mut Self {
        self.start_index = match selected {
            Some(pos) => match height as usize {
                h if self.start_index + h - 1 < pos => pos - h + 1,
                _ if self.start_index > pos => pos,
                _ => self.start_index,
            },
            None => 0,
        };
        self
    }
}

pub struct List<'b, 't, I>
where
    I: Iterator<Item = Vec<Text<'t>>>,
{
    pub block: Option<Block<'b>>,
    pub items: I,
}

impl<'b, 't, I> Widget for List<'b, 't, I>
where
    I: Iterator<Item = Vec<Text<'t>>>,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let list_area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        for (i, text_iterator) in self
            .items
            .by_ref()
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
