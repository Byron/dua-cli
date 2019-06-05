#[allow(unused)]
mod terminal {
    use log::error;
    use std::io;

    use tui::{backend::Backend, buffer::Buffer, layout::Rect, widgets::Widget};

    #[derive(Debug)]
    pub struct Terminal<B>
    where
        B: Backend,
    {
        backend: B,
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

        pub fn backend(&self) -> &B {
            &self.backend
        }

        pub fn backend_mut(&mut self) -> &mut B {
            &mut self.backend
        }

        pub fn flush(&mut self) -> io::Result<()> {
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

        pub fn draw<F>(&mut self, f: F) -> io::Result<()>
        where
            F: FnOnce(),
        {
            // Autoresize - otherwise we get glitches if shrinking or potential desync between widgets
            // and the terminal (if growing), which may OOB.
            self.autoresize()?;

            f();

            self.flush()?;

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
        pub fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
            self.backend.get_cursor()
        }
        pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
            self.backend.set_cursor(x, y)
        }
        pub fn clear(&mut self) -> io::Result<()> {
            self.backend.clear()
        }
        pub fn size(&self) -> io::Result<Rect> {
            self.backend.size()
        }
    }
}

pub use terminal::*;
