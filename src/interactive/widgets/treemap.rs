//! A `WinDirStat`/`QDirStat`-like treemap of the current directory.
//!
//! Every entry of the current view gets a rectangle whose area is proportional to its size.
//! Like `QDirStat`'s minimum tile size, entries too small for a readable tile are lumped into one
//! `+N` tile, which can be entered (zoomed into) to see them laid out over the whole screen.
//!
//! Drawing happens on a pixel canvas with twice the vertical resolution of the terminal:
//! each cell shows two pixels using the upper half block `▀` (foreground = top pixel,
//! background = bottom pixel), so tile edges are only half a cell thick.

use crate::interactive::EntryDataBundle;
use crate::interactive::widgets::COLOR_MARKED;
use dua::traverse::{Tree, TreeIndex};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
};

/// A tile needs about this many cells in width to show a name and a size.
const MIN_TILE_WIDTH: u16 = 14;
/// Pixels (half cells) a tile needs to be readable: three text rows plus edges.
const MIN_TILE_AREA: u32 = MIN_TILE_WIDTH as u32 * 8;

const DIR_BG: Color = Color::Rgb(70, 105, 160);
/// Directories take the colour of the extension filling at least this share of them.
const DOMINANT_SHARE: f64 = 0.4;
/// Directories are darker than files of the same kind.
const DIR_SHADE: f64 = 0.72;
/// Stop counting a directory's contents after this many nodes; the sample decides its colour.
const MAX_SCAN_NODES: usize = 200_000;
const GAP: Color = Color::Rgb(14, 14, 18);
const LUMP_BG: Color = Color::Rgb(85, 85, 85);
const NO_EXT_BG: Color = Color::Rgb(150, 150, 150);
const STATUS_BG: Color = Color::Rgb(40, 42, 52);

const PALETTE: [(u8, u8, u8); 14] = [
    (230, 85, 85),
    (240, 150, 60),
    (230, 205, 70),
    (150, 200, 80),
    (70, 180, 110),
    (60, 190, 180),
    (110, 170, 230),
    (130, 130, 235),
    (165, 110, 220),
    (210, 95, 190),
    (235, 120, 150),
    (180, 140, 100),
    (120, 170, 160),
    (200, 170, 220),
];

/// Inputs needed to draw the treemap.
pub struct TreemapProps<'a> {
    /// Path of the current directory.
    pub path: &'a Path,
    /// The directory the entries belong to; zooming is reset when it changes.
    pub view_root: TreeIndex,
    /// Entries of the current view.
    pub entries: &'a [EntryDataBundle],
    /// The selected entry of the current view.
    pub selected: Option<TreeIndex>,
    /// Entries of the current view that are marked.
    pub marked: HashSet<TreeIndex>,
    pub format: dua::ByteFormat,
    /// Text shown on the right of the status line, like key hints or a message.
    pub status_right: Option<(String, Style)>,
    /// The traversal tree, used to colour directories by their content.
    pub tree: Option<&'a Tree>,
}

/// What a tile on screen stands for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tile {
    Entry(TreeIndex),
    /// The entries from this position of the size-sorted list onwards.
    Lump {
        start: usize,
    },
}

/// Spatial movement between tiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileDirection {
    Left,
    Right,
    Up,
    Down,
}

/// The treemap's layout and zoom, kept between frames for keyboard navigation.
#[derive(Default)]
pub struct Treemap {
    root: Option<TreeIndex>,
    /// Entries with a size, largest first.
    sorted: Vec<TreeIndex>,
    /// Start positions in `sorted` of the lumps zoomed into, innermost last.
    zoom: Vec<usize>,
    /// Tiles of the last frame in pixel coordinates (x in cells, y in half cells).
    layout: Vec<(Tile, Rect, u128)>,
    /// Dominant extension and its share per directory, valid while the directory's size is.
    kinds: HashMap<TreeIndex, (u128, Option<(String, f64)>)>,
}

