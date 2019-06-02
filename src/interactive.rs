mod app {
    use crate::{WalkOptions, WalkResult};
    use failure::Error;
    use std::io;
    use std::path::PathBuf;
    use termion::input::{Keys, TermReadEventsAndRaw};
    use tui::{backend::Backend, Terminal};

    pub struct App {}

    impl App {
        pub fn process_events<B, R>(
            &mut self,
            terminal: &mut Terminal<B>,
            keys: Keys<R>,
        ) -> Result<WalkResult, Error>
        where
            B: Backend,
            R: io::Read + TermReadEventsAndRaw,
        {
            unimplemented!()
        }

        pub fn initialize<B>(
            terminal: &mut Terminal<B>,
            options: WalkOptions,
            input: Vec<PathBuf>,
        ) -> Result<App, Error>
        where
            B: Backend,
        {
            Ok(App {})
        }
    }
}

pub use self::app::*;
