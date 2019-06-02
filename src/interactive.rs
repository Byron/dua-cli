mod app {
    use crate::{WalkOptions, WalkResult};
    use failure::Error;
    use petgraph::{prelude::NodeIndex, Directed, Graph};
    use std::{ffi::OsString, io, path::PathBuf};
    use termion::input::{Keys, TermReadEventsAndRaw};
    use tui::{backend::Backend, Terminal};

    pub type GraphIndexType = u32;
    pub type Tree = Graph<EntryData, (), Directed, GraphIndexType>;

    #[derive(Eq, PartialEq, Debug, Default)]
    pub struct EntryData {
        pub name: OsString,
        /// The entry's size in bytes. If it's a directory, the size is the aggregated file size of all children
        pub size: u64,
        /// If set, the item meta-data could not be obtained
        pub metadata_io_error: bool,
    }

    /// State and methods representing the interactive disk usage analyser for the terminal
    #[derive(Default, Debug)]
    pub struct TerminalApp {
        /// A tree representing the entire filestem traversal
        pub tree: Tree,
        /// The top-level node of the tree.
        pub root_index: NodeIndex<GraphIndexType>,
        /// Amount of files or directories we have seen during the filesystem traversal
        pub entries_traversed: u64,
        /// Total amount of IO errors encountered when traversing the filesystem
        pub io_errors: u64,
    }

    impl TerminalApp {
        pub fn process_events<B, R>(
            &mut self,
            _terminal: &mut Terminal<B>,
            _keys: Keys<R>,
        ) -> Result<WalkResult, Error>
        where
            B: Backend,
            R: io::Read + TermReadEventsAndRaw,
        {
            unimplemented!()
        }

        pub fn initialize<B>(
            _terminal: &mut Terminal<B>,
            options: WalkOptions,
            input: Vec<PathBuf>,
        ) -> Result<TerminalApp, Error>
        where
            B: Backend,
        {
            let mut tree = Tree::new();
            let mut io_errors = 0u64;
            let mut entries_traversed = 0u64;
            let root_index = tree.add_node(EntryData::default());
            for path in input.into_iter() {
                let path_idx = tree.add_node(EntryData {
                    name: path.file_name().unwrap_or_default().into(),
                    ..Default::default()
                });
                tree.add_edge(root_index, path_idx, ());
                for entry in options.iter_from_path(path.as_ref()) {
                    entries_traversed += 1;
                    let mut data = EntryData::default();
                    match entry {
                        Ok(entry) => {
                            dbg!((&entry.file_name, entry.depth));
                            data.name = entry.file_name;
                            let file_size = match entry.metadata {
                                Some(Ok(ref m)) if !m.is_dir() => m.len(),
                                Some(Ok(_)) => 0,
                                Some(Err(_)) => {
                                    io_errors += 1;
                                    data.metadata_io_error = true;
                                    0
                                }
                                None => unreachable!(
                                    "we ask for metadata, so we at least have Some(Err(..))). Issue in jwalk?"
                                ),
                            };
                            data.size = file_size;
                            let entry_node = tree.add_node(data);
                            tree.add_edge(path_idx, entry_node, ());
                        }
                        Err(_) => io_errors += 1,
                    }
                }
            }

            Ok(TerminalApp {
                tree,
                root_index,
                entries_traversed,
                io_errors,
            })
        }
    }
}

pub use self::app::*;