impl Treemap {
    fn tile_of(&self, selected: Option<TreeIndex>) -> Option<usize> {
        let selected = selected?;
        let position = self.sorted.iter().position(|idx| *idx == selected);
        self.layout.iter().position(|(tile, ..)| match tile {
            Tile::Entry(idx) => *idx == selected,
            Tile::Lump { start } => position.is_some_and(|p| p >= *start),
        })
    }

    fn entry_of(&self, tile: Tile) -> Option<TreeIndex> {
        match tile {
            Tile::Entry(idx) => Some(idx),
            Tile::Lump { start } => self.sorted.get(start).copied(),
        }
    }

    /// The entry to select when moving from `selected` in `direction`, or the first one if
    /// nothing visible is selected yet.
    pub fn neighbour(
        &self,
        selected: Option<TreeIndex>,
        direction: TileDirection,
    ) -> Option<TreeIndex> {
        let Some(current) = self.tile_of(selected) else {
            return self
                .layout
                .first()
                .and_then(|(tile, ..)| self.entry_of(*tile));
        };
        let from = self.layout[current].1;
        let overlap = |a0: u16, a1: u16, b0: u16, b1: u16| a1.min(b1).saturating_sub(a0.max(b0));
        // Centres in doubled pixel units to stay integral.
        let centre = |r: Rect| {
            (
                i32::from(r.x) * 2 + i32::from(r.width),
                i32::from(r.y) * 2 + i32::from(r.height),
            )
        };
        let (fx, fy) = centre(from);
        self.layout
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != current)
            .filter_map(|(_, (tile, to, _))| {
                let (distance, shared) = match direction {
                    TileDirection::Right if to.left() >= from.right() => (
                        to.left() - from.right(),
                        overlap(from.top(), from.bottom(), to.top(), to.bottom()),
                    ),
                    TileDirection::Left if to.right() <= from.left() => (
                        from.left() - to.right(),
                        overlap(from.top(), from.bottom(), to.top(), to.bottom()),
                    ),
                    TileDirection::Down if to.top() >= from.bottom() => (
                        to.top() - from.bottom(),
                        overlap(from.left(), from.right(), to.left(), to.right()),
                    ),
                    TileDirection::Up if to.bottom() <= from.top() => (
                        from.top() - to.bottom(),
                        overlap(from.left(), from.right(), to.left(), to.right()),
                    ),
                    _ => return None,
                };
                let (tx, ty) = centre(*to);
                let off_axis = match direction {
                    TileDirection::Left | TileDirection::Right => (ty - fy).abs(),
                    TileDirection::Up | TileDirection::Down => (tx - fx).abs(),
                };
                // Prefer tiles that share an edge, then the nearest, then the best aligned.
                Some(((shared == 0, distance, off_axis), *tile))
            })
            .min_by_key(|(key, _)| *key)
            .and_then(|(_, tile)| self.entry_of(tile))
    }

    /// The extension taking up most of directory `idx`, if it fills at least [`DOMINANT_SHARE`].
    fn dominant_kind(&mut self, tree: &Tree, idx: TreeIndex, size: u128) -> Option<(String, f64)> {
        if let Some((cached_size, kind)) = self.kinds.get(&idx)
            && *cached_size == size
        {
            return kind.clone();
        }
        let mut by_extension: HashMap<String, u128> = HashMap::new();
        let mut counted: u128 = 0;
        let mut stack = vec![idx];
        let mut visited = 0;
        while let Some(node) = stack.pop() {
            visited += 1;
            if visited > MAX_SCAN_NODES {
                break;
            }
            for child in tree.children(node) {
                let Some(data) = tree.data(child) else {
                    continue;
                };
                if data.is_dir {
                    stack.push(child);
                } else if data.size > 0 {
                    counted += data.size;
                    if let Some(ext) = tree.name(child).and_then(|name| {
                        let name = name.file_name()?.to_string_lossy().into_owned();
                        let (stem, ext) = name.rsplit_once('.')?;
                        (!stem.is_empty() && !ext.is_empty()).then(|| ext.to_lowercase())
                    }) {
                        *by_extension.entry(ext).or_default() += data.size;
                    }
                }
            }
        }
        let kind = by_extension
            .into_iter()
            .max_by_key(|(_, bytes)| *bytes)
            .map(|(ext, bytes)| (ext, bytes as f64 / counted.max(1) as f64))
            .filter(|(_, share)| *share >= DOMINANT_SHARE);
        self.kinds.insert(idx, (size, kind.clone()));
        kind
    }

    /// If `selected` is inside the lump, zoom into the lump and return `true`.
    pub fn zoom_in(&mut self, selected: Option<TreeIndex>) -> bool {
        match self.tile_of(selected).map(|i| self.layout[i].0) {
            Some(Tile::Lump { start }) => {
                self.zoom.push(start);
                true
            }
            _ => false,
        }
    }

    /// Leave the innermost zoomed lump, returning `false` if there is none.
    pub fn zoom_out(&mut self) -> bool {
        self.zoom.pop().is_some()
    }

    /// Draw the map into `area`, with a status line at the bottom if `status` is set.
    pub fn render(&mut self, props: TreemapProps<'_>, area: Rect, buf: &mut Buffer, status: bool) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let (map, status_area) = if status && area.height > 1 {
            (
                Rect {
                    height: area.height - 1,
                    ..area
                },
                Some(Rect {
                    y: area.bottom() - 1,
                    height: 1,
                    ..area
                }),
            )
        } else {
            (area, None)
        };

        let mut entries: Vec<&EntryDataBundle> =
            props.entries.iter().filter(|e| e.size > 0).collect();
        entries.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));
        self.sorted = entries.iter().map(|e| e.index).collect();
        if self.root != Some(props.view_root) {
            self.root = Some(props.view_root);
            self.zoom.clear();
            self.kinds.clear();
        }
        while self
            .zoom
            .last()
            .is_some_and(|start| *start >= entries.len())
        {
            self.zoom.pop();
        }
        let skip = self.zoom.last().copied().unwrap_or(0);
        let visible = &entries[skip..];
        let total: u128 = props.entries.iter().map(|e| e.size).sum();

        let mut canvas = Canvas::new(map.width, map.height);
        let bounds = canvas.bounds();
        let tiles = readable(visible, skip, bounds);
        let weights: Vec<f64> = tiles.iter().map(|(_, size)| *size as f64).collect();
        self.layout.clear();
        for ((tile, size), frect) in tiles
            .into_iter()
            .zip(squarify(&weights, FRect::from(bounds)))
        {
            let rect = frect.snap();
            if rect.is_empty() {
                continue;
            }
            self.layout.push((tile, rect, size));
            let size_text = props.format.display(size).to_string();
            let pct = percent(size, total);
            match tile {
                Tile::Entry(idx) => {
                    let entry = visible
                        .iter()
                        .find(|e| e.index == idx)
                        .expect("tiles come from visible entries");
                    let name = display_name(entry);
                    if entry.is_dir {
                        let kind = props
                            .tree
                            .and_then(|tree| self.dominant_kind(tree, idx, entry.size));
                        let (color, kind_text) = match kind {
                            Some((ext, share)) => (
                                darken(extension_color(&format!("x.{ext}")), DIR_SHADE),
                                format!("▪ .{ext} {:.0}%", share * 100.0),
                            ),
                            None => (DIR_BG, String::new()),
                        };
                        canvas.tile(rect, color, &[&name, &size_text, &pct, &kind_text]);
                    } else {
                        canvas.tile(rect, extension_color(&name), &[&name, &size_text, &pct]);
                    }
                }
                Tile::Lump { start } => {
                    let count = format!("+{} …", entries.len() - start);
                    canvas.tile(rect, LUMP_BG, &[&count, &size_text, &pct]);
                }
            }
        }

        let selected = self.tile_of(props.selected);
        for (i, (tile, rect, _)) in self.layout.iter().enumerate() {
            let marked = match tile {
                Tile::Entry(idx) => props.marked.contains(idx),
                Tile::Lump { start } => self.sorted[*start..]
                    .iter()
                    .any(|i| props.marked.contains(i)),
            };
            if Some(i) == selected {
                canvas.outline(*rect, if marked { COLOR_MARKED } else { Color::White });
            } else if marked {
                canvas.outline(*rect, COLOR_MARKED);
            }
        }
        canvas.draw(map, buf);

        if let Some(status_area) = status_area {
            self.render_status(&props, entries.len(), total, status_area, buf);
        }
    }

    fn render_status(
        &self,
        props: &TreemapProps<'_>,
        entry_count: usize,
        total: u128,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let base = Style::default().fg(Color::White).bg(STATUS_BG);
        buf.set_style(area, base);
        let mut left = format!(" {}", props.path.display());
        for start in &self.zoom {
            left.push_str(&format!(" › +{}", entry_count - start));
        }
        let detail = match self.tile_of(props.selected).map(|i| self.layout[i]) {
            Some((Tile::Lump { start }, _, size)) => {
                Some((format!("+{} …", entry_count - start), size))
            }
            _ => props
                .selected
                .and_then(|idx| props.entries.iter().find(|e| e.index == idx))
                .map(|entry| (display_name(entry), entry.size)),
        }
        .map(|(name, size)| {
            format!(
                "  ▸ {name}  {}  {}",
                props.format.display(size).to_string().trim(),
                percent(size, total),
            )
        });
        let right_width = props.status_right.as_ref().map_or(0, |(text, _)| {
            unicode_width::UnicodeWidthStr::width(text.as_str()) as u16 + 1
        });
        let left_end = area.right().saturating_sub(right_width);
        let (x, _) = buf.set_stringn(
            area.x,
            area.y,
            &left,
            usize::from(left_end.saturating_sub(area.x)),
            base.add_modifier(Modifier::BOLD),
        );
        if let Some(detail) = detail {
            buf.set_stringn(
                x,
                area.y,
                &detail,
                usize::from(left_end.saturating_sub(x)),
                base.fg(Color::Yellow),
            );
        }
        if let Some((text, style)) = &props.status_right
            && right_width < area.width
        {
            buf.set_string(left_end + 1, area.y, text, base.patch(*style));
        }
    }
}

