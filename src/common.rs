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
        Byte::from_bytes(b as u128)
            .get_appropriate_unit(binary)
            .format(2)
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

#[derive(Default)]
pub struct WalkResult {
    pub num_errors: u64,
}
