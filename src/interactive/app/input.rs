use crossbeam::channel::Receiver;
pub use crossterm::event::{Event, KeyCode};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// The latest known focus state of the terminal running the interactive UI.
///
/// The application state keeps one clone while another is moved to the input
/// thread. That thread records focus events as soon as they are read, including
/// while the main event loop is busy with synchronous work or its bounded input
/// queue is full. Focus events are consumed by the tracker instead of occupying
/// queue capacity. Completion handlers then consult this state to avoid sending
/// notifications while the terminal is focused.
#[derive(Clone)]
pub struct TerminalFocus(Arc<AtomicBool>);

impl Default for TerminalFocus {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

impl TerminalFocus {
    pub fn is_focussed(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn observe(&self, event: &Event) {
        let value = match event {
            Event::FocusGained => true,
            Event::FocusLost => false,
            _ => return,
        };
        self.0.store(value, Ordering::Relaxed);
    }
}

/// Returns whether the input thread should continue reading events.
///
/// Return `false` if the channel was disconnected, so forwarding should stop.
fn forward_event(
    sender: &crossbeam::channel::Sender<Event>,
    focus: &TerminalFocus,
    event: Event,
) -> bool {
    focus.observe(&event);
    if matches!(event, Event::FocusGained | Event::FocusLost) {
        return true;
    }
    match sender.try_send(event) {
        Ok(()) | Err(crossbeam::channel::TrySendError::Full(_)) => true,
        Err(crossbeam::channel::TrySendError::Disconnected(_)) => false,
    }
}

pub fn input_channel(focus: TerminalFocus) -> Receiver<Event> {
    // Keep reading while the event loop performs synchronous deletion or trash work so the
    // shared focus state stays current, without allowing user input to grow without bound.
    let (key_send, key_receive) = crossbeam::channel::bounded(32);
    std::thread::spawn(move || -> Result<(), std::io::Error> {
        loop {
            let event = loop {
                match crossterm::event::read() {
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                    result => break result?,
                }
            };
            if !forward_event(&key_send, &focus, event) {
                break;
            }
        }
        Ok(())
    });
    key_receive
}

/// Return a receiver that yields one character-key event for each character in `input`.
pub fn input_channel_from_chars(input: &str) -> Receiver<Event> {
    let (key_send, key_receive) = crossbeam::channel::unbounded();
    for event in input
        .chars()
        .map(|character| Event::Key(KeyCode::Char(character).into()))
    {
        if key_send.send(event).is_err() {
            break;
        }
    }
    key_receive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_focus_tracker_observes_changes_immediately() {
        let app_focus = TerminalFocus::default();
        let input_focus = app_focus.clone();

        input_focus.observe(&Event::FocusLost);
        assert!(!app_focus.is_focussed());
        input_focus.observe(&Event::FocusGained);
        assert!(app_focus.is_focussed());
    }

    #[test]
    fn focus_changes_bypass_a_full_bounded_event_queue() {
        let focus = TerminalFocus::default();
        let (sender, receiver) = crossbeam::channel::bounded(1);
        assert!(forward_event(
            &sender,
            &focus,
            Event::Key(KeyCode::Char('a').into()),
        ));
        assert!(forward_event(&sender, &focus, Event::FocusLost));
        assert!(forward_event(&sender, &focus, Event::FocusGained));
        assert!(focus.is_focussed());
        assert!(matches!(receiver.try_recv(), Ok(Event::Key(_))));
        assert!(receiver.try_recv().is_err());
    }
}
