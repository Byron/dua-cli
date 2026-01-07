use crossbeam::channel::Receiver;
pub use crossterm::event::Event;

enum Action<T> {
    Continue,
    Result(Result<T, std::io::Error>),
}

fn continue_on_interrupt<T>(result: Result<T, std::io::Error>) -> Action<T> {
    match result {
        Ok(v) => Action::Result(Ok(v)),
        Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Action::Continue,
        Err(err) => Action::Result(Err(err)),
    }
}

pub fn input_channel() -> Receiver<Event> {
    let (key_send, key_receive) = crossbeam::channel::bounded(0);
    std::thread::spawn(move || -> Result<(), std::io::Error> {
        loop {
            let event = match continue_on_interrupt(crossterm::event::read()) {
                Action::Continue => continue,
                Action::Result(res) => res?,
            };
            if key_send.send(event).is_err() {
                break;
            }
        }
        Ok(())
    });
    key_receive
}
