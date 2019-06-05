//! Derived from TUI-rs, license: MIT, Copyright (c) 2016 Florian Dehau
use std::borrow::Borrow;
use std::marker::PhantomData;
use tui::{
    buffer::Buffer, layout::Rect, style::Color, style::Style, symbols::line, widgets::Borders,
};

pub fn fill_background(area: Rect, buf: &mut Buffer, color: Color) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf.get_mut(x, y).set_bg(color);
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Block<'a, T>(PhantomData<&'a T>);

pub struct BlockProps<'a> {
    /// Optional title place on the upper left of the block
    pub title: Option<&'a str>,
    /// Title style
    pub title_style: Style,
    /// Visible borders
    pub borders: Borders,
    /// Border style
    pub border_style: Style,
    /// Widget style
    pub style: Style,
}

impl<'a> Default for BlockProps<'a> {
    fn default() -> BlockProps<'a> {
        BlockProps {
            title: None,
            title_style: Default::default(),
            borders: Borders::NONE,
            border_style: Default::default(),
            style: Default::default(),
        }
    }
}

impl<'a> BlockProps<'a> {
    /// Compute the inner area of a block based on its border visibility rules.
    pub fn inner(&self, area: Rect) -> Rect {
        if area.width < 2 || area.height < 2 {
            return Rect::default();
        }
        let mut inner = area;
        if self.borders.intersects(Borders::LEFT) {
            inner.x += 1;
            inner.width -= 1;
        }
        if self.borders.intersects(Borders::TOP) || self.title.is_some() {
            inner.y += 1;
            inner.height -= 1;
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width -= 1;
        }
        if self.borders.intersects(Borders::BOTTOM) {
            inner.height -= 1;
        }
        inner
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Block::<()>::default().render(self, area, buf);
    }
}

impl<'a, T> Block<'a, T> {
    fn render(&self, props: impl Borrow<BlockProps<'a>>, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 2 {
            return;
        }
        let BlockProps {
            title,
            title_style,
            borders,
            border_style,
            style,
        } = props.borrow();

        fill_background(area, buf, style.bg);

        // Sides
        if borders.intersects(Borders::LEFT) {
            for y in area.top()..area.bottom() {
                buf.get_mut(area.left(), y)
                    .set_symbol(line::VERTICAL)
                    .set_style(*border_style);
            }
        }
        if borders.intersects(Borders::TOP) {
            for x in area.left()..area.right() {
                buf.get_mut(x, area.top())
                    .set_symbol(line::HORIZONTAL)
                    .set_style(*border_style);
            }
        }
        if borders.intersects(Borders::RIGHT) {
            let x = area.right() - 1;
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(line::VERTICAL)
                    .set_style(*border_style);
            }
        }
        if borders.intersects(Borders::BOTTOM) {
            let y = area.bottom() - 1;
            for x in area.left()..area.right() {
                buf.get_mut(x, y)
                    .set_symbol(line::HORIZONTAL)
                    .set_style(*border_style);
            }
        }

        // Corners
        if borders.contains(Borders::LEFT | Borders::TOP) {
            buf.get_mut(area.left(), area.top())
                .set_symbol(line::TOP_LEFT)
                .set_style(*border_style);
        }
        if borders.contains(Borders::RIGHT | Borders::TOP) {
            buf.get_mut(area.right() - 1, area.top())
                .set_symbol(line::TOP_RIGHT)
                .set_style(*border_style);
        }
        if borders.contains(Borders::LEFT | Borders::BOTTOM) {
            buf.get_mut(area.left(), area.bottom() - 1)
                .set_symbol(line::BOTTOM_LEFT)
                .set_style(*border_style);
        }
        if borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            buf.get_mut(area.right() - 1, area.bottom() - 1)
                .set_symbol(line::BOTTOM_RIGHT)
                .set_style(*border_style);
        }

        if area.width > 2 {
            if let Some(title) = title {
                let lx = if borders.intersects(Borders::LEFT) {
                    1
                } else {
                    0
                };
                let rx = if borders.intersects(Borders::RIGHT) {
                    1
                } else {
                    0
                };
                let width = area.width - lx - rx;
                buf.set_stringn(
                    area.left() + lx,
                    area.top(),
                    title,
                    width as usize,
                    *title_style,
                );
            }
        }
    }
}