fn display_name(entry: &EntryDataBundle) -> String {
    let name = entry.name.file_name().map_or_else(
        || entry.name.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    if entry.is_dir {
        format!("{name}/")
    } else {
        name
    }
}

fn percent(size: u128, total: u128) -> String {
    if total == 0 {
        String::new()
    } else {
        format!("{:.1}%", size as f64 * 100.0 / total as f64)
    }
}

/// Tiles for the `entries` (sorted largest first, starting at `skip` in the full list) that would
/// be readable in `area`, with the rest lumped into one tile.
fn readable(entries: &[&EntryDataBundle], skip: usize, area: Rect) -> Vec<(Tile, u128)> {
    let total: u128 = entries.iter().map(|e| e.size).sum();
    if total == 0 || pixels(area) == 0 {
        return Vec::new();
    }
    let min_size = (total * u128::from(MIN_TILE_AREA)).div_ceil(u128::from(pixels(area)));
    let mut keep = entries.iter().take_while(|e| e.size >= min_size).count();
    // A lump too small to read swallows the smallest kept entries until it can be seen and
    // entered. At least one entry stays, so zooming into a lump always makes progress.
    let lump = |keep: usize| entries[keep..].iter().map(|e| e.size).sum::<u128>();
    while keep > 1 && lump(keep) > 0 && lump(keep) < min_size {
        keep -= 1;
    }
    let keep = keep.max(1);
    let mut tiles: Vec<(Tile, u128)> = entries[..keep]
        .iter()
        .map(|e| (Tile::Entry(e.index), e.size))
        .collect();
    if keep < entries.len() {
        tiles.push((Tile::Lump { start: skip + keep }, lump(keep)));
    }
    tiles
}

fn pixels(rect: Rect) -> u32 {
    u32::from(rect.width) * u32::from(rect.height)
}

/// Split `text` into graphemes that fit into `width` cells. Text that is too wide is cut with `…`
/// if there is room for a few characters, and dropped otherwise.
fn fit(text: &str, width: u16) -> Option<Vec<(&str, u16)>> {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;
    let mut graphemes: Vec<(&str, u16)> = text
        .graphemes(true)
        .map(|g| (g, g.width() as u16))
        .filter(|(_, w)| *w > 0)
        .collect();
    if graphemes.iter().map(|(_, w)| w).sum::<u16>() > width {
        if width < 6 {
            return None;
        }
        let mut used = 0;
        graphemes.retain(|(_, w)| {
            used += w;
            used < width
        });
        graphemes.push(("…", 1));
    }
    Some(graphemes)
}

/// Pixels are one cell wide and half a cell tall. Rectangles on it are in pixel coordinates.
struct Canvas {
    width: u16,
    height: u16,
    pixels: Vec<Color>,
    /// Per cell: a colour for its right half, used for thin vertical edges.
    right: Vec<Option<Color>>,
    /// Per cell: a colour for its left half, used for thin vertical edges.
    left: Vec<Option<Color>>,
    /// Per cell: text that replaces the half blocks.
    text: Vec<Option<(String, Style)>>,
}

impl Canvas {
    fn new(cells_w: u16, cells_h: u16) -> Self {
        let cells = usize::from(cells_w) * usize::from(cells_h);
        Canvas {
            width: cells_w,
            height: cells_h * 2,
            pixels: vec![Color::Reset; cells * 2],
            right: vec![None; cells],
            left: vec![None; cells],
            text: vec![None; cells],
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    fn cell(&self, x: u16, row: u16) -> usize {
        usize::from(row) * usize::from(self.width) + usize::from(x)
    }

    fn set(&mut self, x: u16, y: u16, color: Color) {
        if x < self.width && y < self.height {
            self.pixels[usize::from(y) * usize::from(self.width) + usize::from(x)] = color;
        }
    }

    /// Colour one half of the cells in column `x` along `rect`.
    fn vertical(&mut self, rect: Rect, x: u16, color: Color, right_half: bool) {
        for row in rect.y / 2..rect.bottom().div_ceil(2) {
            if x < self.width && row < self.height / 2 {
                let cell = self.cell(x, row);
                if right_half {
                    self.right[cell] = Some(color);
                } else {
                    self.left[cell] = Some(color);
                }
            }
        }
    }

    /// A coloured tile with a thin dark gap on its right and bottom, and `lines` of text in it.
    fn tile(&mut self, rect: Rect, color: Color, lines: &[&str]) {
        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                self.set(x, y, color);
            }
        }
        if rect.height >= 2 {
            for x in rect.left()..rect.right() {
                self.set(x, rect.bottom() - 1, GAP);
            }
        }
        if rect.width >= 2 {
            self.vertical(rect, rect.right() - 1, GAP, true);
        }
        // Text rows keep the top and bottom pixel rows free for edges and outlines, and one
        // column of padding on either side.
        let rows = (rect.y + 1).div_ceil(2)..rect.bottom().saturating_sub(1) / 2;
        let style = Style::default().fg(text_color(color)).bg(color);
        for (row, line) in rows.zip(lines) {
            let Some(graphemes) = fit(line.trim(), rect.width.saturating_sub(2)) else {
                break;
            };
            let mut x = rect.x + 1;
            for (grapheme, w) in graphemes {
                let cell = self.cell(x, row);
                self.text[cell] = Some((grapheme.to_owned(), style));
                for extra in 1..w {
                    let cell = self.cell(x + extra, row);
                    self.text[cell] = Some((String::new(), style));
                }
                x += w;
            }
        }
    }

    /// A thin frame just inside `rect`, next to its gap edges.
    fn outline(&mut self, rect: Rect, color: Color) {
        let inner = Rect {
            width: rect.width.saturating_sub(1).max(1),
            height: rect.height.saturating_sub(1).max(1),
            ..rect
        };
        for x in inner.left()..inner.right() {
            self.set(x, inner.top(), color);
            self.set(x, inner.bottom() - 1, color);
        }
        self.vertical(inner, inner.left(), color, false);
        self.vertical(inner, inner.right() - 1, color, true);
    }

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        for row in 0..self.height / 2 {
            for x in 0..self.width {
                let cell_idx = self.cell(x, row);
                let px = usize::from(row) * 2 * usize::from(self.width) + usize::from(x);
                let top = self.pixels[px];
                let bottom = self.pixels[px + usize::from(self.width)];
                let cell = &mut buf[(area.x + x, area.y + row)];
                cell.reset();
                if let Some((symbol, style)) = &self.text[cell_idx] {
                    // The cell after a wide grapheme stays reset, like ratatui does it.
                    if !symbol.is_empty() {
                        cell.set_symbol(symbol).set_style(*style);
                    }
                } else if let (Some(color), true) = (self.right[cell_idx], top == bottom) {
                    cell.set_char('▌').set_fg(top).set_bg(color);
                } else if let (Some(color), true) = (self.left[cell_idx], top == bottom) {
                    cell.set_char('▐').set_fg(top).set_bg(color);
                } else if top == bottom {
                    cell.set_char(' ').set_bg(top);
                } else {
                    cell.set_char('▀').set_fg(top).set_bg(bottom);
                }
            }
        }
    }
}

