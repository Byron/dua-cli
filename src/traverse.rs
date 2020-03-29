use crate::{get_size_or_panic, InodeFilter, WalkOptions};
use failure::Error;
use filesize::PathExt;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed, Direction};
use std::{path::PathBuf, time::Duration, time::Instant};

pub type TreeIndex = NodeIndex;
pub type Tree = StableGraph<EntryData, (), Directed>;

#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub struct EntryData {
    pub name: PathBuf,
    /// The entry's size in bytes. If it's a directory, the size is the aggregated file size of all children
    pub size: u64,
    /// If set, the item meta-data could not be obtained
    pub metadata_io_error: bool,
}

const REFRESH_RATE: Duration = Duration::from_millis(100);

/// The result of the previous filesystem traversal
#[derive(Default, Debug)]
pub struct Traversal {
    /// A tree representing the entire filestem traversal
    pub tree: Tree,
    /// The top-level node of the tree.
    pub root_index: TreeIndex,
    /// Amount of files or directories we have seen during the filesystem traversal
    pub entries_traversed: u64,
    /// Total amount of IO errors encountered when traversing the filesystem
    pub io_errors: u64,
    /// Total amount of bytes seen during the traversal
    pub total_bytes: Option<u64>,
}

impl Traversal {
    pub fn from_walk(
        mut walk_options: WalkOptions,
        input: Vec<PathBuf>,
        mut update: impl FnMut(&mut Traversal) -> Result<(), Error>,
    ) -> Result<Traversal, Error> {
        fn set_size_or_panic(tree: &mut Tree, node_idx: TreeIndex, current_size_at_depth: u64) {
            tree.node_weight_mut(node_idx)
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

        let mut t = {
            let mut tree = Tree::new();
            let root_index = tree.add_node(EntryData::default());
            Traversal {
                tree,
                root_index,
                ..Default::default()
            }
        };

        let (mut previous_node_idx, mut parent_node_idx) = (t.root_index, t.root_index);
        let mut sizes_per_depth_level = Vec::new();
        let mut current_size_at_depth = 0;
        let mut previous_depth = 0;
        let mut inodes = InodeFilter::default();

        let mut last_checked = Instant::now();

        const INITIAL_CHECK_INTERVAL: usize = 500;
        let mut check_instant_every = INITIAL_CHECK_INTERVAL;
        if walk_options.threads == 0 {
            // avoid using the global rayon pool, as it will keep a lot of threads alive after we are done.
            // Also means that we will spin up a bunch of threads per root path, instead of reusing them.
            walk_options.threads = num_cpus::get();
        }
        for path in input.into_iter() {
            let mut last_seen_eid = 0;
            for (eid, entry) in walk_options
                .iter_from_path(path.as_ref())
                .into_iter()
                .enumerate()
            {
                t.entries_traversed += 1;
                let mut data = EntryData::default();
                match entry {
                    Ok(entry) => {
                        data.name = if entry.depth < 1 {
                            path.clone().into()
                        } else {
                            entry.file_name.into()
                        };
                        let file_size = match entry.client_state {
                            Some(Ok(ref m))
                                if !m.is_dir()
                                    && (walk_options.count_hard_links || inodes.add(m)) =>
                            {
                                if walk_options.apparent_size {
                                    m.len()
                                } else {
                                    data.name.size_on_disk_fast(m).unwrap_or_else(|_| {
                                        t.io_errors += 1;
                                        data.metadata_io_error = true;
                                        0
                                    })
                                }
                            }
                            Some(Ok(_)) => 0,
                            Some(Err(_)) => {
                                t.io_errors += 1;
                                data.metadata_io_error = true;
                                0
                            }
                            None => unreachable!("must have populated client state for metadata"),
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
                                        &mut t.tree,
                                        parent_node_idx,
                                        current_size_at_depth,
                                    );
                                    current_size_at_depth +=
                                        pop_or_panic(&mut sizes_per_depth_level);
                                    parent_node_idx = parent_or_panic(&mut t.tree, parent_node_idx);
                                }
                                current_size_at_depth += file_size;
                                set_size_or_panic(
                                    &mut t.tree,
                                    parent_node_idx,
                                    current_size_at_depth,
                                );
                            }
                            _ => {
                                current_size_at_depth += file_size;
                            }
                        };

                        data.size = file_size;
                        let entry_index = t.tree.add_node(data);

                        t.tree.add_edge(parent_node_idx, entry_index, ());
                        previous_node_idx = entry_index;
                        previous_depth = entry.depth;
                    }
                    Err(_) => {
                        if previous_depth == 0 {
                            data.name = path.clone().into();
                            let entry_index = t.tree.add_node(data);
                            t.tree.add_edge(parent_node_idx, entry_index, ());
                        }

                        t.io_errors += 1
                    }
                }

                if eid != 0
                    && eid % check_instant_every == 0
                    && last_checked.elapsed() >= REFRESH_RATE
                {
                    let now = Instant::now();
                    let elapsed = (now - last_checked).as_millis() as f64;
                    check_instant_every = (INITIAL_CHECK_INTERVAL as f64
                        * ((eid - last_seen_eid) as f64 / INITIAL_CHECK_INTERVAL as f64)
                        * (REFRESH_RATE.as_millis() as f64 / elapsed))
                        .max(1.0) as usize;
                    last_seen_eid = eid;
                    last_checked = now;

                    update(&mut t)?;
                }
            }
        }

        sizes_per_depth_level.push(current_size_at_depth);
        current_size_at_depth = 0;
        for _ in 0..previous_depth {
            current_size_at_depth += pop_or_panic(&mut sizes_per_depth_level);
            set_size_or_panic(&mut t.tree, parent_node_idx, current_size_at_depth);
            parent_node_idx = parent_or_panic(&mut t.tree, parent_node_idx);
        }
        let root_size = t.recompute_root_size();
        set_size_or_panic(&mut t.tree, t.root_index, root_size);
        t.total_bytes = Some(root_size);

        Ok(t)
    }

    fn recompute_root_size(&self) -> u64 {
        self.tree
            .neighbors_directed(self.root_index, Direction::Outgoing)
            .map(|idx| get_size_or_panic(&self.tree, idx))
            .sum()
    }
}
