use crate::traverse::{EntryData, Tree, TreeIndex};
use byte_unit::{n_gb_bytes, n_gib_bytes, n_mb_bytes, n_mib_bytes, ByteUnit};
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub fn get_entry_or_panic(tree: &Tree, node_idx: TreeIndex) -> &EntryData {
    tree.node_weight(node_idx)
        .expect("node should always be retrievable with valid index")
}

pub(crate) fn get_size_or_panic(tree: &Tree, node_idx: TreeIndex) -> u64 {
    get_entry_or_panic(tree, node_idx).size
}

/// Specifies a way to format bytes
#[derive(Clone, Copy)]
pub enum ByteFormat {
    /// metric format, based on 1000.
    Metric,
    /// binary format, based on 1024
    Binary,
    /// raw bytes, without additional formatting
    Bytes,
    /// only gigabytes without smart-unit
    GB,
    /// only gibibytes without smart-unit
    GiB,
    /// only megabytes without smart-unit
    MB,
    /// only mebibytes without smart-unit
    MiB,
}

impl ByteFormat {
    pub fn width(self) -> usize {
        use ByteFormat::*;
        match self {
            Metric | Binary => 10,
            Bytes => 12,
            MiB | MB => 12,
            _ => 10,
        }
    }
    pub fn total_width(self) -> usize {
        use ByteFormat::*;
        const THE_SPACE_BETWEEN_UNIT_AND_NUMBER: usize = 1;

        self.width()
            + match self {
                Binary | MiB | GiB => 3,
                Metric | MB | GB => 2,
                Bytes => 1,
            }
            + THE_SPACE_BETWEEN_UNIT_AND_NUMBER
    }
    pub fn display(self, bytes: u64) -> ByteFormatDisplay {
        ByteFormatDisplay {
            format: self,
            bytes,
        }
    }
}

pub struct ByteFormatDisplay {
    format: ByteFormat,
    bytes: u64,
}

impl fmt::Display for ByteFormatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use byte_unit::Byte;
        use ByteFormat::*;

        let format = match self.format {
            Bytes => return write!(f, "{} b", self.bytes),
            Binary => (true, None),
            Metric => (false, None),
            GB => (false, Some((n_gb_bytes!(1), ByteUnit::GB))),
            GiB => (false, Some((n_gib_bytes!(1), ByteUnit::GiB))),
            MB => (false, Some((n_mb_bytes!(1), ByteUnit::MB))),
            MiB => (false, Some((n_mib_bytes!(1), ByteUnit::MiB))),
        };

        let b = match format {
            (_, Some((divisor, unit))) => Byte::from_unit(self.bytes as f64 / divisor as f64, unit)
                .expect("byte count > 0")
                .get_adjusted_unit(unit),
            (binary, None) => Byte::from_bytes(self.bytes as u128).get_appropriate_unit(binary),
        }
        .format(2);
        let mut splits = b.split(' ');
        match (splits.next(), splits.next()) {
            (Some(bytes), Some(unit)) => write!(
                f,
                "{} {:>unit_width$}",
                bytes,
                unit,
                unit_width = match self.format {
                    Binary => 3,
                    Metric => 2,
                    _ => 2,
                }
            ),
            _ => f.write_str(&b),
        }
    }
}

/// Identify the kind of sorting to apply during filesystem iteration
#[derive(Clone)]
pub enum TraversalSorting {
    None,
    AlphabeticalByFileName,
}

/// Throttle access to an optional `io::Write` to the specified `Duration`
#[derive(Debug)]
pub struct Throttle {
    trigger: Arc<AtomicBool>,
}

impl Throttle {
    pub fn new(duration: Duration, initial_sleep: Option<Duration>) -> Self {
        let instance = Self {
            trigger: Default::default(),
        };

        let trigger = Arc::downgrade(&instance.trigger);
        std::thread::spawn(move || {
            if let Some(duration) = initial_sleep {
                std::thread::sleep(duration)
            }
            while let Some(t) = trigger.upgrade() {
                t.store(true, Ordering::Relaxed);
                std::thread::sleep(duration);
            }
        });

        instance
    }

