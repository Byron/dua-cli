use dua::ByteFormat as LibraryByteFormat;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ByteFormat {
        Metric,
        Binary,
        Bytes,
        GB,
        GiB,
        MB,
        MiB
    }
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

#[derive(Debug, StructOpt)]
#[structopt(name = "dua", about = "A tool to learn about disk usage, fast!")]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Option<Command>,

    /// The amount of threads to use. Defaults to the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[structopt(short = "t", long = "threads")]
    pub threads: Option<usize>,

    /// The format with which to print byte counts.
    /// Metric - uses 1000 as base (default)
    /// Binary - uses 1024 as base
    /// Bytes - plain bytes without any formatting
    /// GB - only gigabytes
    /// GiB - only gibibytes
    /// MB - only megabytes
    /// MiB - only mebibytes
    #[structopt(short = "f", long = "format")]
    pub format: Option<ByteFormat>,

    /// Display apparent size instead of disk usage.
    #[structopt(short = "A", long = "apparent-size")]
    pub apparent_size: bool,

    /// Count hard-linked files each time they are seen
    #[structopt(short = "l", long = "count-links")]
    pub count_links: bool,

    /// One or more input files or directories. If unset, we will use all entries in the current working directory.
    #[structopt(parse(from_os_str))]
    pub input: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Launch the terminal user interface
    #[structopt(name = "interactive", alias = "i")]
    Interactive {
        /// One or more input files or directories. If unset, we will use all entries in the current working directory.
        #[structopt(parse(from_os_str))]
        input: Vec<PathBuf>,
    },
    /// Aggregrate the consumed space of one or more directories or files
    #[structopt(name = "aggregate", alias = "a")]
    Aggregate {
        /// If set, print additional statistics about the file traversal to stderr
        #[structopt(long = "stats")]
        statistics: bool,
        /// If set, paths will be printed in their order of occurrence on the command-line.
        /// Otherwise they are sorted by their size in bytes, ascending.
        #[structopt(long = "no-sort")]
        no_sort: bool,
        /// If set, no total column will be computed for multiple inputs
        #[structopt(long = "no-total")]
        no_total: bool,
        /// One or more input files or directories. If unset, we will use all entries in the current working directory.
        #[structopt(parse(from_os_str))]
        input: Vec<PathBuf>,
    },
}
