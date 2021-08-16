use clap::Clap;
use dua::ByteFormat as LibraryByteFormat;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum ByteFormat {
    Metric,
    Binary,
    Bytes,
    GB,
    GiB,
    MB,
    MiB,
}

impl FromStr for ByteFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "metric" | "Metric" => ByteFormat::Metric,
            "binary" | "Binary" => ByteFormat::Binary,
            "bytes" | "Bytes" => ByteFormat::Bytes,
            "GB" | "Gb" | "gb" => ByteFormat::GB,
            "GiB" | "gib" => ByteFormat::GiB,
            "MB" | "Mb" | "mb" => ByteFormat::MB,
            "MiB" | "mib" => ByteFormat::MiB,
            _ => return Err(format!("Invalid byte format: {:?}", s)),
        })
    }
}

impl ByteFormat {
    const VARIANTS: &'static [&'static str] =
        &["metric", "binary", "bytes", "MB", "MiB", "GB", "GiB"];
}

impl From<ByteFormat> for LibraryByteFormat {
    fn from(input: ByteFormat) -> Self {
        match input {
            ByteFormat::Metric => LibraryByteFormat::Metric,
            ByteFormat::Binary => LibraryByteFormat::Binary,
            ByteFormat::Bytes => LibraryByteFormat::Bytes,
            ByteFormat::GB => LibraryByteFormat::GB,
            ByteFormat::GiB => LibraryByteFormat::GiB,
            ByteFormat::MB => LibraryByteFormat::MB,
            ByteFormat::MiB => LibraryByteFormat::MiB,
        }
    }
}

#[derive(Debug, Clap)]
#[clap(name = "dua", about = "A tool to learn about disk usage, fast!", version = clap::crate_version!())]
#[clap(setting = clap::AppSettings::ColoredHelp)]
#[clap(setting = clap::AppSettings::DisableVersionForSubcommands)]
#[clap(override_usage = "dua [FLAGS] [OPTIONS] [SUBCOMMAND] [input]...")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The amount of threads to use. Defaults to 0, indicating the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[clap(short = 't', long = "threads", default_value = "0")]
    pub threads: usize,

    /// The format with which to print byte counts.
    /// Metric - uses 1000 as base (default)
    /// Binary - uses 1024 as base
    /// Bytes - plain bytes without any formatting
    /// GB - only gigabytes
    /// GiB - only gibibytes
    /// MB - only megabytes
    /// MiB - only mebibytes
    #[clap(short = 'f', long, case_insensitive = true, possible_values(&ByteFormat::VARIANTS))]
    pub format: Option<ByteFormat>,

    /// Display apparent size instead of disk usage.
    #[clap(short = 'A', long)]
    pub apparent_size: bool,

    /// Count hard-linked files each time they are seen
    #[clap(short = 'l', long)]
    pub count_hard_links: bool,

    /// If set, we will not cross filesystems or traverse mount points
    #[clap(short = 'x', long)]
    pub stay_on_filesystem: bool,

    /// One or more input files or directories. If unset, we will use all entries in the current working directory.
    #[clap(parse(from_os_str))]
    pub input: Vec<PathBuf>,
}

#[derive(Debug, Clap)]
pub enum Command {
    /// Launch the terminal user interface
    #[cfg(any(feature = "tui-unix", feature = "tui-crossplatform"))]
    #[clap(name = "interactive", visible_alias = "i")]
    Interactive {
        /// One or more input files or directories. If unset, we will use all entries in the current working directory.
        #[clap(parse(from_os_str))]
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
        #[clap(parse(from_os_str))]
        input: Vec<PathBuf>,
    },
}
