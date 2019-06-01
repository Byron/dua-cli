use dua::ByteFormat as LibraryByteFormat;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ByteFormat {
        HumanMetric,
        HumanBinary,
        Bytes
    }
}

impl From<ByteFormat> for LibraryByteFormat {
    fn from(input: ByteFormat) -> Self {
        match input {
            ByteFormat::HumanMetric => LibraryByteFormat::Metric,
            ByteFormat::HumanBinary => LibraryByteFormat::Binary,
            ByteFormat::Bytes => LibraryByteFormat::Bytes,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dua", about = "A tool to learn about disk usage, fast!")]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Option<Command>,

    /// The amount of threads to use. Defaults to the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[structopt(short = "t", long = "threads")]
    pub threads: Option<usize>,

    /// The format with which to print byte counts.
    /// HumanMetric - uses 1000 as base (default)
    /// HumanBinary - uses 1024 as base
    /// Bytes - plain bytes without any formatting
    #[structopt(short = "f", long = "format")]
    pub format: Option<ByteFormat>,

    /// One or more input files. If unset, we will assume the current directory
    #[structopt(parse(from_os_str))]
    pub input: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Aggregrate the consumed space of one or more directories or files
    #[structopt(name = "aggregate", alias = "a")]
    Aggregate {
        /// If set, paths will be printed in their order of occurrence on the command-line.
        /// Otherwise they are sorted by their size in bytes, ascending.
        #[structopt(long = "no-sort")]
        no_sort: bool,
        /// If set, no total column will be computed for multiple inputs
        #[structopt(long = "no-total")]
        no_total: bool,
        /// One or more input files. If unset, we will assume the current directory
        #[structopt(parse(from_os_str))]
        input: Vec<PathBuf>,
    },
}
