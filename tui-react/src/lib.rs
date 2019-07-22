#![forbid(unsafe_code)]

mod list;
mod terminal;

pub use list::*;
pub use terminal::*;

use std::iter::repeat;
use tui::{self, buffer::Buffer, layout::Rect, style::Color};

pub fn fill_background_to_right(mut s: String, entire_width: u16) -> String {
    match (s.len(), entire_width as usize) {
        (x, y) if x >= y => s,
        (x, y) => {
            s.extend(repeat(' ').take(y - x));
            s
        }
    }
}

/// Helper method to quickly set the background of all cells inside the specified area.
pub fn fill_background(area: Rect, buf: &mut Buffer, color: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_bg(color);
        }
    }
}
