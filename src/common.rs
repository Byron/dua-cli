use crate::{crossdev, walk};
use byte_unit::{Byte, Unit, UnitType};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{fmt, path::Path};

/// Specifies a way to format bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum ByteFormat {
    /// metric format, based on 1000.
    #[serde(rename = "metric")]
    Metric,
    /// binary format, based on 1024
    #[serde(rename = "binary")]
    Binary,
    /// raw bytes, without additional formatting
    #[serde(rename = "bytes")]
    Bytes,
    /// only gigabytes without smart-unit
    #[serde(rename = "gb")]
    GB,
    /// only gibibytes without smart-unit
    #[serde(rename = "gib")]
    GiB,
    /// only megabytes without smart-unit
    #[serde(rename = "mb")]
    MB,
    /// only mebibytes without smart-unit
    #[serde(rename = "mib")]
    MiB,
}

impl ByteFormat {
    /// Return the content width (without unit suffix) needed to display values in this format.
    #[must_use]
    pub fn width(self) -> usize {
        use ByteFormat::{Binary, Bytes, MB, MiB};
        match self {
            Binary => 11,
            Bytes | MB | MiB => 12,
            _ => 10,
        }
    }
    /// Return the full width (value plus unit and separator) used by this format.
    #[must_use]
    pub fn total_width(self) -> usize {
        use ByteFormat::{Binary, Bytes, GB, GiB, MB, Metric, MiB};
        const THE_SPACE_BETWEEN_UNIT_AND_NUMBER: usize = 1;

        self.width()
            + match self {
                Binary | MiB | GiB => 3,
                Metric | MB | GB => 2,
                Bytes => 1,
            }
            + THE_SPACE_BETWEEN_UNIT_AND_NUMBER
    }
    /// Create a display adapter for `bytes` using this format.
    #[must_use]
    pub fn display(self, bytes: u128) -> impl fmt::Display {
        ByteFormatDisplay {
            format: self,
            bytes,
        }
    }
}

/// A lightweight display adapter created by [`ByteFormat::display`].
struct ByteFormatDisplay {
    format: ByteFormat,
    bytes: u128,
}

impl fmt::Display for ByteFormatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use ByteFormat::{Binary, Bytes, GB, GiB, MB, Metric, MiB};

        let bytes = Byte::from_u128(self.bytes).expect("supported byte count");
        let adjusted = match self.format {
            Bytes => return write!(f, "{} b", self.bytes),
            Binary => bytes.get_appropriate_unit(UnitType::Binary),
            Metric => bytes.get_appropriate_unit(UnitType::Decimal),
            GB => bytes.get_adjusted_unit(Unit::GB),
            GiB => bytes.get_adjusted_unit(Unit::GiB),
            MB => bytes.get_adjusted_unit(Unit::MB),
            MiB => bytes.get_adjusted_unit(Unit::MiB),
        };
        let b = format!("{adjusted:.2}");
        let mut splits = b.split(' ');
        match (splits.next(), splits.next()) {
            (Some(bytes), Some(unit)) => write!(
                f,
                "{} {:>unit_width$}",
                bytes,
                unit,
                unit_width = match self.format {
                    Binary => 3,
                    _ => 2,
                }
            ),
            _ => f.write_str(&b),
        }
    }
}

/// Throttle access to an optional `io::Write` to the specified `Duration`
#[derive(Debug)]
pub(crate) struct Throttle {
    trigger: Arc<AtomicBool>,
}

impl Throttle {
    /// Create a new throttle that allows updates at most once per `duration`.
    ///
    /// If `initial_sleep` is set, the first update is delayed by that amount.
    pub(crate) fn new(duration: Duration, initial_sleep: Option<Duration>) -> Self {
        let instance = Self {
            trigger: Arc::default(),
        };

        let trigger = Arc::downgrade(&instance.trigger);
        std::thread::spawn(move || {
            if let Some(duration) = initial_sleep {
                std::thread::sleep(duration);
            }
            while let Some(t) = trigger.upgrade() {
                t.store(true, Ordering::Relaxed);
                std::thread::sleep(duration);
            }
        });

        instance
    }

    /// Execute `f` only if the throttle currently allows an update.
    pub(crate) fn throttled<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.can_update() {
            f();
        }
    }

    /// Return `true` if we are not currently throttled.
    pub(crate) fn can_update(&self) -> bool {
        self.trigger.swap(false, Ordering::Relaxed)
    }
}

/// Configures a filesystem walk, including output and formatting options.
#[derive(Clone)]
pub struct WalkOptions {
    /// The amount of filesystem worker threads to use.
    pub threads: usize,
    /// If `true`, count every hard-link occurrence independently.
    pub count_hard_links: bool,
    /// If `true`, use apparent size (`metadata.len()`), not allocated blocks on disk.
    pub apparent_size: bool,
    /// If `false`, traversal is constrained to the root filesystem/device.
    pub cross_filesystems: bool,
    /// Canonicalized directories to skip from traversal.
    pub ignore_dirs: BTreeSet<PathBuf>,
}

