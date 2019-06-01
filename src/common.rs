use jwalk::WalkDir;
use std::{fmt, path::Path};

pub enum ByteFormat {
    Metric,
    Binary,
    Bytes,
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
    pub fn iter_from_path(&self, path: &Path) -> WalkDir {
        WalkDir::new(path)
            .preload_metadata(true)
            .skip_hidden(false)
            .num_threads(self.threads)
    }
}

#[derive(Default)]
pub struct WalkResult {
    pub num_errors: u64,
}

impl fmt::Display for WalkResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Encountered {} IO error(s)", self.num_errors)
    }
}