fn extension_color(name: &str) -> Color {
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return NO_EXT_BG;
    };
    if stem.is_empty() || ext.is_empty() {
        return NO_EXT_BG;
    }
    // FNV-1a, stable between runs so a given extension always has the same colour.
    let hash = ext
        .to_lowercase()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
    let (r, g, b) = PALETTE[(hash % PALETTE.len() as u64) as usize];
    Color::Rgb(r, g, b)
}

#[expect(clippy::cast_sign_loss, reason = "clamped to the u8 range")]
fn darken(color: Color, factor: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let f = |c: u8| (f64::from(c) * factor).clamp(0.0, 255.0) as u8;
            Color::Rgb(f(r), f(g), f(b))
        }
        other => other,
    }
}

fn text_color(bg: Color) -> Color {
    match bg {
        Color::Rgb(r, g, b)
            if 299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b) > 140_000 =>
        {
            Color::Black
        }
        _ => Color::White,
    }
}

/// A rectangle in pixel units.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl From<Rect> for FRect {
    fn from(rect: Rect) -> Self {
        FRect {
            x: f64::from(rect.x),
            y: f64::from(rect.y),
            w: f64::from(rect.width),
            h: f64::from(rect.height),
        }
    }
}

impl FRect {
    /// Snap to the pixel grid. Both edges are rounded, so neighbours tile without gaps or overlap.
    #[expect(clippy::cast_sign_loss, reason = "clamped to the u16 range")]
    fn snap(self) -> Rect {
        let snap = |v: f64| v.round().clamp(0.0, f64::from(u16::MAX)) as u16;
        let (x0, x1) = (snap(self.x), snap(self.x + self.w));
        let (y0, y1) = (snap(self.y), snap(self.y + self.h));
        Rect::new(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
    }
}

/// Squarified treemap layout (Bruls, Huizing, van Wijk) of `weights`, which must be sorted
/// in descending order. Returns one rectangle per weight.
fn squarify(weights: &[f64], area: FRect) -> Vec<FRect> {
    let total: f64 = weights.iter().sum();
    let empty = FRect {
        w: 0.0,
        h: 0.0,
        ..area
    };
    if total <= 0.0 || area.w <= 0.0 || area.h <= 0.0 {
        return vec![empty; weights.len()];
    }
    let scale = area.w * area.h / total;
    let areas: Vec<f64> = weights.iter().map(|w| w * scale).collect();
    let mut out = Vec::with_capacity(areas.len());
    let mut rect = area;
    let mut start = 0;
    while start < areas.len() {
        let side = rect.w.min(rect.h);
        let mut end = start + 1;
        let mut sum = areas[start];
        let mut best = worst(&areas[start..end], sum, side);
        while end < areas.len() {
            let candidate = sum + areas[end];
            let ratio = worst(&areas[start..=end], candidate, side);
            if ratio > best {
                break;
            }
            best = ratio;
            sum = candidate;
            end += 1;
        }
        if rect.w >= rect.h {
            let width = if rect.h > 0.0 { sum / rect.h } else { 0.0 };
            let mut y = rect.y;
            for a in &areas[start..end] {
                let h = if width > 0.0 { a / width } else { 0.0 };
                out.push(FRect {
                    x: rect.x,
                    y,
                    w: width,
                    h,
                });
                y += h;
            }
            rect.x += width;
            rect.w = (rect.w - width).max(0.0);
        } else {
            let height = if rect.w > 0.0 { sum / rect.w } else { 0.0 };
            let mut x = rect.x;
            for a in &areas[start..end] {
                let w = if height > 0.0 { a / height } else { 0.0 };
                out.push(FRect {
                    x,
                    y: rect.y,
                    w,
                    h: height,
                });
                x += w;
            }
            rect.y += height;
            rect.h = (rect.h - height).max(0.0);
        }
        start = end;
    }
    out
}

/// The worst aspect ratio of a row of `areas` summing to `sum`, laid along `side`.
fn worst(areas: &[f64], sum: f64, side: f64) -> f64 {
    let (min, max) = areas
        .iter()
        .fold((f64::INFINITY, 0.0_f64), |(min, max), &a| {
            (min.min(a), max.max(a))
        });
    if min <= 0.0 || sum <= 0.0 || side <= 0.0 {
        return f64::INFINITY;
    }
    let (s2, side2) = (sum * sum, side * side);
    (side2 * max / s2).max(s2 / (side2 * min))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(sizes: &[(&str, u128)]) -> Vec<EntryDataBundle> {
        sizes
            .iter()
            .enumerate()
            .map(|(i, (name, size))| EntryDataBundle {
                index: TreeIndex::new(i),
                name: name.trim_end_matches('/').into(),
                size: *size,
                mtime: std::time::UNIX_EPOCH,
                entry_count: name.ends_with('/').then_some(0),
                is_dir: name.ends_with('/'),
                exists: true,
            })
            .collect()
    }

    fn render(
        map: &mut Treemap,
        entries: &[EntryDataBundle],
        selected: usize,
        w: u16,
        h: u16,
    ) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        map.render(
            TreemapProps {
                path: Path::new("/x"),
                view_root: TreeIndex::new(1000),
                entries,
                selected: Some(TreeIndex::new(selected)),
                marked: HashSet::new(),
                format: dua::ByteFormat::Bytes,
                status_right: None,
                tree: None,
            },
            area,
            &mut buf,
            true,
        );
        buf
    }