impl WalkOptions {
    /// Create an iterator over `root` honoring this walk configuration.
    ///
    /// `root_device_id` is used to filter entries when `cross_filesystems == false`.
    /// If `skip_root` is `true`, the root directory itself is omitted from yielded entries.
    pub(crate) fn iter_from_path(
        &self,
        root: &Path,
        root_device_id: u64,
        skip_root: bool,
        order: walk::Order,
    ) -> impl Iterator<Item = std::io::Result<walk::Entry>> {
        let ignore_dirs = self.ignore_dirs.clone();
        let cwd = std::env::current_dir().unwrap_or_else(|_| root.to_owned());
        let cross_filesystems = self.cross_filesystems;
        walk::walk(root, self.threads, order, move |entry| {
            (cross_filesystems
                || entry.metadata.as_ref().map_or(true, |metadata| {
                    crossdev::is_same_device(root_device_id, metadata)
                }))
                && (entry.depth == 0 || !ignore_directory(&entry.path(), &ignore_dirs, &cwd))
        })
        .filter(move |entry| !skip_root || entry.as_ref().map_or(true, |entry| entry.depth > 0))
    }
}

/// Information we gather during a filesystem walk
#[derive(Default)]
pub struct WalkResult {
    /// The amount of `io::errors` we encountered. Can happen when fetching meta-data, or when reading the directory contents.
    pub num_errors: u64,
}

impl WalkResult {
    /// Convert traversal result into a process exit code.
    ///
    /// Returns `0` if no I/O errors occurred, otherwise `1`.
    #[must_use]
    pub fn to_exit_code(&self) -> i32 {
        i32::from(self.num_errors > 0)
    }
}

/// Canonicalize user-provided ignore directory paths.
///
/// Non-canonicalizable paths are ignored.
pub fn canonicalize_ignore_dirs(ignore_dirs: &[PathBuf]) -> BTreeSet<PathBuf> {
    let dirs = ignore_dirs
        .iter()
        .map(gix::path::realpath)
        .filter_map(Result::ok)
        .collect();
    log::info!("Ignoring canonicalized {dirs:?}");
    dirs
}

fn ignore_directory(path: &Path, ignore_dirs: &BTreeSet<PathBuf>, cwd: &Path) -> bool {
    if ignore_dirs.is_empty() {
        return false;
    }
    let path = gix::path::realpath_opts(path, cwd, 32);
    path.is_ok_and(|path| {
        let ignored = ignore_dirs.contains(&path);
        if ignored {
            log::debug!("Ignored {}", path.display());
        }
        ignored
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_directories() {
        let cwd = std::env::current_dir().unwrap();
        #[cfg(unix)]
        let mut parameters = vec![
            ("/usr", vec!["/usr"], true),
            ("/usr/local", vec!["/usr"], false),
            ("/smth", vec!["/usr"], false),
            ("/usr/local/..", vec!["/usr/local/.."], true),
            ("/usr", vec!["/usr/local/.."], true),
            ("/usr/local/share/../..", vec!["/usr"], true),
        ];

        #[cfg(windows)]
        let mut parameters = vec![
            ("C:\\Windows", vec!["C:\\Windows"], true),
            ("C:\\Windows\\System", vec!["C:\\Windows"], false),
            ("C:\\Smth", vec!["C:\\Windows"], false),
            (
                "C:\\Windows\\System\\..",
                vec!["C:\\Windows\\System\\.."],
                true,
            ),
            ("C:\\Windows", vec!["C:\\Windows\\System\\.."], true),
            (
                "C:\\Windows\\System\\Speech\\..\\..",
                vec!["C:\\Windows"],
                true,
            ),
        ];

        parameters.extend([
            ("src", vec!["src"], true),
            ("src/interactive", vec!["src"], false),
            ("src/interactive/..", vec!["src"], true),
        ]);

        for (path, ignore_dirs, expected_result) in parameters {
            let ignore_dirs = canonicalize_ignore_dirs(
                &ignore_dirs.into_iter().map(Into::into).collect::<Vec<_>>(),
            );
            assert_eq!(
                ignore_directory(path.as_ref(), &ignore_dirs, &cwd),
                expected_result,
                "result='{expected_result}' for path='{path}' and ignore_dir='{ignore_dirs:?}' "
            );
        }
    }

    #[test]
    fn explicitly_selected_ignored_root_is_traversed() {
        let root = tempfile::tempdir().unwrap();
        let child = root.path().join("child");
        std::fs::create_dir(&child).unwrap();
        let options = WalkOptions {
            threads: 2,
            count_hard_links: false,
            apparent_size: false,
            cross_filesystems: true,
            ignore_dirs: canonicalize_ignore_dirs(&[root.path().to_owned()]),
        };

        let paths = options
            .iter_from_path(
                root.path(),
                crossdev::init(root.path()).unwrap(),
                false,
                walk::Order::Completion,
            )
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();

        assert!(paths.contains(&child));
    }
}
