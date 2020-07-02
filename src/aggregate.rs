use crate::{crossdev, InodeFilter, WalkOptions, WalkResult};
use anyhow::Result;
use filesize::PathExt;
use std::borrow::Cow;
use std::{fmt, io, path::Path};
use termion::color;

/// Aggregate the given `paths` and write information about them to `out` in a human-readable format.
/// If `compute_total` is set, it will write an additional line with the total size across all given `paths`.
/// If `sort_by_size_in_bytes` is set, we will sort all sizes (ascending) before outputting them.
pub fn aggregate(
    mut out: impl io::Write,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<(WalkResult, Statistics)> {
    let mut res = WalkResult::default();
    let mut stats = Statistics::default();
    stats.smallest_file_in_bytes = u128::max_value();
    let mut total = 0;
    let mut num_roots = 0;
    let mut aggregates = Vec::new();
    let mut inodes = InodeFilter::default();
    let paths: Vec<_> = paths.into_iter().collect();
    for path in paths.into_iter() {
        num_roots += 1;
        let mut num_bytes = 0u128;
        let mut num_errors = 0u64;
        let device_id = crossdev::init(path.as_ref())?;
        for entry in walk_options.iter_from_path(path.as_ref()) {
            stats.entries_traversed += 1;
            match entry {
                Ok(entry) => {
                    let file_size = match entry.client_state {
                        Some(Ok(ref m))
                            if !m.is_dir()
                                && (walk_options.count_hard_links || inodes.add(m))
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
                        None => unreachable!("must have populated client state for metadata"),
                    } as u128;
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
            write_path(
                &mut out,
                &walk_options,
                &path,
                num_bytes,
                num_errors,
                path_color(&path),
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
            write_path(
                &mut out,
                &walk_options,
                &path,
                num_bytes,
                num_errors,
                path_color(&path),
            )?;
        }
    }

    if num_roots > 1 && compute_total {
        write_path(
            &mut out,
            &walk_options,
            Path::new("total"),
            total,
            res.num_errors,
            color::Fg(color::Reset),
        )?;
    }
    Ok((res, stats))
}

fn path_color(path: impl AsRef<Path>) -> Box<dyn fmt::Display> {
    if path.as_ref().is_file() {
        Box::new(color::Fg(color::LightBlack))
    } else {
        Box::new(color::Fg(color::Reset))
    }
}

fn write_path<C: fmt::Display>(
    out: &mut impl io::Write,
    options: &WalkOptions,
    path: impl AsRef<Path>,
    num_bytes: u128,
    num_errors: u64,
    path_color: C,
) -> std::result::Result<(), io::Error> {
    writeln!(
        out,
        "{byte_color}{:>byte_column_width$}{byte_color_reset} {path_color}{}{path_color_reset}{}",
        options.byte_format.display(num_bytes).to_string(), // needed for formatting to work (unless we implement it ourselves)
        path.as_ref().display(),
        if num_errors == 0 {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!(
                "  <{} IO Error{}>",
                num_errors,
                if num_errors > 1 { "s" } else { "" }
            ))
        },
        byte_color = options.color.display(color::Fg(color::Green)),
        byte_color_reset = options.color.display(color::Fg(color::Reset)),
        path_color = options.color.display(path_color),
        path_color_reset = options.color.display(color::Fg(color::Reset)),
        byte_column_width = options.byte_format.width()
    )
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