    fn text(buf: &Buffer) -> String {
        let area = buf.area;
        (area.top()..area.bottom())
            .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
            .map(|p| buf[p].symbol().to_owned())
            .collect()
    }

    #[test]
    fn squarify_tiles_the_whole_area_without_overlap() {
        let area = Rect::new(1, 2, 37, 22);
        let weights = [50.0, 20.0, 10.0, 10.0, 5.0, 3.0, 1.0, 1.0];
        let rects: Vec<Rect> = squarify(&weights, FRect::from(area))
            .into_iter()
            .map(FRect::snap)
            .collect();
        let mut covered = vec![0u8; usize::from(area.width) * usize::from(area.height)];
        for rect in &rects {
            assert!(area.union(*rect) == area, "{rect:?} escapes {area:?}");
            for y in rect.top()..rect.bottom() {
                for x in rect.left()..rect.right() {
                    covered[usize::from(y - area.y) * usize::from(area.width)
                        + usize::from(x - area.x)] += 1;
                }
            }
        }
        assert!(
            covered.iter().all(|&c| c == 1),
            "every pixel is covered exactly once"
        );
        assert!(
            pixels(rects[0]) > pixels(rects[1]),
            "bigger weights get bigger tiles"
        );
    }

    #[test]
    fn extensions_have_stable_colours_and_files_without_one_are_grey() {
        assert_eq!(extension_color("a.RS"), extension_color("b.rs"));
        assert_eq!(extension_color("Makefile"), NO_EXT_BG);
        assert_eq!(extension_color(".bashrc"), NO_EXT_BG);
    }

