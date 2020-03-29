use crate::draw_text_without_ellipsis_nowrap;
use crate::util::rect::offset_x;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Text, Widget},
};

#[derive(Default)]
pub struct List {
    /// The index at which the list last started. Used for scrolling
    pub offset: usize,
}

impl List {
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
pub struct ListProps<'b> {
    pub block: Option<Block<'b>>,
    pub entry_in_view: Option<usize>,
}

impl List {
    pub fn render<'a, 't>(
        &mut self,
        props: ListProps<'a>,
        items: impl IntoIterator<Item = Vec<Text<'t>>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let ListProps {
            block,
            entry_in_view,
        } = props;

        let list_area = match block {
            Some(mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };
        self.offset = self.list_offset_for(entry_in_view, list_area.height as usize);

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
            let mut bound = Rect {
                x,
                y,
                width: list_area.width,
                height: 1,
            };
            for text in text_iterator.into_iter() {
                let (text, style) = match text {
                    Text::Raw(s) => (s, Style::default()),
                    Text::Styled(s, style) => (s, style),
                };
                let offset = draw_text_without_ellipsis_nowrap(bound, buf, text, Some(style));
                bound = offset_x(bound, offset);
            }
        }
    }
}
