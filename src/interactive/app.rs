use crate::{WalkOptions, WalkResult};
use failure::Error;
use petgraph::{prelude::NodeIndex, Directed, Direction, Graph};
use std::time::{Duration, Instant};
use std::{ffi::OsString, io, path::PathBuf};
use termion::input::{Keys, TermReadEventsAndRaw};
use tui::widgets::Widget;
use tui::{backend::Backend, Terminal};

pub type TreeIndexType = u32;
pub type TreeIndex = NodeIndex<TreeIndexType>;
pub type Tree = Graph<EntryData, (), Directed, TreeIndexType>;

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
    pub root_index: TreeIndex,
    /// Amount of files or directories we have seen during the filesystem traversal
    pub entries_traversed: u64,
    /// Total amount of IO errors encountered when traversing the filesystem
    pub io_errors: u64,
}

const GUI_REFRESH_RATE: Duration = Duration::from_millis(100);

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
        terminal: &mut Terminal<B>,
        options: WalkOptions,
        input: Vec<PathBuf>,
    ) -> Result<TerminalApp, Error>
    where
        B: Backend,
    {
        fn set_size_or_panic(
            tree: &mut Tree,
            parent_node_idx: TreeIndex,
            current_size_at_depth: u64,
        ) {
            tree.node_weight_mut(parent_node_idx)
                .expect("node for parent index we just retrieved")
                .size = current_size_at_depth;
        }
        fn parent_or_panic(tree: &mut Tree, parent_node_idx: TreeIndex) -> TreeIndex {
            tree.neighbors_directed(parent_node_idx, Direction::Incoming)
                .next()
                .expect("every node in the iteration has a parent")
        }
        fn pop_or_panic(v: &mut Vec<u64>) -> u64 {
            v.pop().expect("sizes per level to be in sync with graph")
        }
        let mut tree = Tree::new();
        let mut io_errors = 0u64;
        let mut entries_traversed = 0u64;

        let root_index = tree.add_node(EntryData::default());
        let (mut previous_node_idx, mut parent_node_idx) = (root_index, root_index);
        let mut sizes_per_depth_level = Vec::new();
        let mut current_size_at_depth = 0;
        let mut previous_depth = 0;

        let mut last_checked = Instant::now();

        const INITIAL_CHECK_INTERVAL: usize = 500;
        let mut check_instant_every = INITIAL_CHECK_INTERVAL;
        let mut last_seen_eid;

        for path in input.into_iter() {
            last_seen_eid = 0;
            for (eid, entry) in options
                .iter_from_path(path.as_ref())
                .into_iter()
                .enumerate()
            {
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
                                    set_size_or_panic(
                                        &mut tree,
                                        parent_node_idx,
                                        current_size_at_depth,
                                    );
                                    current_size_at_depth +=
                                        pop_or_panic(&mut sizes_per_depth_level);
                                    parent_node_idx = parent_or_panic(&mut tree, parent_node_idx);
                                }
                                current_size_at_depth += file_size;
                                set_size_or_panic(
                                    &mut tree,
                                    parent_node_idx,
                                    current_size_at_depth,
                                );
                            }
                            _ => {
                                current_size_at_depth += file_size;
                            }
                        };

                        data.size = file_size;
                        let entry_index = tree.add_node(data);

                        tree.add_edge(parent_node_idx, entry_index, ());
                        previous_node_idx = entry_index;
                        previous_depth = entry.depth;
                    }
                    Err(_) => io_errors += 1,
                }

                if eid != 0
                    && eid % check_instant_every == 0
                    && last_checked.elapsed() >= GUI_REFRESH_RATE
                {
                    let now = Instant::now();
                    let elapsed = (now - last_checked).as_millis() as f64;
                    check_instant_every = (INITIAL_CHECK_INTERVAL as f64
                        * ((eid - last_seen_eid) as f64 / INITIAL_CHECK_INTERVAL as f64)
                        * (GUI_REFRESH_RATE.as_millis() as f64 / elapsed))
                        as usize;
                    last_seen_eid = eid;
                    last_checked = now;

                    terminal.draw(|mut f| {
                        let full_screen = f.size();
                        super::widgets::Entries {
                            tree: &tree,
                            root: root_index,
                        }
                        .render(&mut f, full_screen)
                    })?;
                }
            }
        }

        sizes_per_depth_level.push(current_size_at_depth);
        current_size_at_depth = 0;
        for _ in 0..previous_depth {
            current_size_at_depth += pop_or_panic(&mut sizes_per_depth_level);
            set_size_or_panic(&mut tree, parent_node_idx, current_size_at_depth);
            parent_node_idx = parent_or_panic(&mut tree, parent_node_idx);
        }
        set_size_or_panic(&mut tree, root_index, current_size_at_depth);

        Ok(TerminalApp {
            tree,
            root_index,
            entries_traversed,
            io_errors,
        })
    }
}
