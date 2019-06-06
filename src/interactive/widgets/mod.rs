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
pub const COLOR_MARKED_LIGHT: Color = Color::LightYellow;
pub const COLOR_MARKED_DARK: Color = Color::Rgb(176, 126, 0);
pub const COLOR_MARKED_DARKER: Color = Color::Rgb(106, 66, 0);
