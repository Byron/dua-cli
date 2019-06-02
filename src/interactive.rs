mod app {
    use crate::{WalkOptions, WalkResult};
    use failure::Error;
    use petgraph::{prelude::NodeIndex, Directed, Direction, Graph};
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
            let (mut previous_node_idx, mut parent_node_idx) = (root_index, root_index);
            let mut sizes_per_depth_level = Vec::new();
            let mut current_size_at_depth = 0;
            let mut previous_depth = 0;

            for path in input.into_iter() {
                for entry in options.iter_from_path(path.as_ref()) {
                    entries_traversed += 1;
                    let mut data = EntryData::default();
                    match entry {
                        Ok(entry) => {
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

                            match (entry.depth, previous_depth) {
                                (n, p) if n > p => {
                                    sizes_per_depth_level.push(current_size_at_depth);
                                    current_size_at_depth = file_size;
                                    parent_node_idx = previous_node_idx;
                                }
                                (n, p) if n < p => {
                                    for _ in n..p {
                                        let size_at_level_above = sizes_per_depth_level
                                            .pop()
                                            .expect("sizes per level to be in sync with graph");
                                        tree.node_weight_mut(parent_node_idx)
                                            .expect("node for parent index we just retrieved")
                                            .size = current_size_at_depth;
                                        current_size_at_depth += size_at_level_above;
                                        parent_node_idx = tree
                                            .neighbors_directed(
                                                parent_node_idx,
                                                Direction::Incoming,
                                            )
                                            .next()
                                            .expect("every node in the iteration has a parent");
                                    }
                                    current_size_at_depth += file_size;
                                    tree.node_weight_mut(parent_node_idx)
                                        .expect("node for parent index we just retrieved")
                                        .size = current_size_at_depth;
                                }
                                _ => {
                                    current_size_at_depth += file_size;
                                }
                            };

                            previous_depth = entry.depth;
                            data.size = file_size;
                            let entry_index = tree.add_node(data);
                            tree.add_edge(parent_node_idx, entry_index, ());
                            previous_node_idx = entry_index;
                        }
                        Err(_) => io_errors += 1,
                    }
                }
            }

            dbg!(previous_depth);
            dbg!(&sizes_per_depth_level);
            dbg!(current_size_at_depth);
            for _ in 0..previous_depth {
                let size_at_level_above = sizes_per_depth_level
                    .pop()
                    .expect("sizes per level to be in sync with graph");
                parent_node_idx = tree
                    .neighbors_directed(parent_node_idx, Direction::Incoming)
                    .next()
                    .expect("every node in the iteration has a parent test");
                tree.node_weight_mut(parent_node_idx)
                    .expect("node for parent index we just retrieved")
                    .size = current_size_at_depth;
                current_size_at_depth += size_at_level_above;
            }
            // TODO finish size computation - there may still be unresolved sizes on the stack

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
