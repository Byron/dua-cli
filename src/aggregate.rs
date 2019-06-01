use crate::{Sorting, WalkOptions, WalkResult};
use failure::Error;
use std::borrow::Cow;
use std::{io, path::Path};

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
            write_path(&mut out, &options, path, num_bytes, num_errors)?;
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
            write_path(&mut out, &options, path, num_bytes, num_errors)?;
        }
    }

    if num_roots > 1 && compute_total {
        write_path(
            &mut out,
            &options,
            Path::new("total"),
            total,
            res.num_errors,
        )?;
    }
    Ok(res)
}

fn write_path(
    out: &mut impl io::Write,
    options: &WalkOptions,
    path: impl AsRef<Path>,
    num_bytes: u64,
    num_errors: u64,
) -> Result<(), io::Error> {
    use termion::color;
    writeln!(
        out,
        "{}{:>10}{}\t{}{}",
        options.color.display(color::Fg(color::Green)),
        options.format_bytes(num_bytes),
        options.color.display(color::Fg(color::Reset)),
        path.as_ref().display(),
        if num_errors == 0 {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!("\t<{} IO Error(s)>", num_errors))
        }
    )
}