    #[test]
    fn small_entries_are_lumped_and_the_lump_zooms_to_the_whole_map() {
        let mut sizes = vec![("big.iso", 10_000), ("dir/", 5_000)];
        let small: Vec<String> = (0..50).map(|i| format!("f{i:02}.txt")).collect();
        sizes.extend(
            small
                .iter()
                .map(|n| (n.as_str(), 100 - small.len() as u128)),
        );
        let entries = entries(&sizes);
        let mut map = Treemap::default();
        let screen = text(&render(&mut map, &entries, 0, 60, 20));
        assert!(
            screen.contains("big.iso") && screen.contains("dir/"),
            "{screen}"
        );
        assert!(screen.contains("+50"), "{screen}");
        assert!(screen.contains("/x  ▸ big.iso"), "status shows selection");
        assert_eq!(map.layout.len(), 3);

        let in_lump = TreeIndex::new(2);
        assert!(map.zoom_in(Some(in_lump)));
        let screen = text(&render(&mut map, &entries, 2, 60, 20));
        assert!(
            !screen.contains("big.iso"),
            "zoomed view only shows the lump"
        );
        assert!(screen.contains("f00.txt"), "{screen}");
        assert!(screen.contains("/x › +50"), "status shows the zoom");
        assert!(map.zoom_out());
        assert!(!map.zoom_out());
    }

