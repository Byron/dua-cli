mod app {
    use crate::{WalkOptions, WalkResult};
    use failure::Error;
    use petgraph::{Directed, Graph};
    use std::{ffi::OsString, io, path::PathBuf};
    use termion::input::{Keys, TermReadEventsAndRaw};
    use tui::{backend::Backend, Terminal};

    pub type GraphIndexType = u32;
    pub type Tree = Graph<ItemData, (), Directed, GraphIndexType>;

    #[derive(Eq, PartialEq, Debug)]
    pub struct ItemData {
        pub name: OsString,
        pub size: u64,
    }

    #[derive(Default, Debug)]
    pub struct App {
        pub tree: Tree,
    }

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
            let tree = Tree::new();
            Ok(App { tree })
        }
    }
}

pub use self::app::*;
