mod entries;
mod footer;
mod header;
mod help;
mod main;
mod mark;

pub use entries::*;
pub use footer::*;
pub use header::*;
pub use help::*;
pub use main::*;
pub use mark::*;

use tui::style::Color;

pub const COLOR_MARKED: Color = Color::Yellow;
pub const COLOR_MARKED_DARK: Color = Color::Rgb(176, 126, 0);

fn entry_color(fg: Option<Color>, is_file: bool, is_marked: bool) -> Option<Color> {
    match (is_file, is_marked) {
        (true, false) => Color::DarkGray.into(),
        (true, true) => COLOR_MARKED_DARK.into(),
        (false, true) => COLOR_MARKED.into(),
        _ => fg,
    }
}
