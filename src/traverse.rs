use crate::{crossdev, file_size_on_disk, get_size_or_panic, InodeFilter, Throttle, WalkOptions};
use ::moonwalk::{DirEntry, WalkState};
use parking_lot::Mutex;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed, Direction};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{io, path::PathBuf, time::Duration};

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

/// The result of the previous filesystem traversal
#[derive(Debug)]
pub struct Traversal {
    /// A tree representing the entire filestem traversal
    pub tree: Arc<Mutex<Tree>>,
    /// The top-level node of the tree.
    pub root_index: TreeIndex,
    /// Amount of files or directories we have seen during the filesystem traversal
    pub entries_traversed: u64,
    /// The time at which the traversal started.
    pub start: std::time::Instant,
    /// The amount of time it took to finish the traversal. Set only once done.
    pub elapsed: Option<std::time::Duration>,
    /// Total amount of IO errors encountered when traversing the filesystem
    pub io_errors: u64,
    /// Total amount of bytes seen during the traversal, available when traversal is done.
    pub total_bytes: Option<u64>,
}

impl Traversal {
    pub fn is_done(&self) -> bool {
        self.elapsed.is_some()
    }
    fn recompute_size_by_aggregating_children(&self, index: TreeIndex) -> u64 {
        let guard = self.tree.lock();
        guard
            .neighbors_directed(index, Direction::Outgoing)
            .map(|idx| get_size_or_panic(&guard, idx))
            .sum()
    }

    pub fn from_moonwalk(
        mut walk_options: WalkOptions,
        input: Vec<PathBuf>,
        mut update: impl FnMut(&mut Traversal) -> anyhow::Result<bool>,
    ) -> anyhow::Result<Option<Traversal>> {
        fn set_size_or_panic(tree: &mut Tree, node_idx: TreeIndex, current_size_at_depth: u64) {
            tree.node_weight_mut(node_idx)
                .expect("node for parent index we just retrieved")
                .size = current_size_at_depth;
        }
        let mut t = {
            let tree = Arc::new(Mutex::new(Tree::new()));
            let root_index = tree.lock().add_node(EntryData::default());
            Traversal {
                tree,
                root_index,
                entries_traversed: 0,
                start: std::time::Instant::now(),
                elapsed: None,
                io_errors: 0,
                total_bytes: None,
            }
        };

        #[derive(Clone)]
        struct Delegate {
            tree: Arc<Mutex<Tree>>,
            io_errors: Arc<AtomicU64>,
            results: std::sync::mpsc::Sender<()>,
            count_hard_links: bool,
            apparent_size: bool,
            inodes: Arc<InodeFilter>,
        }

        fn compute_file_size(
            m: &::moonwalk::Metadata,
            count_hard_links: bool,
            apparent_size: bool,
            inodes: &InodeFilter,
        ) -> u64 {
            if !m.is_dir() && (count_hard_links || inodes.is_first_moonwalk(m)) {
                if apparent_size {
                    m.len()
                } else {
                    file_size_on_disk(m)
                }
            } else {
                0
            }
        }

        impl ::moonwalk::VisitorParallel for Delegate {
            type State = (TreeIndex, AtomicU64);

            fn visit<'a>(
                &mut self,
                mut parents: impl Iterator<Item = &'a Self::State> + Clone,
                dent: io::Result<&mut DirEntry<'_>>,
            ) -> WalkState<Self::State> {
                match dent {
                    Ok(dent) => {
                        let (parent_idx, parent_size) =
                            parents.next().expect("always the root node");
                        let mut data = EntryData::default();
                        data.name = dent.file_name().into();

                        let file_size = match dent.metadata() {
                            Ok(m) => compute_file_size(
                                m,
                                self.count_hard_links,
                                self.apparent_size,
                                &self.inodes,
                            ),
                            Err(_) => {
                                self.io_errors.fetch_add(1, Ordering::SeqCst);
                                data.metadata_io_error = true;
                                0
                            }
                        };

                        if file_size != 0 {
                            data.size = file_size;
                            parent_size.fetch_add(file_size, Ordering::SeqCst);
                            for (_, parent_size) in parents {
                                parent_size.fetch_add(file_size, Ordering::SeqCst);
                            }
                        }

                        let node_idx = {
                            let tree = &mut self.tree.lock();
                            let node_idx = tree.add_node(data);
                            tree.add_edge(*parent_idx, node_idx, ());
                            node_idx
                        };

                        if self.results.send(()).is_err() {
                            WalkState::Quit
                        } else {
                            if dent.file_type().is_dir() {
                                WalkState::Continue((node_idx, Default::default()))
                            } else {
                                WalkState::Skip
                            }
                        }
                    }
                    Err(_err) => {
                        self.io_errors.fetch_add(1, Ordering::SeqCst);
                        WalkState::Skip
                    }
                }
            }

            fn pop_dir<'a>(
                &mut self,
                state: Self::State,
                _parents: impl Iterator<Item = &'a Self::State> + Clone,
            ) {
                let (node_idx, size) = state;
                set_size_or_panic(&mut self.tree.lock(), node_idx, size.load(Ordering::SeqCst));
            }
        }

        let inodes = Arc::new(InodeFilter::default());
        let io_errors = Arc::new(AtomicU64::default());
        let throttle = Throttle::new(Duration::from_millis(50), None);

        if walk_options.threads == 0 {
            // avoid using the global rayon pool, as it will keep a lot of threads alive after we are done.
            // Also means that we will spin up a bunch of threads per root path, instead of reusing them.
            walk_options.threads = num_cpus::get();
        }

        for path in input.into_iter() {
            let (device_id, meta) = match crossdev::init(path.as_ref()) {
                Ok(id) => id,
                Err(_) => {
                    t.io_errors += 1;
                    continue;
                }
            };

            let (rx, traversal_root_idx) = {
                let (tx, rx) = std::sync::mpsc::channel();
                if !meta.is_dir() {
                    // moonwalk will fail to traverse non-dirs, so we have to fill in what it would do.
                    tx.send(()).ok();
                }
                let traversal_root_idx = {
                    let tree = &mut t.tree.lock();
                    let traversal_root_idx = tree.add_node(EntryData {
                        name: path.clone(),
                        size: compute_file_size(
                            &meta.into(),
                            walk_options.count_hard_links,
                            walk_options.apparent_size,
                            &inodes,
                        ),
                        ..Default::default()
                    });
                    tree.add_edge(t.root_index, traversal_root_idx, ());
                    traversal_root_idx
                };

                std::thread::spawn({
                    let walk_options = walk_options.clone();
                    let tx = tx.clone();
                    let path = path.clone();
                    let tree = t.tree.clone();
                    let io_errors = io_errors.clone();
                    let inodes = inodes.clone();
                    move || {
                        walk_options.moonwalk_from_path_2(
                            path.as_ref(),
                            device_id,
                            Delegate {
                                tree,
                                io_errors,
                                results: tx,
                                apparent_size: walk_options.apparent_size,
                                count_hard_links: walk_options.count_hard_links,
                                inodes: inodes.clone(),
                            },
                            (traversal_root_idx, 0.into()),
                        )
                    }
                });
                (rx, traversal_root_idx)
            };

            for () in rx {
                t.entries_traversed += 1;
                if throttle.can_update() && update(&mut t)? {
                    return Ok(None);
                }
            }

            let root_size = t.recompute_size_by_aggregating_children(traversal_root_idx);
            if root_size != 0 {
                set_size_or_panic(&mut t.tree.lock(), traversal_root_idx, root_size);
            }
        }

        t.io_errors = io_errors.load(Ordering::Relaxed);

        let root_size = t.recompute_size_by_aggregating_children(t.root_index);
        set_size_or_panic(&mut t.tree.lock(), t.root_index, root_size);
        t.total_bytes = Some(root_size);

        t.elapsed = t.start.elapsed().into();
        Ok(Some(t))
    }
}

