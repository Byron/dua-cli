mod entries;
mod footer;
mod glob;
mod header;
mod help;
mod main;
mod mark;
mod tui_ext;

pub use entries::*;
pub use footer::*;
pub use glob::*;
pub use header::*;
pub use help::*;
pub use main::*;
pub use mark::*;
use once_cell::sync::Lazy;

use tui::style::Color;

pub const COLOR_MARKED: Color = Color::Yellow;
pub const COLOR_MARKED_DARK: Color = Color::Rgb(176, 126, 0);

static COUNT: Lazy<human_format::Formatter> = Lazy::new(|| {
    let mut formatter = human_format::Formatter::new();
    formatter.with_decimals(0).with_separator("");
    formatter
});

fn entry_color(fg: Option<Color>, is_file: bool, is_marked: bool) -> Option<Color> {
    match (is_file, is_marked) {
        (true, false) => fg,
        (true, true) => COLOR_MARKED_DARK.into(),
        (false, true) => COLOR_MARKED.into(),
        (false, false) => Color::Cyan.into(),
    }
}
