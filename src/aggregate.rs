use crate::{ByteFormat, InodeFilter, Throttle, WalkOptions, WalkResult, WalkRoot, crossdev};
use anyhow::Result;
#[cfg(not(any(windows, target_os = "macos")))]
use filesize::PathExt;
use owo_colors::{AnsiColors as Color, OwoColorize};
use std::path::PathBuf;
use std::time::Duration;
use std::{io, path::Path};

#[cfg(not(any(windows, target_os = "macos")))]
fn size_on_disk(entry: &crate::walk::Entry, metadata: &crate::walk::Metadata) -> io::Result<u64> {
    entry.path().size_on_disk_fast(metadata)
}

#[cfg(windows)]
#[allow(clippy::unnecessary_wraps)]
fn size_on_disk(entry: &crate::walk::Entry, metadata: &crate::walk::Metadata) -> io::Result<u64> {
    Ok(if entry.file_type.is_dir() {
        0
    } else {
        metadata.allocated_size()
    })
}

const CLEAR_CURRENT_LINE: &str = "\x1b[2K\r";

/// Accumulated output state for one input root, retained until roots can be emitted in the
/// requested order.
struct Aggregate {
    /// Path printed for this root.
    path: PathBuf,
    /// Sum of the accepted entries' apparent or allocated sizes.
    bytes: u128,
    /// Number of root, entry, metadata, or size-query errors encountered.
    errors: u64,
    /// Whether the root is a file, used to distinguish file and directory output styling.
    is_file: bool,
}

impl Aggregate {
    fn path_color(&self) -> Option<Color> {
        (!self.is_file).then_some(Color::Cyan)
    }
}

/// Aggregate the given `paths` and write information about them to `out` in a human-readable format.
/// If `compute_total` is set, it will write an additional line with the total size across all given `paths`.
/// If `sort_by_size_in_bytes` is set, we will sort all sizes (ascending) before outputting them.
pub fn aggregate(
    out: impl io::Write,
    err: Option<impl io::Write>,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    byte_format: ByteFormat,
    paths: Vec<PathBuf>,
) -> Result<(WalkResult, Statistics)> {
    aggregate_inner(
        out,
        err,
        walk_options,
        compute_total,
        sort_by_size_in_bytes,
        byte_format,
        paths.into_iter().map(|path| (path, None)),
    )
}

/// Aggregate bulk-enumerated directory entries without querying their paths for metadata again.
///
/// Reuses each entry's existing metadata and filesystem identity while preserving the output and
/// traversal behavior of [`aggregate`].
#[cfg(any(windows, target_os = "macos"))]
pub fn aggregate_entries(
    out: impl io::Write,
    err: Option<impl io::Write>,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    byte_format: ByteFormat,
    entries: Vec<dua_core::Entry>,
) -> Result<(WalkResult, Statistics)> {
    aggregate_inner(
        out,
        err,
        walk_options,
        compute_total,
        sort_by_size_in_bytes,
        byte_format,
        entries.into_iter().map(|entry| (entry.path(), Some(entry))),
    )
}