// TODO: put this into common, and make it possible to pass a closure then once there is closure support to
//       use aggregate code with that.
mod moonwalk {
    use crate::{crossdev, WalkOptions};
    use moonwalk::{DirEntry, WalkState};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    impl WalkOptions {
        pub fn moonwalk_from_path_2<D>(
            &self,
            root: &Path,
            root_device_id: u64,
            inner: D,
            initial_state: D::State,
        ) -> std::io::Result<()>
        where
            D: moonwalk::VisitorParallel,
        {
            let delegate = Delegate {
                inner,
                root_device_id,
                storage: Default::default(),
                opts: self.clone(),
            };

            let mut builder = moonwalk::WalkBuilder::new();
            builder.follow_links(false);
            builder.run_parallel(root, self.threads, delegate, (root.into(), initial_state))?;

            Ok(())
        }
    }

    #[derive(Clone)]
    struct Delegate<D> {
        inner: D,
        root_device_id: u64,
        storage: PathBuf,
        opts: WalkOptions,
    }

    impl<D> moonwalk::VisitorParallel for Delegate<D>
    where
        D: moonwalk::VisitorParallel,
    {
        type State = (OsString, D::State);

        fn visit<'a>(
            &mut self,
            parents: impl Iterator<Item = &'a Self::State> + Clone,
            mut dent: std::io::Result<&mut DirEntry<'_>>,
        ) -> WalkState<Self::State> {
            let dir_name = if let Ok(dent) = dent.as_mut() {
                let is_dir = dent.file_type().is_dir();
                if is_dir {
                    if let Ok(meta) = dent.metadata() {
                        let ok_for_fs = self.opts.cross_filesystems
                            || crossdev::is_same_device_moonwalk(self.root_device_id, meta);
                        if !ok_for_fs {
                            return WalkState::Skip;
                        } else if !self.opts.ignore_dirs.is_empty() {
                            let is_ignored = {
                                let p = &mut self.storage;
                                p.clear();
                                p.extend(parents.clone().map(|t| &t.0));
                                p.push(dent.file_name());
                                self.opts.ignore_dirs.contains(&p)
                            };

                            if is_ignored {
                                return WalkState::Skip;
                            }
                        }
                    }
                    Some(dent.file_name().to_owned())
                } else {
                    None
                }
            } else {
                None
            };
            let next = self.inner.visit(parents.map(|t| &t.1), dent);
            match next {
                WalkState::Skip => WalkState::Skip,
                WalkState::Quit => WalkState::Quit,
                WalkState::Continue(inner) => {
                    WalkState::Continue((dir_name.unwrap_or_default(), inner))
                }
            }
        }

        fn pop_dir<'a>(
            &mut self,
            state: Self::State,
            parents: impl Iterator<Item = &'a Self::State> + Clone,
        ) {
            self.inner.pop_dir(state.1, parents.map(|t| &t.1))
        }
    }
}
