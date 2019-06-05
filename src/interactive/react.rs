mod terminal {
    //! Derived from TUI-rs, license: MIT, Copyright (c) 2016 Florian Dehau
    use log::error;
    use std::{borrow::Borrow, io};

    use std::borrow::BorrowMut;
    use tui::{backend::Backend, buffer::Buffer, layout::Rect};

    pub trait Component {
        type Props;
        type PropsMut;

        fn render(
            &mut self,
            props: impl Borrow<Self::Props>,
            props_mut: impl BorrowMut<Self::PropsMut>,
            area: Rect,
            buf: &mut Buffer,
        );
    }

    #[derive(Debug)]
    pub struct Terminal<B>
    where
        B: Backend,
    {
        pub backend: B,
        buffers: [Buffer; 2],
        current: usize,
        hidden_cursor: bool,
        known_size: Rect,
    }

    impl<B> Drop for Terminal<B>
    where
        B: Backend,
    {
        fn drop(&mut self) {
            // Attempt to restore the cursor state
            if self.hidden_cursor {
                if let Err(err) = self.show_cursor() {
                    error!("Failed to show the cursor: {}", err);
                }
            }
        }
    }

    impl<B> Terminal<B>
    where
        B: Backend,
    {
        pub fn new(backend: B) -> io::Result<Terminal<B>> {
            let size = backend.size()?;
            Ok(Terminal {
                backend,
                buffers: [Buffer::empty(size), Buffer::empty(size)],
                current: 0,
                hidden_cursor: false,
                known_size: size,
            })
        }

        pub fn current_buffer_mut(&mut self) -> &mut Buffer {
            &mut self.buffers[self.current]
        }

        pub fn reconcile_and_flush(&mut self) -> io::Result<()> {
            let previous_buffer = &self.buffers[1 - self.current];
            let current_buffer = &self.buffers[self.current];
            let updates = previous_buffer.diff(current_buffer);
            self.backend.draw(updates.into_iter())
        }

        pub fn resize(&mut self, area: Rect) -> io::Result<()> {
            self.buffers[self.current].resize(area);
            self.buffers[1 - self.current].reset();
            self.buffers[1 - self.current].resize(area);
            self.known_size = area;
            self.backend.clear()
        }

        pub fn autoresize(&mut self) -> io::Result<()> {
            let size = self.size()?;
            if self.known_size != size {
                self.resize(size)?;
            }
            Ok(())
        }

        pub fn render<C>(
            &mut self,
            component: &mut C,
            props: impl Borrow<C::Props>,
            props_mut: impl BorrowMut<C::PropsMut>,
        ) -> io::Result<()>
        where
            C: Component,
        {
            // Autoresize - otherwise we get glitches if shrinking or potential desync between widgets
            // and the terminal (if growing), which may OOB.
            self.autoresize()?;

            component.render(props, props_mut, self.known_size, self.current_buffer_mut());

            self.reconcile_and_flush()?;

            self.buffers[1 - self.current].reset();
            self.current = 1 - self.current;

            self.backend.flush()?;
            Ok(())
        }

        pub fn hide_cursor(&mut self) -> io::Result<()> {
            self.backend.hide_cursor()?;
            self.hidden_cursor = true;
            Ok(())
        }
        pub fn show_cursor(&mut self) -> io::Result<()> {
            self.backend.show_cursor()?;
            self.hidden_cursor = false;
            Ok(())
        }
        #[allow(unused)]
        pub fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
            self.backend.get_cursor()
        }
        #[allow(unused)]
        pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
            self.backend.set_cursor(x, y)
        }
        #[allow(unused)]
        pub fn clear(&mut self) -> io::Result<()> {
            self.backend.clear()
        }
        pub fn size(&self) -> io::Result<Rect> {
            self.backend.size()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use tui::backend::TestBackend;

        #[derive(Default, Clone)]
        struct ComplexProps {
            x: usize,
            y: String,
        }

        #[derive(Default)]
        struct StatefulComponent {
            x: usize,
        }

        #[derive(Default)]
        struct StatelessComponent;

        impl Component for StatefulComponent {
            type Props = usize;
            type PropsMut = ();

            fn render(
                &mut self,
                props: impl Borrow<Self::Props>,
                _props_mut: impl BorrowMut<Self::PropsMut>,
                _area: Rect,
                _buf: &mut Buffer,
            ) {
                self.x += *props.borrow();
            }
        }

        impl Component for StatelessComponent {
            type Props = ComplexProps;
            type PropsMut = ();
            fn render(
                &mut self,
                props: impl Borrow<Self::Props>,
                _props_mut: impl BorrowMut<Self::PropsMut>,
                area: Rect,
                _buf: &mut Buffer,
            ) {
                // does not matter - we want to see it compiles essentially
            }
        }

        #[test]
        fn it_does_render_with_simple_and_complex_props() {
            let mut term = Terminal::new(TestBackend::new(20, 20)).unwrap();
            let mut c = StatefulComponent::default();

            term.render(&mut c, 3usize, ()).ok();
            assert_eq!(c.x, 3);

            let mut c = StatelessComponent::default();
            term.render(&mut c, ComplexProps::default(), ()).ok();
        }
    }
}

