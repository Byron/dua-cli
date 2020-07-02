use std::path::PathBuf;

use argh::{FromArgValue, FromArgs};
use dua::ByteFormat;

pub enum CliByteFormat {
    Metric,
    Binary,
    Bytes,
    GB,
    GiB,
    MB,
    MiB,
}

impl FromArgValue for CliByteFormat {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        use CliByteFormat::*;
        let value_lc = value.to_ascii_lowercase();
        Ok(match value_lc.as_str() {
            "metric" => Metric,
            "binary" => Binary,
            "bytes" => Bytes,
            "gb" => GB,
            "gib" => GiB,
            "mb" => MB,
            "mib" => MiB,
            _ => return Err(format!("Invalid byte format: {}", value)),
        })
    }
}

impl From<CliByteFormat> for ByteFormat {
    fn from(input: CliByteFormat) -> Self {
        use CliByteFormat::*;
        match input {
            Metric => ByteFormat::Metric,
            Binary => ByteFormat::Binary,
            Bytes => ByteFormat::Bytes,
            GB => ByteFormat::GB,
            GiB => ByteFormat::GiB,
            MB => ByteFormat::MB,
            MiB => ByteFormat::MiB,
        }
    }
}

/// a tool to learn about disk usage, fast!
#[derive(FromArgs)]
#[argh(name = "dua")]
pub struct Args {
    #[argh(subcommand)]
    pub command: Option<Command>,

    /// the amount of threads to use. Defaults to the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[argh(option, short = 't')]
    pub threads: Option<usize>,

    /// the format with which to print byte counts.
    /// Metric - uses 1000 as base (default)
    /// Binary - uses 1024 as base
    /// Bytes - plain bytes without any formatting
    /// GB - only gigabytes
    /// GiB - only gibibytes
    /// MB - only megabytes
    /// MiB - only mebibytes
    #[argh(option, short = 'f')]
    pub format: Option<CliByteFormat>,

    /// display apparent size instead of disk usage.
    #[argh(switch, short = 'A')]
    pub apparent_size: bool,

    /// count hard-linked files each time they are seen
    #[argh(switch, short = 'l')]
    pub count_hard_links: bool,

    /// if set, we will not cross filesystems or traverse mount points
    #[argh(switch, short = 'x')]
    pub stay_on_filesystem: bool,

    /// one or more input files or directories. If unset, we will use all entries in the current working directory.
    #[argh(positional)]
    pub input: Vec<PathBuf>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Interactive(Interactive),
    InteractiveAlias(InteractiveAlias),
    Aggregate(Aggregate),
    AggregateAlias(AggregateAlias),
}

/// Launch the terminal user interface
#[derive(FromArgs)]
#[argh(subcommand, name = "interactive")]
pub struct Interactive {
    /// one or more input files or directories. If unset, we will use all entries in the current working directory.
    #[argh(positional)]
    pub input: Vec<PathBuf>,
}

/// Alias for 'interactive'
#[derive(FromArgs)]
#[argh(subcommand, name = "i")]
pub struct InteractiveAlias {
    #[argh(positional)]
    pub input: Vec<PathBuf>,
}

/// Aggregate the consumed space of one or more directories or files
#[derive(FromArgs)]
#[argh(subcommand, name = "aggregate")]
pub struct Aggregate {
    /// if set, print additional statistics about the file traversal to stderr
    #[argh(switch)]
    pub stats: bool,
    /// if set, paths will be printed in their order of occurrence on the command-line.
    /// Otherwise they are sorted by their size in bytes, ascending.
    #[argh(switch)]
    pub no_sort: bool,
    /// if set, no total column will be computed for multiple inputs
    #[argh(switch)]
    pub no_total: bool,
    /// one or more input files or directories. If unset, we will use all entries in the current working directory.
    #[argh(positional)]
    pub input: Vec<PathBuf>,
}

/// An alias for "aggregate"
#[derive(FromArgs)]
#[argh(subcommand, name = "a")]
pub struct AggregateAlias {
    /// see `dua aggregate --help`
    #[argh(switch)]
    pub stats: bool,
    /// see `dua aggregate --help`
    #[argh(switch)]
    pub no_sort: bool,
    /// see `dua aggregate --help`
    #[argh(switch)]
    pub no_total: bool,
    /// see `dua aggregate --help`
    #[argh(positional)]
    pub input: Vec<PathBuf>,
}
