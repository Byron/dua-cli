#[cfg(any(windows, target_os = "macos"))]
use crate::walk::walk_root_entries as walk_roots;
#[cfg(not(any(windows, target_os = "macos")))]
use crate::walk::walk_roots;
use crate::{crossdev, walk};
use anyhow::Context;
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

/// Gitignore-style patterns, read from files, which exclude entries from a traversal.
///
/// Patterns are matched against the path `dua` reports for an entry: normally relative to its
/// effective working directory, or relative to an input root outside that directory. So `target/`
/// excludes directories named `target` at any depth, while `/target/` excludes only `target` at
/// the traversal root of that relative path, exactly like a `.gitignore` at the top of a repository.
///
/// Matching is case-sensitive on every platform so that the same pattern file produces the same
/// report everywhere.
#[derive(Clone, Debug)]
pub struct IgnorePatterns {
    search: gix::ignore::Search,
}

impl IgnorePatterns {
    /// Read gitignore-style patterns from each file in `files`, in the given order.
    ///
    /// The usual `.gitignore` precedence applies: within a file the last matching line wins, and
    /// files listed later win over files listed earlier. Reading a file that does not exist, or
    /// that cannot be read, is an error - a silently empty pattern set would quietly report sizes
    /// the caller did not ask for.
    pub fn from_files(files: &[PathBuf]) -> anyhow::Result<Option<Self>> {
        let mut search = gix::ignore::Search::default();
        for file in files {
            let buf = std::fs::read(file).with_context(|| {
                format!("Failed to read ignore patterns from {}", file.display())
            })?;
            search.add_patterns_buffer(
                &buf,
                file.clone(),
                None,
                gix::ignore::search::Ignore::default(),
            );
        }
        let pattern_count = search
            .patterns
            .iter()
            .map(|list| list.patterns.len())
            .sum::<usize>();
        Ok(if pattern_count != 0 {
            log::info!(
                "Loaded {pattern_count} ignore pattern(s) from {file_count} file(s)",
                file_count = files.len()
            );
            Some(Self { search })
        } else {
            None
        })
    }

    /// Return `true` if `relative_path` is excluded, with `is_dir` telling directories from files
    /// so that patterns ending in `/` only match directories.
    ///
    /// Note that callers are expected to stop descending into excluded directories, which also
    /// means a negated pattern cannot bring back an entry below an excluded directory - the same
    /// restriction Git has.
    #[must_use]
    pub fn is_excluded(&self, relative_path: &Path, is_dir: bool) -> bool {
        if relative_path.as_os_str().is_empty() {
            return false;
        }
        let relative_path =
            gix::path::to_unix_separators_on_windows(gix::path::into_bstr(relative_path));
        self.search
            .pattern_matching_relative_path(
                relative_path.as_ref(),
                Some(is_dir),
                gix::ignore::glob::pattern::Case::Sensitive,
            )
            .is_some_and(|match_| !match_.pattern.is_negative())
    }

    /// Return `true` if the input `path` is excluded, resolving it the way the traversal does.
    /// These are made relative to `cwd`.
    ///
    /// Input paths that are excluded are best dropped before the walk starts, or they would be
    /// reported as being empty rather than not reported at all.
    #[must_use]
    pub fn excludes_input_path(&self, path: &Path, cwd: &Path) -> bool {
        pattern_relative_path(path, cwd, path)
            .is_some_and(|relative_path| self.is_excluded(relative_path, path.is_dir()))
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
    /// Gitignore-style patterns whose matches are left out of the traversal entirely.
    /// `None` if no pattern was configured.
    pub ignore_patterns: Option<IgnorePatterns>,
    /// Platform-specific metadata requested during traversal.
    pub metadata_options: crate::TraversalOptions,
}

/// Tells whether an entry found under the root with the given index is left out of the traversal.
type ExcludeEntry = Arc<dyn Fn(usize, &walk::Entry) -> bool + Send + Sync>;

/// A root prepared for a filesystem walk.
pub(crate) struct WalkRoot {
    /// Index used to associate emitted events with the original input path.
    pub index: usize,
    /// Path at which to start walking.
    pub path: PathBuf,
    /// Entry already collected while enumerating this root, if available.
    #[cfg(any(windows, target_os = "macos"))]
    pub entry: Option<walk::Entry>,
    /// Most specific original input root containing `path`, used as the ignore-pattern base.
    /// Choosing the longest containing root preserves the right base when input roots overlap and
    /// `path` is a subtree being refreshed.
    ///
    /// This is usually `path`, but can be a parent directory if this is the root for an
    /// interactive refresh.
    pub pattern_root: Option<PathBuf>,
    /// Device containing the root, used to constrain cross-filesystem walks.
    /// It's `0` if it couldn't be obtained or if `cross_filesystem` is `true`.
    pub device_id: u64,
}

impl WalkOptions {
    /// Return whether `path`, resolved relative to `cwd`, names an ignored directory.
    #[must_use]
    pub fn ignores_directory(&self, path: &Path, cwd: &Path) -> bool {
        ignore_directory(path, &self.ignore_dirs, cwd)
    }