fn aggregate_inner(
    mut out: impl io::Write,
    mut err: Option<impl io::Write>,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    byte_format: ByteFormat,
    inputs: impl ExactSizeIterator<Item = (PathBuf, Option<crate::walk::Entry>)>,
) -> Result<(WalkResult, Statistics)> {
    #[cfg(target_os = "macos")]
    let apfs_clone_accounting = walk_options.metadata_options.apfs_clone_metadata;
    let mut res = WalkResult::default();
    let mut stats = Statistics {
        smallest_file_in_bytes: u128::MAX,
        ..Default::default()
    };
    let num_roots = inputs.len();
    let mut aggregates = Vec::with_capacity(num_roots);
    let mut device_ids = vec![0; num_roots];
    let mut completed = vec![false; num_roots];
    let mut roots = Vec::with_capacity(num_roots);
    let has_ignore_patterns = walk_options.ignore_patterns.is_some();
    for (root_idx, (path, prepared_entry)) in inputs.enumerate() {
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = prepared_entry;

        aggregates.push(Aggregate {
            path: path.clone(),
            bytes: 0,
            errors: 0,
            is_file: false,
        });
        let device_id = if walk_options.cross_filesystems {
            0
        } else {
            #[cfg(target_os = "macos")]
            let root_device_id = prepared_entry
                .as_ref()
                .and_then(|entry| entry.metadata.as_ref().ok())
                .map_or_else(|| crossdev::init(&path), |metadata| Ok(metadata.dev()));
            #[cfg(not(target_os = "macos"))]
            let root_device_id = crossdev::init(&path);

            let Ok(device_id) = root_device_id else {
                aggregates[root_idx].errors += 1;
                completed[root_idx] = true;
                continue;
            };
            device_id
        };
        device_ids[root_idx] = device_id;
        roots.push(WalkRoot {
            index: root_idx,
            pattern_root: has_ignore_patterns.then(|| path.clone()),
            path,
            #[cfg(any(windows, target_os = "macos"))]
            entry: prepared_entry,
            device_id,
        });
    }
    let mut inodes = InodeFilter::default();
    let progress = Throttle::new(Duration::from_millis(100), Duration::from_secs(1).into());
    let mut progress_visible = false;
    let mut next_output = 0;

    // Shared hard links and, when enabled, cloned data belong to the first root that reaches them.
    for (root_idx, event) in
        walk_options.iter_from_paths(roots, false, crate::walk::Order::Completion)
    {
        let entry = match event {
            crate::walk::RootEvent::Entry(entry) => entry,
            crate::walk::RootEvent::Finished => {
                completed[root_idx] = true;
                if !sort_by_size_in_bytes {
                    output_completed(
                        &mut out,
                        &mut err,
                        &aggregates,
                        &completed,
                        &mut next_output,
                        &mut progress_visible,
                        byte_format,
                    )?;
                }
                continue;
            }
        };
        let aggregate = &mut aggregates[root_idx];
        stats.entries_traversed += 1;
        progress.throttled(|| {
            if let Some(err) = err.as_mut() {
                write!(err, "Enumerating {} items\r", stats.entries_traversed).ok();
                progress_visible = true;
            }
        });
        match entry {
            Ok(entry) => {
                if entry.depth == 0 {
                    aggregate.is_file = entry.file_type.is_file()
                        || entry.file_type.is_symlink() && entry.path().is_file();
                }
                let file_size = u128::from(match &entry.metadata {
                    Ok(m)
                        if (walk_options.count_hard_links || inodes.add(&entry, m))
                            && (walk_options.cross_filesystems
                                || crossdev::is_same_device(device_ids[root_idx], m)) =>
                    {
                        if walk_options.apparent_size {
                            m.len()
                        } else {
                            #[cfg(target_os = "macos")]
                            if apfs_clone_accounting {
                                inodes.allocated_size(m)
                            } else {
                                m.allocated_size()
                            }
                            #[cfg(not(target_os = "macos"))]
                            {
                                size_on_disk(&entry, m).unwrap_or_else(|_| {
                                    aggregate.errors += 1;
                                    0
                                })
                            }
                        }
                    }
                    Ok(_) => 0,
                    Err(_) => {
                        aggregate.errors += 1;
                        0
                    }
                });
                stats.largest_file_in_bytes = stats.largest_file_in_bytes.max(file_size);
                stats.smallest_file_in_bytes = stats.smallest_file_in_bytes.min(file_size);
                aggregate.bytes += file_size;
            }
            Err(_) => aggregate.errors += 1,
        }
    }

    let total = aggregates.iter().map(|aggregate| aggregate.bytes).sum();
    res.num_errors = aggregates.iter().map(|aggregate| aggregate.errors).sum();

    if stats.entries_traversed == 0 {
        stats.smallest_file_in_bytes = 0;
    }

    if progress_visible && let Some(err) = err.as_mut() {
        write!(err, "{CLEAR_CURRENT_LINE}").ok();
    }

    if sort_by_size_in_bytes {
        output_sorted(&mut out, aggregates, byte_format)?;
    } else {
        // Be sure failed roots are also printed, as they lack a `Finished` event,
        // the traversal never starts on them.
        output_completed(
            &mut out,
            &mut err,
            &aggregates,
            &completed,
            &mut next_output,
            &mut progress_visible,
            byte_format,
        )?;
        debug_assert_eq!(next_output, num_roots);
    }

    if num_roots > 1 && compute_total {
        output_colored_path(
            &mut out,
            Path::new("total"),
            total,
            res.num_errors,
            None,
            byte_format,
        )?;
    }
    Ok((res, stats))
}

