mod list;
mod terminal;

pub use list::*;
pub use terminal::*;

/// re-export our exact version, in case it matters to someone
pub use tui;

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Color;

/// Helper method to quickly set the background of all cells inside the specified area.
pub fn fill_background(area: Rect, buf: &mut Buffer, color: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_bg(color);
        }
    }
}
