use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Text, Widget},
};

pub struct List<'b, 't, I, T>
where
    I: Iterator<Item = Paragraph<'b, 't, T>>,
    T: Iterator<Item = &'t Text<'t>>,
{
    pub block: Option<Block<'b>>,
    pub items: I,
}

impl<'b, 't, I, T> Widget for List<'b, 't, I, T>
where
    I: Iterator<Item = Paragraph<'b, 't, T>>,
    T: Iterator<Item = &'t Text<'t>>,
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

        for (i, mut paragraph) in self
            .items
            .by_ref()
            .enumerate()
            .take(list_area.height as usize)
        {
            let (x, y) = (list_area.left(), list_area.top() + i as u16);
            paragraph.draw(
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
