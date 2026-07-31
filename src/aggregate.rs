use crate::{ByteFormat, InodeFilter, Throttle, WalkOptions, WalkResult, crossdev};
use anyhow::Result;
use filesize::PathExt;
use owo_colors::{AnsiColors as Color, OwoColorize};
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
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<(WalkResult, Statistics)> {
    let mut res = WalkResult::default();
    let mut stats = Statistics {
        smallest_file_in_bytes: u128::MAX,
        ..Default::default()
    };
    let mut total = 0;
    let mut num_roots = 0;
    let mut aggregates = Vec::new();
    let mut inodes = InodeFilter::default();
    let progress = Throttle::new(Duration::from_millis(100), Duration::from_secs(1).into());
    let mut progress_visible = false;

    for path in paths {
        num_roots += 1;
        let mut num_bytes = 0u128;
        let mut num_errors = 0u64;
        let Ok(device_id) = crossdev::init(path.as_ref()) else {
            num_errors += 1;
            res.num_errors += 1;
            aggregates.push((path.as_ref().to_owned(), num_bytes, num_errors));
            continue;
        };
        for entry in walk_options.iter_from_path(
            path.as_ref(),
            device_id,
            false,
            crate::walk::Order::Completion,
        ) {
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
                                    || crossdev::is_same_device(device_id, m)) =>
                        {
                            if walk_options.apparent_size {
                                m.len()
                            } else {
                                entry.path().size_on_disk_fast(m).unwrap_or_else(|_| {
                                    num_errors += 1;
                                    0
                                })
                            }
                        }
                        Ok(_) => 0,
                        Err(_) => {
                            num_errors += 1;
                            0
                        }
                    });
                    stats.largest_file_in_bytes = stats.largest_file_in_bytes.max(file_size);
                    stats.smallest_file_in_bytes = stats.smallest_file_in_bytes.min(file_size);
                    num_bytes += file_size;
                }
                Err(_) => num_errors += 1,
            }
        }

        if sort_by_size_in_bytes {
            aggregates.push((path.as_ref().to_owned(), num_bytes, num_errors));
        } else {
            if progress_visible {
                if let Some(err) = err.as_mut() {
                    write!(err, "{CLEAR_CURRENT_LINE}").ok();
                }
                progress_visible = false;
            }
            output_colored_path(
                &mut out,
                &path,
                num_bytes,
                num_errors,
                path_color_of(&path),
                byte_format,
            )?;
        }
        total += num_bytes;
        res.num_errors += num_errors;
    }

    if stats.entries_traversed == 0 {
        stats.smallest_file_in_bytes = 0;
    }

    if progress_visible && let Some(err) = err.as_mut() {
        write!(err, "{CLEAR_CURRENT_LINE}").ok();
    }

    if sort_by_size_in_bytes {
        output_sorted(&mut out, aggregates, byte_format)?;
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
            },
            true,
            true,
            ByteFormat::Metric,
            paths,
        )
        .unwrap();

        assert!(
            err.is_empty(),
            "fast roots should not clear unseen progress"
        );
    }
}
