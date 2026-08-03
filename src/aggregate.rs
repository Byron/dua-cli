use crate::{ByteFormat, InodeFilter, Throttle, WalkOptions, WalkResult, WalkRoot, crossdev};
use anyhow::Result;
use filesize::PathExt;
use owo_colors::{AnsiColors as Color, OwoColorize};
use std::path::PathBuf;
use std::time::Duration;
use std::{io, path::Path};

const CLEAR_CURRENT_LINE: &str = "\x1b[2K\r";

/// Aggregate the given `paths` and write information about them to `out` in a human-readable format.
/// If `compute_total` is set, it will write an additional line with the total size across all given `paths`.
/// If `sort_by_size_in_bytes` is set, we will sort all sizes (ascending) before outputting them.
pub fn aggregate(
    mut out: impl io::Write,
    mut err: Option<impl io::Write>,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    byte_format: ByteFormat,
    paths: Vec<PathBuf>,
) -> Result<(WalkResult, Statistics)> {
    let mut res = WalkResult::default();
    let mut stats = Statistics {
        smallest_file_in_bytes: u128::MAX,
        ..Default::default()
    };
    let num_roots = paths.len();
    let mut aggregates = paths
        .iter()
        .cloned()
        .map(|path| {
            (
                path,  // root path
                0u128, // accumulated byte size
                0u64,  // I/O error count
            )
        })
        .collect::<Vec<_>>();
    let mut device_ids = vec![0; num_roots];
    let mut completed = vec![false; num_roots];
    let mut roots = Vec::with_capacity(num_roots);
    let has_ignore_patterns = walk_options.ignore_patterns.is_some();
    for (root_idx, path) in paths.into_iter().enumerate() {
        let device_id = if walk_options.cross_filesystems {
            0
        } else {
            let Ok(device_id) = crossdev::init(&path) else {
                aggregates[root_idx].2 += 1;
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
            device_id,
        });
    }
    let mut inodes = InodeFilter::default();
    let progress = Throttle::new(Duration::from_millis(100), Duration::from_secs(1).into());
    let mut progress_visible = false;
    let mut next_output = 0;

    // With multiple roots, a shared hard link is attributed to whichever root reaches it first.
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
        let (_, num_bytes, num_errors) = &mut aggregates[root_idx];
        stats.entries_traversed += 1;
        progress.throttled(|| {
            if let Some(err) = err.as_mut() {
                write!(err, "Enumerating {} items\r", stats.entries_traversed).ok();
                progress_visible = true;
            }
        });
        match entry {
            Ok(entry) => {
                let file_size = u128::from(match &entry.metadata {
                    Ok(m)
                        if (walk_options.count_hard_links || inodes.add(m))
                            && (walk_options.cross_filesystems
                                || crossdev::is_same_device(device_ids[root_idx], m)) =>
                    {
                        if walk_options.apparent_size {
                            m.len()
                        } else {
                            entry.path().size_on_disk_fast(m).unwrap_or_else(|_| {
                                *num_errors += 1;
                                0
                            })
                        }
                    }
                    Ok(_) => 0,
                    Err(_) => {
                        *num_errors += 1;
                        0
                    }
                });
                stats.largest_file_in_bytes = stats.largest_file_in_bytes.max(file_size);
                stats.smallest_file_in_bytes = stats.smallest_file_in_bytes.min(file_size);
                *num_bytes += file_size;
            }
            Err(_) => *num_errors += 1,
        }
    }

    let total = aggregates.iter().map(|(_, bytes, _)| bytes).sum();
    res.num_errors = aggregates.iter().map(|(_, _, errors)| errors).sum();

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
    aggregates: &[(std::path::PathBuf, u128, u64)],
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
        let (path, num_bytes, num_errors) = &aggregates[*next_output];
        output_colored_path(
            out,
            path,
            *num_bytes,
            *num_errors,
            path_color_of(path),
            byte_format,
        )?;
        *next_output += 1;
    }
    Ok(())
}

fn output_sorted(
    out: &mut impl io::Write,
    mut aggregates: Vec<(std::path::PathBuf, u128, u64)>,
    byte_format: ByteFormat,
) -> std::result::Result<(), io::Error> {
    aggregates.sort_by_key(|&(_, num_bytes, _)| num_bytes);
    for (path, num_bytes, num_errors) in aggregates {
        output_colored_path(
            out,
            &path,
            num_bytes,
            num_errors,
            path_color_of(&path),
            byte_format,
        )?;
    }
    Ok(())
}

fn path_color_of(path: impl AsRef<Path>) -> Option<Color> {
    (!path.as_ref().is_file()).then_some(Color::Cyan)
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

    #[test]
    fn completed_roots_stream_in_input_order() {
        let aggregates = [("first".into(), 1, 0), ("second".into(), 2, 0)];
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
}