/// Write the contiguous run of completed roots starting at `next_output`, preserving input order.
/// Clears a visible progress line before writing the first completed root.
/// `progress_visible` tracks if progress information is currently shown, taking up the last line.
fn output_completed<W: io::Write, E: io::Write>(
    out: &mut W,
    err: &mut Option<E>,
    aggregates: &[Aggregate],
    completed: &[bool],
    next_output: &mut usize,
    progress_visible: &mut bool,
    byte_format: ByteFormat,
) -> io::Result<()> {
    let must_report_completed_path = completed.get(*next_output).copied() == Some(true);
    // Remove the transient progress line before writing permanent results to the terminal.
    if must_report_completed_path && *progress_visible {
        if let Some(err) = err.as_mut() {
            write!(err, "{CLEAR_CURRENT_LINE}").ok();
        }
        *progress_visible = false;
    }
    while completed.get(*next_output).copied() == Some(true) {
        let aggregate = &aggregates[*next_output];
        output_colored_path(
            out,
            &aggregate.path,
            aggregate.bytes,
            aggregate.errors,
            aggregate.path_color(),
            byte_format,
        )?;
        *next_output += 1;
    }
    Ok(())
}

fn output_sorted(
    out: &mut impl io::Write,
    mut aggregates: Vec<Aggregate>,
    byte_format: ByteFormat,
) -> std::result::Result<(), io::Error> {
    aggregates.sort_by_key(|aggregate| aggregate.bytes);
    for aggregate in aggregates {
        output_colored_path(
            out,
            &aggregate.path,
            aggregate.bytes,
            aggregate.errors,
            aggregate.path_color(),
            byte_format,
        )?;
    }
    Ok(())
}

fn output_colored_path(
    out: &mut impl io::Write,
    path: impl AsRef<Path>,
    num_bytes: u128,
    num_errors: u64,
    path_color: Option<Color>,
    byte_format: ByteFormat,
) -> std::result::Result<(), io::Error> {
    let size = byte_format.display(num_bytes).to_string();
    let size = size.green();
    let size_width = byte_format.width();
    let path = path.as_ref().display();

    let errors = if num_errors != 0 {
        format!(
            "  <{num_errors} IO Error{plural_s}>",
            plural_s = if num_errors > 1 { "s" } else { "" }
        )
    } else {
        String::new()
    };

    if let Some(color) = path_color {
        writeln!(out, "{size:>size_width$} {}{errors}", path.color(color))
    } else {
        writeln!(out, "{size:>size_width$} {path}{errors}")
    }
}

