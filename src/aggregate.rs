use crate::{crossdev, ByteFormat, InodeFilter, Throttle, WalkOptions, WalkResult};
use anyhow::Result;
use filesize::PathExt;
use owo_colors::{AnsiColors as Color, OwoColorize};
use std::time::Duration;
use std::{io, path::Path};

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

    for path in paths.into_iter() {
        num_roots += 1;
        let mut num_bytes = 0u128;
        let mut num_errors = 0u64;
        let device_id = match crossdev::init(path.as_ref()) {
            Ok(id) => id,
            Err(_) => {
                num_errors += 1;
                res.num_errors += 1;
                aggregates.push((path.as_ref().to_owned(), num_bytes, num_errors));
                continue;
            }
        };
        for entry in walk_options.iter_from_path(path.as_ref(), device_id, false) {
            stats.entries_traversed += 1;
            progress.throttled(|| {
                if let Some(err) = err.as_mut() {
                    write!(err, "Enumerating {} items\r", stats.entries_traversed).ok();
                }
            });
            match entry {
                Ok(entry) => {
                    let file_size = match entry.client_state {
                        Some(Ok(ref m))
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
                        Some(Ok(_)) => 0,
                        Some(Err(_)) => {
                            num_errors += 1;
                            0
                        }
                        None => 0, // ignore directory
                    } as u128;
                    stats.largest_file_in_bytes = stats.largest_file_in_bytes.max(file_size);
                    stats.smallest_file_in_bytes = stats.smallest_file_in_bytes.min(file_size);
                    num_bytes += file_size;
                }
                Err(_) => num_errors += 1,
            }
        }

        if let Some(err) = err.as_mut() {
            write!(err, "\x1b[2K\r").ok();
        }

        if sort_by_size_in_bytes {
            aggregates.push((path.as_ref().to_owned(), num_bytes, num_errors));
        } else {
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

    if sort_by_size_in_bytes {
        aggregates.sort_by_key(|&(_, num_bytes, _)| num_bytes);
        for (path, num_bytes, num_errors) in aggregates.into_iter() {
            output_colored_path(
                &mut out,
                &path,
                num_bytes,
                num_errors,
                path_color_of(&path),
                byte_format,
            )?;
        }
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

    let errors = (num_errors != 0)
        .then(|| {
            let plural_s = if num_errors > 1 { "s" } else { "" };
            format!("  <{num_errors} IO Error{plural_s}>")
        })
        .unwrap_or_default();

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