    pub(crate) fn iter_from_paths(
        &self,
        roots: Vec<WalkRoot>,
        skip_root: bool,
        order: walk::Order,
    ) -> impl Iterator<Item = (usize, walk::RootEvent)> + use<> {
        let num_roots = roots
            .iter()
            .map(|root| root.index)
            .max()
            .map_or(0, |idx| idx + 1);
        let path_count = roots.len();
        let (device_ids, root_paths, indexed_roots) = roots.into_iter().fold(
            (
                vec![0; num_roots],
                vec![None; num_roots],
                Vec::with_capacity(path_count),
            ),
            |(mut device_ids, mut root_paths, mut indexed_roots), root| {
                device_ids[root.index] = root.device_id;
                root_paths[root.index] = root.pattern_root;
                #[cfg(any(windows, target_os = "macos"))]
                indexed_roots.push((
                    root.index,
                    root.entry.map_or_else(
                        || walk::Entry::from_path(&root.path, self.metadata_options),
                        Ok,
                    ),
                ));
                #[cfg(not(any(windows, target_os = "macos")))]
                indexed_roots.push((root.index, root.path));
                (device_ids, root_paths, indexed_roots)
            },
        );
        let ignore_dirs = self.ignore_dirs.clone();
        let cwd = std::env::current_dir().unwrap_or_default();
        let cross_filesystems = self.cross_filesystems;

        // Excluding an entry means pruning it from the walk *and* from the emitted events, so the
        // predicate is shared between the two. It short-circuits on the pattern set being empty to
        // keep the common case free of the path building it would otherwise do per entry.
        let is_excluded: ExcludeEntry = {
            let patterns = self.ignore_patterns.clone();
            let cwd = cwd.clone();
            Arc::new(move |root_idx: usize, entry: &walk::Entry| {
                let Some((patterns, pattern_root)) =
                    patterns.as_ref().zip(root_paths[root_idx].as_deref())
                else {
                    return false;
                };
                let path = entry.path();
                pattern_relative_path(&path, &cwd, pattern_root).is_some_and(|relative_path| {
                    patterns.is_excluded(relative_path, entry.file_type.is_dir())
                })
            })
        };
        let is_excluded_while_walking = Arc::clone(&is_excluded);
        let descend = move |root_idx: usize, entry: &walk::Entry| {
            (cross_filesystems
                || entry.metadata.as_ref().map_or(true, |metadata| {
                    crossdev::is_same_device(device_ids[root_idx], metadata)
                }))
                && (entry.depth == 0 || !ignore_directory(&entry.path(), &ignore_dirs, &cwd))
                && !is_excluded_while_walking(root_idx, entry)
        };

        let walk = walk_roots(
            indexed_roots,
            self.threads,
            order,
            self.metadata_options,
            descend,
        );

        walk.filter(move |(root_idx, event)| match event {
            walk::RootEvent::Entry(Ok(entry)) => {
                (!skip_root || entry.depth > 0) && !is_excluded(*root_idx, entry)
            }
            walk::RootEvent::Entry(Err(_)) | walk::RootEvent::Finished => true,
        })
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

/// Return the path that ignore patterns are matched against, or `None` if there is none.
///
/// `dua` reports entries by their path relative to the current directory `cwd` - it even changes into
/// the directory it was given, if it was given exactly one - so that is what patterns should see.
/// Roots outside via `traversal_root` of the current directory have no such path, and fall back to
/// being relative to the root they were found under.
fn pattern_relative_path<'a>(
    path: &'a Path,
    cwd: &Path,
    traversal_root: &Path,
) -> Option<&'a Path> {
    if path.is_relative() {
        return Some(path);
    }
    path.strip_prefix(cwd)
        .or_else(|_| path.strip_prefix(traversal_root))
        .ok()
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
            ignore_patterns: None,
            metadata_options: crate::TraversalOptions::default(),
        };

