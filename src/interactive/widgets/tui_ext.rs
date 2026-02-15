use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, ListItem, ListState},
};
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct List {
    pub offset: usize,
}

pub struct ListProps<'a> {
    pub block: Option<Block<'a>>,
    pub entry_in_view: Option<usize>,
}

impl List {
    pub fn render<'a>(
        &mut self,
        props: ListProps<'_>,
        entries: impl IntoIterator<Item = impl Into<Line<'a>>>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let mut state = ListState::default().with_offset(self.offset);
        if let Some(selected) = props.entry_in_view {
            state.select(Some(selected));
        }
        let mut list = tui::widgets::List::new(
            entries
                .into_iter()
                .map(|entry| ListItem::new(tui::text::Text::from(entry.into()))),
        );
        if let Some(block) = props.block {
            list = list.block(block);
        }
        tui::widgets::StatefulWidget::render(list, area, buf, &mut state);
        self.offset = state.offset();
    }
}

pub fn draw_text_nowrap_fn(
    area: Rect,
    buf: &mut Buffer,
    text: &str,
    mut style_fn: impl FnMut(usize, usize, char) -> Style,
) {
    let mut col = 0usize;
    for (x, ch) in text.chars().enumerate() {
        if col >= area.width as usize {
            break;
        }
        let cell = buf.get_mut(area.x.saturating_add(col as u16), area.y);
        cell.set_char(ch);
        let style = style_fn(x, 0, ch);
        cell.set_style(style);
        let width = UnicodeWidthChar::width(ch).unwrap_or(1);
        for continuation in 1..width {
            if col + continuation >= area.width as usize {
                break;
            }
            let cell = buf.get_mut(area.x.saturating_add((col + continuation) as u16), area.y);
            cell.set_char(' ');
            cell.set_style(style);
        }
        col += width;
    }
}

pub mod util {
    use super::*;

    pub fn block_width(text: &str) -> u16 {
        text.width() as u16
    }

    pub mod rect {
        use super::*;

        pub fn snap_to_right(area: Rect, width: u16) -> Rect {
            Rect {
                x: area.x + area.width.saturating_sub(width),
                width: width.min(area.width),
                ..area
            }
        }

        pub fn line_bound(area: Rect, line: usize) -> Rect {
            Rect {
                y: area.y + (line as u16).min(area.height.saturating_sub(1)),
                height: 1,
                ..area
            }
        }
    }
}
