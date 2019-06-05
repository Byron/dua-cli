use super::{BlockProps, Component};
use std::borrow::Borrow;
use std::iter::repeat;
use std::marker::PhantomData;
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
pub struct ReactList<'a, 'b, T> {
    /// The index at which the list last started. Used for scrolling
    start_index: usize,
    _a: PhantomData<&'a T>,
    _b: PhantomData<&'b T>,
}

impl<'a, 'b, T> ReactList<'a, 'b, T> {
    fn update_start_index(&mut self, selected: Option<usize>, height: usize) -> &mut Self {
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

pub struct ReactListProps<'b, 't> {
    pub block: Option<BlockProps<'b>>,
    pub items: Vec<Vec<Text<'t>>>,
}

impl<'b, 't, T> Component for ReactList<'b, 't, T> {
    type Props = ReactListProps<'b, 't>;

    fn render(&mut self, props: impl Borrow<Self::Props>, area: Rect, buf: &mut Buffer) {
        let ReactListProps { block, items } = props.borrow();

        let list_area = match block {
            Some(b) => {
                b.render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        for (i, text_iterator) in items.iter().enumerate().take(list_area.height as usize) {
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
