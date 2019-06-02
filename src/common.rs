use jwalk::WalkDir;
use std::fmt;
use std::path::Path;

/// Specifies a way to format bytes
pub enum ByteFormat {
    /// metric format, based on 1000.
    Metric,
    /// binary format, based on 1024
    Binary,
    /// raw bytes, without additional formatting
    Bytes,
}

/// Identify the kind of sorting to apply during filesystem iteration
pub enum Sorting {
    None,
    AlphabeticalByFileName,
}

/// Specify the kind of color to use
#[derive(Clone, Copy)]
pub enum Color {
    /// Use no color
    None,
    /// Use terminal colors
    Terminal,
}

pub(crate) struct DisplayColor<C> {
    kind: Color,
    color: C,
}

impl Color {
    pub(crate) fn display<C>(&self, color: C) -> DisplayColor<C> {
        DisplayColor { kind: *self, color }
    }
}

impl<C> fmt::Display for DisplayColor<C>
where
    C: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.kind {
            Color::None => Ok(()),
            Color::Terminal => self.color.fmt(f),
        }
    }
}

/// Configures a filesystem walk, including output and formatting options.
pub struct WalkOptions {
    /// The amount of threads to use. Refer to [`WalkDir::num_threads()`](https://docs.rs/jwalk/0.4.0/jwalk/struct.WalkDir.html#method.num_threads)
    /// for more information.
    pub threads: usize,
    pub byte_format: ByteFormat,
    pub color: Color,
    pub sorting: Sorting,
}

impl WalkOptions {
    pub(crate) fn format_bytes(&self, b: u64) -> String {
        use byte_unit::Byte;
        use ByteFormat::*;
        let binary = match self.byte_format {
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
                unit_width = match self.byte_format {
                    Binary => 3,
                    Metric => 2,
                    _ => 2,
                }
            ),
            _ => b,
        }
    }

    pub(crate) fn iter_from_path(&self, path: &Path) -> WalkDir {
        WalkDir::new(path)
            .preload_metadata(true)
            .sort(match self.sorting {
                Sorting::None => false,
                Sorting::AlphabeticalByFileName => true,
            })
            .skip_hidden(false)
            .num_threads(self.threads)
    }
}

/// Information we gather during a filesystem walk
#[derive(Default)]
pub struct WalkResult {
    /// The amount of io::errors we encountered. Can happen when fetching meta-data, or when reading the directory contents.
    pub num_errors: u64,
}
