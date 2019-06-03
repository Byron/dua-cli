use crate::interactive::{widgets::SortMode, EntryData, Tree, TreeIndex};
use itertools::Itertools;
use jwalk::WalkDir;
use petgraph::Direction;
use std::{fmt, path::Path};

pub(crate) fn get_entry_or_panic(tree: &Tree, node_idx: TreeIndex) -> &EntryData {
    tree.node_weight(node_idx)
        .expect("node should always be retrievable with valid index")
}

pub(crate) fn get_size_or_panic(tree: &Tree, node_idx: TreeIndex) -> u64 {
    get_entry_or_panic(tree, node_idx).size
}

pub(crate) fn sorted_entries(
    tree: &Tree,
    node_idx: TreeIndex,
    sorting: SortMode,
) -> Vec<(TreeIndex, &EntryData)> {
    use SortMode::*;
    tree.neighbors_directed(node_idx, Direction::Outgoing)
        .filter_map(|idx| tree.node_weight(idx).map(|w| (idx, w)))
        .sorted_by(|(_, l), (_, r)| match sorting {
            SizeDescending => r.size.cmp(&l.size),
            SizeAscending => l.size.cmp(&r.size),
        })
        .collect()
}

/// Specifies a way to format bytes
#[derive(Clone, Copy)]
pub enum ByteFormat {
    /// metric format, based on 1000.
    Metric,
    /// binary format, based on 1024
    Binary,
    /// raw bytes, without additional formatting
    Bytes,
}

impl ByteFormat {
    pub fn display(&self, bytes: u64) -> ByteFormatDisplay {
        ByteFormatDisplay {
            format: *self,
            bytes,
        }
    }
}

pub struct ByteFormatDisplay {
    format: ByteFormat,
    bytes: u64,
}

impl fmt::Display for ByteFormatDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use byte_unit::Byte;
        use ByteFormat::*;

        let binary = match self.format {
            Bytes => return write!(f, "{} b", self.bytes),
            Binary => true,
            Metric => false,
        };
        let b = Byte::from_bytes(self.bytes as u128)
            .get_appropriate_unit(binary)
            .format(2);
        let mut splits = b.split(' ');
        match (splits.next(), splits.next()) {
            (Some(bytes), Some(unit)) => write!(
                f,
                "{:>8} {:>unit_width$}",
                bytes,
                unit,
                unit_width = match self.format {
                    Binary => 3,
                    Metric => 2,
                    _ => 2,
                }
            ),
            _ => f.write_str(&b),
        }
    }
}

/// Identify the kind of sorting to apply during filesystem iteration
#[derive(Clone)]
pub enum TraversalSorting {
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
#[derive(Clone)]
pub struct WalkOptions {
    /// The amount of threads to use. Refer to [`WalkDir::num_threads()`](https://docs.rs/jwalk/0.4.0/jwalk/struct.WalkDir.html#method.num_threads)
    /// for more information.
    pub threads: usize,
    pub byte_format: ByteFormat,
    pub color: Color,
    pub sorting: TraversalSorting,
}

impl WalkOptions {
    pub(crate) fn iter_from_path(&self, path: &Path) -> WalkDir {
        WalkDir::new(path)
            .preload_metadata(true)
            .sort(match self.sorting {
                TraversalSorting::None => false,
                TraversalSorting::AlphabeticalByFileName => true,
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