mod block {
    //! Derived from TUI-rs, license: MIT, Copyright (c) 2016 Florian Dehau
    use super::Component;
    use std::borrow::{Borrow, BorrowMut};
    use std::marker::PhantomData;
    use tui::{
        buffer::Buffer,
        layout::Rect,
        style::Color,
        style::Style,
        symbols::line,
        widgets::{Borders, Widget},
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
        title: Option<&'a str>,
        /// Title style
        title_style: Style,
        /// Visible borders
        borders: Borders,
        /// Border style
        border_style: Style,
        /// Widget style
        style: Style,
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
    }
    impl<'a> BlockProps<'a> {
        pub fn render(&self, area: Rect, buf: &mut Buffer) {
            Block::<()>::default().render(self, (), area, buf);
        }
    }

    impl<'a, T> Component for Block<'a, T> {
        type Props = BlockProps<'a>;
        type PropsMut = ();

        fn render(
            &mut self,
            props: impl Borrow<Self::Props>,
            _: impl BorrowMut<Self::PropsMut>,
            area: Rect,
            buf: &mut Buffer,
        ) {
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
}

mod list {
    use super::{Block, BlockProps, Component};
    use std::borrow::{Borrow, BorrowMut};
    use std::iter::repeat;
    use std::marker::PhantomData;
    use tui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{Paragraph, Text, Widget},
    };

    pub fn fill_background_to_right(mut s: String, entire_width: u16) -> String {
        match (s.len(), entire_width as usize) {
            (x, y) if x >= y => s,
            (x, y) => {
                s.extend(repeat(' ').take(y - x));
                s
            }
        }
    }

    #[derive(Default)] // TODO: remove Clone derive
    pub struct ReactList<'a, 'b, T> {
        /// The index at which the list last started. Used for scrolling
        start_index: usize,
        _a: PhantomData<&'a T>,
        _b: PhantomData<&'b T>,
    }

    impl<'a, 'b, T> ReactList<'a, 'b, T> {
        fn update_start_index(&mut self, selected: Option<usize>, height: usize) -> &mut Self {
            self.start_index = match selected {
                Some(pos) => match height as usize {
                    h if self.start_index + h - 1 < pos => pos - h + 1,
                    _ if self.start_index > pos => pos,
                    _ => self.start_index,
                },
                None => 0,
            };
            self
        }
    }

    pub struct ReactListProps<'b> {
        pub block: Option<BlockProps<'b>>,
    }

    pub struct ReactListPropsMut<'t, I>
    where
        I: Iterator<Item = Vec<Text<'t>>>,
    {
        pub items: I,
    }

    impl<'b, 't, I> Component for ReactList<'b, 't, I>
    where
        I: Iterator<Item = Vec<Text<'t>>>,
    {
        type Props = ReactListProps<'b>;
        type PropsMut = ReactListPropsMut<'t, I>;

        fn render(
            &mut self,
            props: impl Borrow<Self::Props>,
            mut props_mut: impl BorrowMut<Self::PropsMut>,
            area: Rect,
            buf: &mut Buffer,
        ) {
            let ReactListProps { block } = props.borrow();
            let ReactListPropsMut { ref mut items } = props_mut.borrow_mut();

            let list_area = match block {
                Some(b) => {
                    b.render(area, buf);
                    b.inner(area)
                }
                None => area,
            };

            if list_area.width < 1 || list_area.height < 1 {
                return;
            }

            for (i, text_iterator) in items.by_ref().enumerate().take(list_area.height as usize) {
                let (x, y) = (list_area.left(), list_area.top() + i as u16);
                Paragraph::new(text_iterator.iter()).draw(
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
}

pub use block::*;
pub use list::*;
pub use terminal::*;