    pub fn throttled<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.can_update() {
            f()
        }
    }

    /// Return `true` if we are not currently throttled.
    pub fn can_update(&self) -> bool {
        self.trigger.swap(false, Ordering::Relaxed)
    }
}

/// Configures a filesystem walk, including output and formatting options.
#[derive(Clone)]
pub struct WalkOptions {
    /// The amount of threads to use. Refer to [`WalkDir::num_threads()`](https://docs.rs/jwalk/0.4.0/jwalk/struct.WalkDir.html#method.num_threads)
    /// for more information.
    pub threads: usize,
    pub byte_format: ByteFormat,
    pub count_hard_links: bool,
    pub apparent_size: bool,
    pub sorting: TraversalSorting,
    pub cross_filesystems: bool,
    pub ignore_dirs: Vec<PathBuf>,
}

pub enum FlowControl {
    Continue,
    Abort,
}

#[cfg(unix)]
pub fn file_size_on_disk(meta: &::moonwalk::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.blocks() * 512
}

#[cfg(windows)]
pub fn file_size_on_disk(meta: &::moonwalk::Metadata) -> u64 {
    // TODO: use `windows` crate, remove `filesize` crate
    meta.len()
}

#[cfg(all(not(unix), not(windows)))]
pub fn file_size_on_disk(meta: &::moonwalk::Metadata) -> u64 {
    meta.len()
}

mod moonwalk {
    use crate::{crossdev, FlowControl, WalkOptions};
    use moonwalk::{DirEntry, WalkState};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    impl WalkOptions {
        /// Returns the amount of IO errors we encountered.
        pub fn moonwalk_from_path(
            &self,
            root: &Path,
            root_device_id: u64,
            update: impl FnMut(std::io::Result<&mut DirEntry<'_>>) -> FlowControl + Send + Clone,
        ) -> std::io::Result<()> {
            let delegate = Delegate {
                cb: update,
                root_device_id,
                storage: Default::default(),
                opts: self.clone(),
            };

            let mut builder = moonwalk::WalkBuilder::new();
            builder.follow_links(false);
            builder.run_parallel(root, self.threads, delegate, root.into())?;

            Ok(())
        }
    }

    #[derive(Clone)]
    struct Delegate<CB> {
        cb: CB,
        root_device_id: u64,
        storage: PathBuf,
        opts: WalkOptions,
    }

    impl<CB> moonwalk::VisitorParallel for Delegate<CB>
    where
        CB: for<'a> FnMut(std::io::Result<&'a mut DirEntry>) -> FlowControl + Send + Clone,
    {
        type State = OsString;

        fn visit<'a>(
            &mut self,
            parents: impl Iterator<Item = &'a Self::State>,
            dent: std::io::Result<&mut DirEntry<'_>>,
        ) -> WalkState<Self::State> {
            match dent {
                Ok(dent) => {
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
                                    p.extend(parents);
                                    p.push(dent.file_name());
                                    self.opts.ignore_dirs.contains(p)
                                };

                                if is_ignored {
                                    return WalkState::Skip;
                                }
                            }
                        }
                        WalkState::Continue(dent.file_name().to_owned())
                    } else {
                        match (self.cb)(Ok(dent)) {
                            FlowControl::Abort => WalkState::Quit,
                            FlowControl::Continue => WalkState::Skip,
                        }
                    }
                }
                Err(err) => match (self.cb)(Err(err)) {
                    FlowControl::Abort => WalkState::Quit,
                    FlowControl::Continue => WalkState::Skip,
                },
            }
        }

        fn pop_dir<'a>(
            &mut self,
            _state: Self::State,
            _parents: impl Iterator<Item = &'a Self::State>,
        ) {
        }
    }
}

/// Information we gather during a filesystem walk
#[derive(Default)]
pub struct WalkResult {
    /// The amount of io::errors we encountered. Can happen when fetching meta-data, or when reading the directory contents.
    pub num_errors: u64,
}

impl WalkResult {
    pub fn to_exit_code(&self) -> i32 {
        i32::from(self.num_errors > 0)
    }
}