        let paths = options
            .iter_from_paths(
                vec![WalkRoot {
                    index: 0,
                    pattern_root: None,
                    path: root.path().to_owned(),
                    #[cfg(any(windows, target_os = "macos"))]
                    entry: None,
                    device_id: crossdev::init(root.path()).unwrap(),
                }],
                false,
                walk::Order::Completion,
            )
            .filter_map(|(_, event)| match event {
                walk::RootEvent::Entry(entry) => Some(entry.unwrap().path()),
                walk::RootEvent::Finished => None,
            })
            .collect::<Vec<_>>();

        assert!(paths.contains(&child));
    }

    #[test]
    fn ignore_patterns_use_gitignore_semantics() {
        let patterns = patterns_from(
            "# a comment, and the blank line below, match nothing\n\
             \n\
             *.log\n\
             !keep.log\n\
             build/\n\
             /anchored\n\
             **/node_modules/\n",
        );

        for (path, is_dir, expected) in [
            ("debug.log", false, true),
            ("nested/debug.log", false, true),
            ("keep.log", false, false),
            ("build", true, true),
            // A trailing '/' in a pattern only ever matches directories.
            ("build", false, false),
            ("anchored", true, true),
            ("nested/anchored", true, false),
            ("nested/deeply/node_modules", true, true),
            ("src", true, false),
            ("", true, false),
        ] {
            assert_eq!(
                patterns.is_excluded(Path::new(path), is_dir),
                expected,
                "expected is_excluded({path:?}, is_dir={is_dir}) to be {expected}"
            );
        }
    }

    #[test]
    fn later_ignore_files_win_over_earlier_ones() {
        let dir = tempfile::tempdir().unwrap();
        let (first, second) = (dir.path().join("first"), dir.path().join("second"));
        std::fs::write(&first, "*.tmp\n").unwrap();
        std::fs::write(&second, "!important.tmp\n").unwrap();

        let patterns = IgnorePatterns::from_files(&[first, second])
            .unwrap()
            .unwrap();

        assert!(patterns.is_excluded(Path::new("scratch.tmp"), false));
        assert!(
            !patterns.is_excluded(Path::new("important.tmp"), false),
            "the negation in the second file overrides the first file"
        );
    }

    #[test]
    fn unreadable_ignore_files_are_an_error() {
        let err = IgnorePatterns::from_files(&[PathBuf::from("does-not-exist")])
            .expect_err("a missing pattern file must not be silently skipped");
        assert!(err.to_string().contains("does-not-exist"));
    }

    #[test]
    fn empty_ignore_files_produce_none() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), "# comment only\n").unwrap();
        assert!(
            IgnorePatterns::from_files(&[file.path().to_owned()])
                .unwrap()
                .is_none(),
            "no patterns means no need to match anything"
        );
    }

    #[test]
    fn matching_entries_are_pruned_from_the_walk() {
        let root = tempfile::tempdir().unwrap();
        for dir in ["keep", "build", "keep/node_modules"] {
            std::fs::create_dir(root.path().join(dir)).unwrap();
        }
        for file in [
            "keep/main.rs",
            "keep/debug.log",
            "build/artifact",
            "keep/node_modules/dep",
        ] {
            std::fs::write(root.path().join(file), b"x").unwrap();
        }

        assert_eq!(
            walk_with_patterns(root.path(), "*.log\n**/node_modules/\n"),
            [
                PathBuf::new(),
                PathBuf::from("build"),
                PathBuf::from("build/artifact"),
                PathBuf::from("keep"),
                PathBuf::from("keep/main.rs"),
            ],
            "excluded directories are pruned along with everything below them, \
             and excluded files never show up"
        );
    }

    #[test]
    fn patterns_match_the_path_dua_reports() {
        let here = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let (cwd, outside) = (here.path(), elsewhere.path());

        for dir in ["target", "nested", "nested/target"] {
            std::fs::create_dir_all(here.path().join(dir)).unwrap();
        }
        assert_eq!(
            walk_with_patterns(here.path(), "/target/\n"),
            [
                PathBuf::new(),
                PathBuf::from("nested"),
                PathBuf::from("nested/target"),
            ],
            "an anchored pattern excludes only the top-level target"
        );

        // Several inputs below the current directory also keep their reported path.
        let (entry, root) = (cwd.join("a").join("b"), cwd.join("a"));
        let reported = Path::new("a").join("b");
        assert_eq!(
            pattern_relative_path(&entry, cwd, &root),
            Some(reported.as_path())
        );

        // Inputs elsewhere have no path relative to the current directory, so they fall back to
        // being relative to the input they were found under.
        let entry = outside.join("syslog");
        assert_eq!(
            pattern_relative_path(&entry, cwd, outside),
            Some(Path::new("syslog"))
        );

        // Such an input is empty relative to itself, and so never matches a pattern.
        assert_eq!(
            pattern_relative_path(outside, cwd, outside),
            Some(Path::new(""))
        );
        assert!(!patterns_from("*\n").is_excluded(Path::new(""), true));
    }

    #[test]
    fn subtree_walk_keeps_the_original_pattern_root() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("secret"), []).unwrap();
        std::fs::write(nested.join("visible"), []).unwrap();
        let options = WalkOptions {
            threads: 1,
            count_hard_links: false,
            apparent_size: false,
            cross_filesystems: true,
            ignore_dirs: BTreeSet::default(),
            ignore_patterns: Some(patterns_from("nested/secret\n")),
            metadata_options: crate::TraversalOptions::default(),
        };

        let paths = options
            .iter_from_paths(
                vec![WalkRoot {
                    index: 0,
                    path: nested,
                    #[cfg(any(windows, target_os = "macos"))]
                    entry: None,
                    pattern_root: Some(root.path().to_owned()),
                    device_id: 0,
                }],
                false,
                walk::Order::Completion,
            )
            .filter_map(|(_, event)| match event {
                walk::RootEvent::Entry(entry) => Some(entry.unwrap().file_name),
                walk::RootEvent::Finished => None,
            })
            .collect::<Vec<_>>();

        assert!(!paths.iter().any(|path| path == "secret"));
        assert!(paths.iter().any(|path| path == "visible"));
    }

    #[test]
    fn excluded_input_paths_are_dropped_before_the_walk() {
        // Relies on the crate directory being current, like `test_ignore_directories` above.
        let patterns = patterns_from("src/\n*.toml\n");
        let cwd = std::env::current_dir().unwrap();

        assert!(
            patterns.excludes_input_path(Path::new("src"), &cwd),
            "a directory pattern matches a directory given as input"
        );
        assert!(patterns.excludes_input_path(Path::new("Cargo.toml"), &cwd));
        assert!(!patterns.excludes_input_path(Path::new("README.md"), &cwd));
    }

    fn patterns_from(contents: &str) -> IgnorePatterns {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), contents).unwrap();
        IgnorePatterns::from_files(&[file.path().to_owned()])
            .unwrap()
            .unwrap()
    }

    fn walk_with_patterns(root: &Path, contents: &str) -> Vec<PathBuf> {
        let options = WalkOptions {
            threads: 2,
            count_hard_links: false,
            apparent_size: false,
            cross_filesystems: true,
            ignore_dirs: BTreeSet::default(),
            ignore_patterns: Some(patterns_from(contents)),
            metadata_options: crate::TraversalOptions::default(),
        };

        let mut paths = options
            .iter_from_paths(
                vec![WalkRoot {
                    index: 0,
                    pattern_root: Some(root.to_owned()),
                    path: root.to_owned(),
                    #[cfg(any(windows, target_os = "macos"))]
                    entry: None,
                    device_id: crossdev::init(root).unwrap(),
                }],
                false,
                walk::Order::Completion,
            )
            .filter_map(|(_, event)| match event {
                walk::RootEvent::Entry(entry) => {
                    Some(entry.unwrap().path().strip_prefix(root).unwrap().to_owned())
                }
                walk::RootEvent::Finished => None,
            })
            .collect::<Vec<_>>();
        paths.sort();
        paths
    }
}
