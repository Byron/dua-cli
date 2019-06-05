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
