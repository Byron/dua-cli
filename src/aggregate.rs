use crate::inodefilter::InodeFilter;
use crate::{crossdev, file_size_on_disk, FlowControl, Throttle, WalkOptions, WalkResult};
use anyhow::Result;
use owo_colors::{AnsiColors as Color, OwoColorize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{io, path::Path};

/// Aggregate the given `paths` and write information about them to `out` in a human-readable format.
/// If `compute_total` is set, it will write an additional line with the total size across all given `paths`.
/// If `sort_by_size_in_bytes` is set, we will sort all sizes (ascending) before outputting them.
pub fn aggregate(
    mut out: impl io::Write,
    progress_to_stderr: bool,
    walk_options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<(WalkResult, Statistics)> {
    let mut res = WalkResult::default();
    let mut stats = Statistics {
        smallest_file_in_bytes: u64::max_value(),
        ..Default::default()
    };
    let mut total = 0;
    let mut num_roots = 0;
    let mut aggregates = Vec::new();
    let inodes = Arc::new(InodeFilter::default());
    let progress = Throttle::new(Duration::from_millis(100), Duration::from_secs(1).into());

    for path in paths.into_iter() {
        num_roots += 1;
        let num_bytes = AtomicU64::default();
        let entries_traversed = AtomicU64::default();
        let largest_file_in_bytes = AtomicU64::new(0);
        let smallest_file_in_bytes = AtomicU64::new(u64::MAX);
        let (device_id, meta) = match crossdev::init(path.as_ref()) {
            Ok(id) => id,
            Err(_) => {
                res.num_errors += 1;
                aggregates.push((
                    path.as_ref().to_owned(),
                    num_bytes.load(Ordering::Relaxed),
                    1,
                ));
                continue;
            }
        };
        let num_errors = AtomicU64::default();
        let count_size = {
            let inodes = inodes.clone();
            let apparent_size = walk_options.apparent_size;
            let count_hard_links = walk_options.count_hard_links;
            let smallest_file_in_bytes = &smallest_file_in_bytes;
            let largest_file_in_bytes = &largest_file_in_bytes;
            let entries_traversed = &entries_traversed;
            let num_bytes = &num_bytes;
            move |meta: &moonwalk::Metadata| {
                entries_traversed.fetch_add(1, Ordering::SeqCst);
                let file_size = {
                    if count_hard_links || inodes.is_first_moonwalk(meta) {
                        if apparent_size {
                            meta.len()
                        } else {
                            file_size_on_disk(meta)
                        }
                    } else {
                        0
                    }
                };

                largest_file_in_bytes.fetch_max(file_size, Ordering::SeqCst);
                smallest_file_in_bytes.fetch_min(file_size, Ordering::SeqCst);
                num_bytes.fetch_add(file_size, Ordering::SeqCst);
            }
        };
        if meta.is_dir() {
            walk_options.moonwalk_from_path(
                path.as_ref(),
                device_id,
                {
                    let progress = &progress;
                    let entries_traversed = &entries_traversed;
                    let num_errors = &num_errors;
                    move |entry, _depth| {
                        match entry {
                            Ok(entry) => {
                                if let Ok(meta) = entry.metadata() {
                                    count_size(meta);
                                }

                                progress.throttled(|| {
                                    if progress_to_stderr {
                                        eprint!(
                                            "Enumerating {} entries\r",
                                            entries_traversed.load(Ordering::Relaxed)
                                        );
                                    }
                                });
                            }
                            Err(_err) => {
                                num_errors.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                        FlowControl::Continue
                    }
                },
                false,
            )?;
        } else {
            count_size(&meta.into());
        }
        stats.entries_traversed = entries_traversed.load(Ordering::Relaxed);
        stats.largest_file_in_bytes = largest_file_in_bytes.load(Ordering::Relaxed);
        stats.smallest_file_in_bytes = smallest_file_in_bytes.load(Ordering::Relaxed);

        if progress_to_stderr {
            eprint!("\x1b[2K\r");
        }

        let num_errors = num_errors.load(Ordering::Relaxed);
        if sort_by_size_in_bytes {
            aggregates.push((
                path.as_ref().to_owned(),
                num_bytes.load(Ordering::Relaxed),
                num_errors,
            ));
        } else {
            output_colored_path(
                &mut out,
                &walk_options,
                &path,
                num_bytes.load(Ordering::Relaxed),
                num_errors,
                path_color_of(&path),
            )?;
        }
        total += num_bytes.load(Ordering::Relaxed);
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
                &walk_options,
                &path,
                num_bytes,
                num_errors,
                path_color_of(&path),
            )?;
        }
    }

    if num_roots > 1 && compute_total {
        output_colored_path(
            &mut out,
            &walk_options,
            Path::new("total"),
            total,
            res.num_errors,
            None,
        )?;
    }
    Ok((res, stats))
}

fn path_color_of(path: impl AsRef<Path>) -> Option<Color> {
    (!path.as_ref().is_file()).then_some(Color::Cyan)
}

fn output_colored_path(
    out: &mut impl io::Write,
    options: &WalkOptions,
    path: impl AsRef<Path>,
    num_bytes: u64,
    num_errors: u64,
    path_color: Option<Color>,
) -> std::result::Result<(), io::Error> {
    let size = options.byte_format.display(num_bytes).to_string();
    let size = size.green();
    let size_width = options.byte_format.width();
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
    pub smallest_file_in_bytes: u64,
    /// The size of the largest file encountered in bytes
    pub largest_file_in_bytes: u64,
}