/// Statistics obtained during a filesystem walk
#[derive(Default, Debug)]
pub struct Statistics {
    /// The amount of entries we have seen during filesystem traversal
    pub entries_traversed: u64,
    /// The size of the smallest file encountered in bytes
    pub smallest_file_in_bytes: u128,
    /// The size of the largest file encountered in bytes
    pub largest_file_in_bytes: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn byte_counts(out: &[u8]) -> Vec<u128> {
        let out = std::str::from_utf8(out).unwrap();
        out.match_indices(" b")
            .map(|(unit, _)| {
                out[..unit]
                    .chars()
                    .rev()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
                    .parse()
                    .unwrap()
            })
            .collect()
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn file_as_root_keeps_cached_metadata_after_removal() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("prepared-file");

        for sort_by_size_in_bytes in [false, true] {
            std::fs::write(&path, b"cached metadata").unwrap();
            let expected_size = std::fs::metadata(&path).unwrap().len();
            let entry = dua_core::read_dir(directory.path(), dua_core::Options::default())
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            std::fs::remove_file(&path).unwrap();

            let mut out = Vec::new();
            let (result, statistics) = aggregate_entries(
                &mut out,
                None::<Vec<u8>>,
                WalkOptions {
                    threads: 1,
                    count_hard_links: false,
                    apparent_size: true,
                    cross_filesystems: false,
                    ignore_dirs: std::collections::BTreeSet::default(),
                    ignore_patterns: None,
                    metadata_options: crate::TraversalOptions::default(),
                },
                false,
                sort_by_size_in_bytes,
                ByteFormat::Bytes,
                vec![entry],
            )
            .unwrap();

            assert_eq!(result.num_errors, 0);
            assert_eq!(statistics.entries_traversed, 1);
            assert_eq!(byte_counts(&out), [u128::from(expected_size)]);
            let out = String::from_utf8(out).unwrap();
            assert!(
                out.contains(&format!(" {}\n", path.display())),
                "the bulk reader must preserve the cached file type after removal; querying the \
                 path again would fail, leave `is_file` false, and incorrectly add cyan directory \
                 coloring: {out:?}"
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn overlapping_directory_roots_preserve_stat_link_count_cycles() {
        use std::os::unix::fs::MetadataExt;

        const OVERLAPPING_VISITS: u64 = 5;

        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("parent");
        let child = parent.join("child");
        let grandchild = child.join("grandchild");
        std::fs::create_dir_all(&grandchild).unwrap();
        let file = grandchild.join("file");
        std::fs::write(&file, b"repeated directory contents").unwrap();

        let parent_metadata = std::fs::symlink_metadata(&parent).unwrap();
        let child_metadata = std::fs::symlink_metadata(&child).unwrap();
        let grandchild_metadata = std::fs::symlink_metadata(&grandchild).unwrap();
        let file_metadata = std::fs::symlink_metadata(&file).unwrap();
        assert!(
            child_metadata.nlink() > 1,
            "expected multiple links for {child:?}, got {}",
            child_metadata.nlink()
        );
        assert!(
            grandchild_metadata.nlink() > 1,
            "expected multiple links for {grandchild:?}, got {}",
            grandchild_metadata.nlink()
        );

        let roots = vec![parent, child.clone(), child.clone(), child.clone(), child];

        for count_hard_links in [false, true] {
            let directory_visits = |metadata: &std::fs::Metadata| {
                if count_hard_links || metadata.nlink() <= 1 {
                    OVERLAPPING_VISITS
                } else {
                    OVERLAPPING_VISITS.div_ceil(metadata.nlink())
                }
            };
            let expected = u128::from(parent_metadata.len())
                + u128::from(directory_visits(&child_metadata) * child_metadata.len())
                + u128::from(directory_visits(&grandchild_metadata) * grandchild_metadata.len())
                + u128::from(OVERLAPPING_VISITS * file_metadata.len());

            let mut out = Vec::new();
            let result = aggregate(
                &mut out,
                None::<Vec<u8>>,
                WalkOptions {
                    threads: 1,
                    count_hard_links,
                    apparent_size: true,
                    cross_filesystems: true,
                    ignore_dirs: std::collections::BTreeSet::default(),
                    ignore_patterns: None,
                    metadata_options: crate::TraversalOptions::default(),
                },
                true,
                false,
                ByteFormat::Bytes,
                roots.clone(),
            )
            .unwrap();

            assert_eq!(result.0.num_errors, 0);
            assert_eq!(
                byte_counts(&out).last().copied(),
                Some(expected),
                "overlapping directory totals with count_hard_links={count_hard_links}"
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn full_apfs_clones_preserve_private_forks_hard_links_and_logical_sizes() {
        use std::io::Write as _;
        use std::os::unix::fs::MetadataExt;

        const STAT_BLOCK_BYTES: u128 = 512;
        const RESOURCE_FORK_BYTES: usize = 4096;
        const DATA_FORK_BYTES: usize = RESOURCE_FORK_BYTES * 2;

        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("original");
        let clone = directory.path().join("clone");
        let partial_clone = directory.path().join("partial-clone");
        let hard_link = directory.path().join("hard-link");
        std::fs::write(&original, vec![7; DATA_FORK_BYTES]).unwrap();
        // On Apple platforms, std::fs::copy first tries fclonefileat(2), so these become
        // copy-on-write APFS clones with distinct inodes and shared data blocks. A non-APFS
        // fallback copies the bytes instead and intentionally fails the clone-ID assertions below.
        std::fs::copy(&original, &clone).unwrap();
        std::fs::copy(&original, &partial_clone).unwrap();
        std::fs::write(clone.join("..namedfork/rsrc"), vec![5; RESOURCE_FORK_BYTES]).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&partial_clone)
            .unwrap()
            .write_all(&[9])
            .unwrap();
        std::fs::hard_link(&original, &hard_link).unwrap();

        let original_metadata = std::fs::metadata(&original).unwrap();
        let clone_metadata = std::fs::metadata(&clone).unwrap();
        let partial_metadata = std::fs::metadata(&partial_clone).unwrap();
        let directory_metadata = std::fs::metadata(directory.path()).unwrap();
        let allocated_size = u128::from(original_metadata.blocks()) * STAT_BLOCK_BYTES;
        let clone_allocated_size = u128::from(clone_metadata.blocks()) * STAT_BLOCK_BYTES;
        let partial_allocated_size = u128::from(partial_metadata.blocks()) * STAT_BLOCK_BYTES;
        let directory_allocated_size = u128::from(directory_metadata.blocks()) * STAT_BLOCK_BYTES;
        let clone_private_size = clone_allocated_size - allocated_size;
        let apparent_size = u128::from(original_metadata.len());
        let directory_apparent_size = u128::from(directory_metadata.len());
        assert_ne!(
            clone_private_size, 0,
            "the cloned fixture must own separately allocated resource-fork blocks"
        );

        let directory_root = || vec![directory.path().to_owned()];
        for (case, roots, apparent_size_requested, count_hard_links, expected_total) in [
            (
                "full and partial clones retain private forks",
                directory_root(),
                false,
                false,
                directory_allocated_size
                    + allocated_size
                    + partial_allocated_size
                    + clone_private_size,
            ),
            (
                "explicit hard links remain counted",
                directory_root(),
                false,
                true,
                directory_allocated_size
                    + allocated_size * 2
                    + partial_allocated_size
                    + clone_private_size,
            ),
            (
                "logical sizes remain independent",
                directory_root(),
                true,
                false,
                directory_apparent_size + apparent_size * 3,
            ),
            (
                "logical sizes count requested hard links",
                directory_root(),
                true,
                true,
                directory_apparent_size + apparent_size * 4,
            ),
            (
                "clone-first roots preserve explicit hard links",
                vec![clone.clone(), original.clone(), hard_link],
                false,
                true,
                allocated_size * 2 + clone_private_size,
            ),
            (
                "repeated cloned roots are not distinct clone inodes",
                vec![clone.clone(), clone, original],
                false,
                false,
                clone_allocated_size * 2,
            ),
        ] {
            let mut output = Vec::new();
            let (result, _) = aggregate(
                &mut output,
                None::<Vec<u8>>,
                WalkOptions {
                    threads: 2,
                    count_hard_links,
                    apparent_size: apparent_size_requested,
                    cross_filesystems: true,
                    ignore_dirs: std::collections::BTreeSet::default(),
                    ignore_patterns: None,
                    metadata_options: crate::TraversalOptions {
                        apfs_clone_metadata: true,
                    },
                },
                true,
                true,
                ByteFormat::Bytes,
                roots,
            )
            .unwrap();

            assert_eq!(result.num_errors, 0, "unexpected traversal errors: {case}");
            assert_eq!(
                byte_counts(&output).last().copied(),
                Some(expected_total),
                "incorrect aggregate for {case}"
            );
        }

        let entries = dua_core::read_dir(
            directory.path(),
            dua_core::Options {
                apfs_clone_metadata: true,
            },
        )
        .unwrap()
        .collect::<std::io::Result<Vec<_>>>()
        .unwrap();
        let mut output = Vec::new();
        let (result, _) = aggregate_entries(
            &mut output,
            None::<Vec<u8>>,
            WalkOptions {
                threads: 2,
                count_hard_links: false,
                apparent_size: false,
                cross_filesystems: false,
                ignore_dirs: std::collections::BTreeSet::default(),
                ignore_patterns: None,
                metadata_options: crate::TraversalOptions {
                    apfs_clone_metadata: true,
                },
            },
            true,
            true,
            ByteFormat::Bytes,
            entries,
        )
        .unwrap();

        assert_eq!(
            result.num_errors, 0,
            "prepared sibling roots should retain their cached filesystem identities"
        );
        assert_eq!(
            byte_counts(&output).last().copied(),
            Some(allocated_size + partial_allocated_size + clone_private_size),
            "prepared sibling roots should deduplicate cloned data, retain private forks, \
             and omit their parent directory"
        );
    }

    #[test]
    fn completed_roots_stream_in_input_order() {
        let aggregates = [
            Aggregate {
                path: "first".into(),
                bytes: 1,
                errors: 0,
                is_file: false,
            },
            Aggregate {
                path: "second".into(),
                bytes: 2,
                errors: 0,
                is_file: false,
            },
        ];
        let mut completed = [false, true];
        let mut next_output = 0;
        let mut progress_visible = true;
        let mut out = Vec::new();
        let mut err = Some(Vec::new());

        output_completed(
            &mut out,
            &mut err,
            &aggregates,
            &completed,
            &mut next_output,
            &mut progress_visible,
            ByteFormat::Bytes,
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "later roots must not overtake earlier roots"
        );

        completed[0] = true;
        output_completed(
            &mut out,
            &mut err,
            &aggregates,
            &completed,
            &mut next_output,
            &mut progress_visible,
            ByteFormat::Bytes,
        )
        .unwrap();

        assert_eq!(byte_counts(&out), [1, 2]);
        let out = String::from_utf8(out).unwrap();
        assert!(
            out.find("first").unwrap() < out.find("second").unwrap(),
            "the first root is also emitted first"
        );
        assert_eq!(next_output, 2, "output stopped at root {next_output}");
        assert_eq!(
            err.as_deref(),
            Some(CLEAR_CURRENT_LINE.as_bytes()),
            "unexpected progress cleanup: {err:?}"
        );
        assert!(!progress_visible, "progress remained visible after cleanup");
    }

    #[test]
    fn fast_roots_do_not_emit_terminal_erases() {
        let dir = tempfile::tempdir().unwrap();
        let paths = [dir.path().join("a"), dir.path().join("b")];
        for path in &paths {
            std::fs::write(path, []).unwrap();
        }
        let mut out = Vec::new();
        let mut err = Vec::new();

        aggregate(
            &mut out,
            Some(&mut err),
            WalkOptions {
                threads: 2,
                count_hard_links: true,
                apparent_size: false,
                cross_filesystems: true,
                ignore_dirs: std::collections::BTreeSet::default(),
                ignore_patterns: None,
                metadata_options: crate::TraversalOptions::default(),
            },
            true,
            true,
            ByteFormat::Metric,
            paths.into(),
        )
        .unwrap();

        assert!(
            err.is_empty(),
            "fast roots should not clear unseen progress"
        );
    }

    #[cfg(unix)]
    #[test]
    fn root_device_error_is_reported() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("dangling");
        symlink(dir.path().join("missing"), &root).unwrap();

        let (result, _) = aggregate(
            Vec::new(),
            None::<Vec<u8>>,
            WalkOptions {
                threads: 1,
                count_hard_links: true,
                apparent_size: true,
                cross_filesystems: false,
                ignore_dirs: std::collections::BTreeSet::default(),
                ignore_patterns: None,
                metadata_options: crate::TraversalOptions::default(),
            },
            false,
            true,
            ByteFormat::Bytes,
            vec![root],
        )
        .unwrap();

        assert_eq!(result.num_errors, 1);
    }

    #[test]
    fn ignored_patterns_are_left_out_of_the_reported_size() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("cache")).unwrap();
        std::fs::write(dir.path().join("kept"), [0; 64]).unwrap();
        std::fs::write(dir.path().join("cache/blob"), [0; 4096]).unwrap();

        // Kept outside the traversed tree so they are not counted themselves.
        let patterns_dir = tempfile::tempdir().unwrap();
        let ignore_cache = patterns_dir.path().join("cache-only");
        let ignore_both = patterns_dir.path().join("cache-and-kept");
        std::fs::write(&ignore_cache, "cache/\n").unwrap();
        std::fs::write(&ignore_both, "cache/\nkept\n").unwrap();

        let aggregate_with = |ignore_from: &[PathBuf]| -> u128 {
            let mut out = Vec::new();
            aggregate(
                &mut out,
                None::<&mut Vec<u8>>,
                WalkOptions {
                    threads: 2,
                    count_hard_links: true,
                    apparent_size: true,
                    cross_filesystems: true,
                    ignore_dirs: std::collections::BTreeSet::default(),
                    ignore_patterns: crate::IgnorePatterns::from_files(ignore_from).unwrap(),
                    metadata_options: crate::TraversalOptions::default(),
                },
                false,
                true,
                ByteFormat::Bytes,
                vec![dir.path().to_owned()],
            )
            .unwrap();
            byte_counts(&out)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("expected a byte count in {out:?}"))
        };

        // Directory entries have a size of their own that differs per filesystem - 4096 bytes on
        // ext4, next to nothing on APFS - so only differences between runs are compared here.
        let full = aggregate_with(&[]);
        let without_cache = aggregate_with(&[ignore_cache]);
        let without_either = aggregate_with(&[ignore_both]);

        assert!(
            full >= 4096 + 64,
            "without patterns both files are counted, got {full}"
        );
        assert!(
            full - without_cache >= 4096,
            "excluding `cache/` drops at least the 4096-byte file inside it, \
             but only {} bytes disappeared",
            full - without_cache
        );
        assert_eq!(
            without_cache - without_either,
            64,
            "the 64-byte file is still counted until a pattern matches it too"
        );
    }
    #[cfg(windows)]
    #[test]
    fn windows_disk_size_survives_removing_the_entry_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file");
        std::fs::write(&path, b"content").unwrap();
        let entry =
            crate::walk::Entry::from_path(&path, crate::TraversalOptions::default()).unwrap();
        let metadata = entry.metadata.as_ref().unwrap();
        let expected = metadata.allocated_size();
        std::fs::remove_file(path).unwrap();
        assert_eq!(
            size_on_disk(&entry, metadata).unwrap(),
            expected,
            "Windows aggregation should use the already-enumerated allocation size"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_disk_size_preserves_zero_sized_directories() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file"), b"content").unwrap();
        let entry =
            crate::walk::Entry::from_path(dir.path(), crate::TraversalOptions::default()).unwrap();
        let metadata = entry.metadata.as_ref().unwrap();
        assert_eq!(size_on_disk(&entry, metadata).unwrap(), 0);
    }
}
