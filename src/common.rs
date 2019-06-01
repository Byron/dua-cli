use jwalk::WalkDir;
use std::path::Path;

pub enum ByteFormat {
    Metric,
    Binary,
    Bytes,
}

pub enum Sorting {
    None,
    Alphabetical,
}

pub struct WalkOptions {
    pub threads: usize,
    pub format: ByteFormat,
}

impl WalkOptions {
    pub fn format_bytes(&self, b: u64) -> String {
        use byte_unit::Byte;
        use ByteFormat::*;
        let binary = match self.format {
            Bytes => return format!("{} b", b),
            Binary => true,
            Metric => false,
        };
        let b = Byte::from_bytes(b as u128)
            .get_appropriate_unit(binary)
            .format(2);
        let mut splits = b.split(' ');
        match (splits.next(), splits.next()) {
            (Some(bytes), Some(unit)) => format!(
                "{:>8} {:>unit_width$}",
                bytes,
                unit,
                unit_width = match self.format {
                    Binary => 3,
                    Metric => 2,
                    _ => 2,
                }
            ),
            _ => b,
        }
    }

    pub fn iter_from_path(&self, path: &Path, sort: Sorting) -> WalkDir {
        WalkDir::new(path)
            .preload_metadata(true)
            .sort(match sort {
                Sorting::Alphabetical => true,
                Sorting::None => false,
            })
            .skip_hidden(false)
            .num_threads(self.threads)
    }
}

#[derive(Default, Debug)]
pub struct Statistics {
    pub files_traversed: u64,
    pub smallest_file_in_bytes: u64,
    pub largest_file_in_bytes: u64,
}

#[derive(Default)]
pub struct WalkResult {
    pub num_errors: u64,
    pub stats: Statistics,
}
