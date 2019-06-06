use termion::event::Key;

pub trait Handle {
    fn key(&mut self, key: Key);
}
