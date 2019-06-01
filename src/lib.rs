extern crate failure;
extern crate jwalk;

pub struct WalkOptions {
    pub threads: usize,
}

impl WalkOptions {
    pub fn iter_from_path(&self, path: &Path) -> WalkDir {
        WalkDir::new(path)
            .preload_metadata(true)
            .skip_hidden(false)
            .num_threads(self.threads)
    }
}

#[derive(Default)]
pub struct WalkResult {
    pub num_errors: usize,
}

impl fmt::Display for WalkResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Encountered {} IO errors", self.num_errors)
    }
}

mod aggregate {
    use crate::{WalkOptions, WalkResult};
    use failure::Error;
    use std::{io, path::Path};

    pub fn aggregate(
        mut out: impl io::Write,
        options: WalkOptions,
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> Result<WalkResult, Error> {
        let mut res = WalkResult::default();
        for path in paths.into_iter() {
            let mut num_bytes = 0u64;
            for entry in options.iter_from_path(path.as_ref()) {
                match entry {
                    Ok(entry) => {
                        num_bytes += match entry.metadata {
                            Some(Ok(m)) => m.len(),
                            Some(Err(_)) => {
                                res.num_errors += 1;
                                0
                            }
                            None => unreachable!(
                                "we ask for metadata, so we at least have Some(Err(..)))"
                            ),
                        };
                    }
                    Err(_) => res.num_errors += 1,
                }
            }

            writeln!(out, "{}\t{}", num_bytes, path.as_ref().display())?;
        }
        Ok(res)
    }
}

pub use aggregate::aggregate;
use jwalk::WalkDir;
use std::{fmt, path::Path};