    #[test]
    fn arrows_move_between_neighbouring_tiles() {
        let entries = entries(&[("a.txt", 100), ("b.txt", 100)]);
        let mut map = Treemap::default();
        render(&mut map, &entries, 0, 40, 10);
        let (a, b) = (TreeIndex::new(0), TreeIndex::new(1));
        let (to_b, back) = if map.layout[0].1.x == map.layout[1].1.x {
            (TileDirection::Down, TileDirection::Up)
        } else {
            (TileDirection::Right, TileDirection::Left)
        };
        assert_eq!(map.neighbour(Some(a), to_b), Some(b));
        assert_eq!(map.neighbour(Some(b), back), Some(a));
        assert_eq!(
            map.neighbour(Some(a), back),
            None,
            "nothing beyond the edge"
        );
        assert_eq!(
            map.neighbour(None, back),
            Some(a),
            "start at the biggest tile"
        );
    }

    #[test]
    fn directories_take_the_colour_of_what_fills_them() {
        use dua::traverse::EntryData;
        let file = |size| EntryData {
            size,
            ..EntryData::default()
        };
        let dir = || EntryData {
            is_dir: true,
            ..EntryData::default()
        };
        let mut tree = Tree::new();
        let root = tree.add_root("root", dir());
        let libs = tree.add_child(root, "libs", dir());
        tree.add_child(libs, "a.SO", file(600));
        let nested = tree.add_child(libs, "nested", dir());
        tree.add_child(nested, "b.so", file(300));
        tree.add_child(libs, "notes.txt", file(100));
        let mixed = tree.add_child(root, "mixed", dir());
        for (i, ext) in ["a", "b", "c", "d"].iter().enumerate() {
            tree.add_child(mixed, format!("f{i}.{ext}"), file(100));
        }

        let mut map = Treemap::default();
        let (ext, share) = map.dominant_kind(&tree, libs, 1000).expect("mostly .so");
        assert_eq!(ext, "so");
        assert!((share - 0.9).abs() < 1e-9);
        assert_eq!(
            map.dominant_kind(&tree, mixed, 400),
            None,
            "no extension dominates"
        );
    }

    #[test]
    fn rendering_survives_tiny_areas() {
        let entries = entries(&[("a.txt", 100), ("b/", 50), ("c", 1)]);
        for (w, h) in [(0, 0), (1, 1), (3, 2), (10, 4)] {
            render(&mut Treemap::default(), &entries, 0, w, h);
        }
    }
}
