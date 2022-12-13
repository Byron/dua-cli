use dua::ByteFormat as LibraryByteFormat;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Debug, Clone, Copy, clap::ValueEnum)]
pub enum ByteFormat {
    Metric,
    Binary,
    Bytes,
    GB,
    Gib,
    MB,
    Mib,
}

impl From<ByteFormat> for LibraryByteFormat {
    fn from(input: ByteFormat) -> Self {
        match input {
            ByteFormat::Metric => LibraryByteFormat::Metric,
            ByteFormat::Binary => LibraryByteFormat::Binary,
            ByteFormat::Bytes => LibraryByteFormat::Bytes,
            ByteFormat::GB => LibraryByteFormat::GB,
            ByteFormat::Gib => LibraryByteFormat::GiB,
            ByteFormat::MB => LibraryByteFormat::MB,
            ByteFormat::Mib => LibraryByteFormat::MiB,
        }
    }
}

/// A tool to learn about disk usage, fast!
#[derive(Debug, clap::Parser)]
#[clap(name = "dua", version)]
#[clap(override_usage = "dua [FLAGS] [OPTIONS] [SUBCOMMAND] [INPUT]...")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The amount of threads to use. Defaults to 0, indicating the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[clap(short = 't', long = "threads", default_value_t = 0)]
    pub threads: usize,

    /// The format with which to print byte counts:
    /// metric - uses 1000 as base (default),
    /// binary - uses 1024 as base,
    /// bytes - plain bytes without any formatting,
    /// GB - only gigabytes,
    /// GiB - only gibibytes,
    /// MB - only megabytes,
    /// MiB - only mebibytes
    #[clap(
        short = 'f',
        long,
        value_enum,
        default_value_t = ByteFormat::Metric,
        ignore_case = true,
        hide_default_value = true,
        hide_possible_values = true
    )]
    pub format: ByteFormat,

    /// Display apparent size instead of disk usage.
    #[clap(short = 'A', long)]
    pub apparent_size: bool,

    /// Count hard-linked files each time they are seen
    #[clap(short = 'l', long)]
    pub count_hard_links: bool,

    /// If set, we will not cross filesystems or traverse mount points
    #[clap(short = 'x', long)]
    pub stay_on_filesystem: bool,

    /// One or more absolute directories to ignore. Note that these are not ignored if they are passed as input path.
    ///
    /// Hence, they will only be ignored if they are eventually reached as part of the traversal.
    #[clap(long = "ignore-dirs", short = 'i', value_parser)]
    #[cfg_attr(target_os = "linux", clap(default_values = &["/proc", "/dev", "/sys", "/run"]))]
    pub ignore_dirs: Vec<PathBuf>,

    /// One or more input files or directories. If unset, we will use all entries in the current working directory.
    #[clap(value_parser)]
    pub input: Vec<PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Launch the terminal user interface
    #[cfg(any(feature = "tui-unix", feature = "tui-crossplatform"))]
    #[clap(name = "interactive", visible_alias = "i")]
    Interactive {
        /// One or more input files or directories. If unset, we will use all entries in the current working directory.
        #[clap(value_parser)]
        input: Vec<PathBuf>,
    },
    /// Aggregrate the consumed space of one or more directories or files
    #[clap(name = "aggregate", visible_alias = "a")]
    Aggregate {
        /// If set, print additional statistics about the file traversal to stderr
        #[clap(long = "stats")]
        statistics: bool,
        /// If set, paths will be printed in their order of occurrence on the command-line.
        /// Otherwise they are sorted by their size in bytes, ascending.
        #[clap(long)]
        no_sort: bool,
        /// If set, no total column will be computed for multiple inputs
        #[clap(long)]
        no_total: bool,
        /// One or more input files or directories. If unset, we will use all entries in the current working directory.
        #[clap(value_parser)]
        input: Vec<PathBuf>,
    },
}
