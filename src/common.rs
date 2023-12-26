use crate::crossdev;
use crate::traverse::{EntryData, Tree, TreeIndex};
use byte_unit::{n_gb_bytes, n_gib_bytes, n_mb_bytes, n_mib_bytes, ByteUnit};
use log::info;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{fmt, path::Path};

pub fn get_entry_or_panic(tree: &Tree, node_idx: TreeIndex) -> &EntryData {
    tree.node_weight(node_idx)
        .expect("node should always be retrievable with valid index")
}

pub(crate) fn get_size_or_panic(tree: &Tree, node_idx: TreeIndex) -> u128 {
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
            Metric => 10,
            Binary => 11,
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
    pub fn display(self, bytes: u128) -> ByteFormatDisplay {
        ByteFormatDisplay {
            format: self,
            bytes,
        }
    }
}

pub struct ByteFormatDisplay {
    format: ByteFormat,
    bytes: u128,
}

impl fmt::Display for ByteFormatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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
            (binary, None) => Byte::from_bytes(self.bytes).get_appropriate_unit(binary),
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

type WalkDir = jwalk::WalkDirGeneric<((), Option<Result<std::fs::Metadata, jwalk::Error>>)>;

impl WalkOptions {
    pub(crate) fn iter_from_path(&self, root: &Path, root_device_id: u64) -> WalkDir {
        info!("root path={:?}", root);
        WalkDir::new(root)
            .follow_links(false)
            .sort(match self.sorting {
                TraversalSorting::None => false,
                TraversalSorting::AlphabeticalByFileName => true,
            })
            .skip_hidden(false)
            .process_read_dir({
                let ignore_dirs = self.ignore_dirs.clone();
                let cross_filesystems = self.cross_filesystems;
                move |_, _, _, dir_entry_results| {
                    dir_entry_results.iter_mut().for_each(|dir_entry_result| {
                        if let Ok(dir_entry) = dir_entry_result {
                            let metadata = dir_entry.metadata();

                            if dir_entry.file_type.is_dir() {
                                let ok_for_fs = cross_filesystems
                                    || metadata
                                        .as_ref()
                                        .map(|m| crossdev::is_same_device(root_device_id, m))
                                        .unwrap_or(true);
                                if !ok_for_fs || ignore_dirs.contains(&dir_entry.path()) {
                                    dir_entry.read_children_path = None;
                                }
                            }

                            dir_entry.client_state = Some(metadata);
                        }
                    })
                }
            })
            .parallelism(match self.threads {
                0 => jwalk::Parallelism::RayonDefaultPool {
                    busy_timeout: std::time::Duration::from_secs(1),
                },
                1 => jwalk::Parallelism::Serial,
                _ => jwalk::Parallelism::RayonExistingPool {
                    pool: jwalk::rayon::ThreadPoolBuilder::new()
                        .stack_size(128 * 1024)
                        .num_threads(self.threads)
                        .thread_name(|idx| format!("dua-fs-walk-{idx}"))
                        .build()
                        .expect("fields we set cannot fail")
                        .into(),
                    busy_timeout: None,
                },
            })
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
