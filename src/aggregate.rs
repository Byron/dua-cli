use crate::{WalkOptions, WalkResult};
use failure::Error;
use std::borrow::Cow;
use std::{io, path::Path};

pub fn aggregate(
    mut out: impl io::Write,
    options: WalkOptions,
    compute_total: bool,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<WalkResult, Error> {
    let mut res = WalkResult::default();
    let mut total = 0;
    let mut num_roots = 0;
    for path in paths.into_iter() {
        num_roots += 1;
        let mut num_bytes = 0u64;
        let mut num_errors = 0u64;
        for entry in options.iter_from_path(path.as_ref()) {
            match entry {
                Ok(entry) => {
                    num_bytes += match entry.metadata {
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
                }
                Err(_) => num_errors += 1,
            }
        }

        write_path(&mut out, &options, path, num_bytes, num_errors)?;
        total += num_bytes;
        res.num_errors += num_errors;
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
    writeln!(
        out,
        "{}\t{}{}",
        options.format_bytes(num_bytes),
        path.as_ref().display(),
        if num_errors == 0 {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!("\t<{} IO Error(s)>", num_errors))
        }
    )
}
