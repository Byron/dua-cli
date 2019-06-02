use crate::{Sorting, WalkOptions, WalkResult};
use failure::Error;
use std::borrow::Cow;
use std::{fmt, io, path::Path};
use termion::color;

/// Aggregate the given `paths` and write information about them to `out` in a human-readable format.
/// If `compute_total` is set, it will write an additional line with the total size across all given `paths`.
/// If `sort_by_size_in_bytes` is set, we will sort all sizes (ascending) before outputting them.
pub fn aggregate(
    mut out: impl io::Write,
    options: WalkOptions,
    compute_total: bool,
    sort_by_size_in_bytes: bool,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<WalkResult, Error> {
    let mut res = WalkResult::default();
    res.stats.smallest_file_in_bytes = u64::max_value();
    let mut total = 0;
    let mut num_roots = 0;
    let mut aggregates = Vec::new();
    for path in paths.into_iter() {
        num_roots += 1;
        let mut num_bytes = 0u64;
        let mut num_errors = 0u64;
        for entry in options.iter_from_path(path.as_ref(), Sorting::None) {
            res.stats.files_traversed += 1;
            match entry {
                Ok(entry) => {
                    let file_size = match entry.metadata {
                        Some(Ok(ref m)) if !m.is_dir() => m.len(),
                        Some(Ok(_)) => 0,
                        Some(Err(_)) => {
                            num_errors += 1;
                            0
                        }
                        None => unreachable!(
                            "we ask for metadata, so we at least have Some(Err(..))). Issue in jwalk?"
                        ),
                    };
                    res.stats.largest_file_in_bytes =
                        res.stats.largest_file_in_bytes.max(file_size);
                    res.stats.smallest_file_in_bytes =
                        res.stats.smallest_file_in_bytes.min(file_size);
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
                &options,
                &path,
                num_bytes,
                num_errors,
                path_color(&path),
            )?;
        }
        total += num_bytes;
        res.num_errors += num_errors;
    }

    if res.stats.files_traversed == 0 {
        res.stats.smallest_file_in_bytes = 0;
    }

    if sort_by_size_in_bytes {
        aggregates.sort_by_key(|&(_, num_bytes, _)| num_bytes);
        for (path, num_bytes, num_errors) in aggregates.into_iter() {
            write_path(
                &mut out,
                &options,
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
            &options,
            Path::new("total"),
            total,
            res.num_errors,
            color::Fg(color::Reset),
        )?;
    }
    Ok(res)
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
    num_bytes: u64,
    num_errors: u64,
    path_color: C,
) -> Result<(), io::Error> {
    writeln!(
        out,
        "{byte_color}{:>10}{byte_color_reset}\t{path_color}{}{path_color_reset}{}",
        options.format_bytes(num_bytes),
        path.as_ref().display(),
        if num_errors == 0 {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!("\t<{} IO Error(s)>", num_errors))
        },
        byte_color = options.color.display(color::Fg(color::Green)),
        byte_color_reset = options.color.display(color::Fg(color::Reset)),
        path_color = options.color.display(path_color),
        path_color_reset = options.color.display(color::Fg(color::Reset)),
    )
}
