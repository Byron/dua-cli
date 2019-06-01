extern crate failure;
extern crate failure_tools;
extern crate structopt;

use structopt::StructOpt;

use dua::ByteFormat;
use failure::Error;
use failure_tools::ok_or_exit;
use std::{io, path::PathBuf};

mod options {
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
            /// One or more input files. If unset, we will assume the current directory
            #[structopt(parse(from_os_str))]
            input: Vec<PathBuf>,
        },
    }
}

fn run() -> Result<(), Error> {
    use io::Write;
    use options::Command::*;

    let opt: options::Args = options::Args::from_args();
    let stdout = io::stdout();
    let stdout_locked = stdout.lock();
    let walk_options = dua::WalkOptions {
        threads: opt.threads.unwrap_or(0),
        format: opt.format.map(Into::into).unwrap_or(ByteFormat::Metric),
    };
    let res = match opt.command {
        Some(Aggregate { input }) => dua::aggregate(stdout_locked, walk_options, input),
        None => dua::aggregate(
            stdout_locked,
            walk_options,
            if opt.input.len() == 0 {
                vec![PathBuf::from(".")]
            } else {
                opt.input
            },
        ),
    }?;

    if res.num_errors > 0 {
        writeln!(io::stderr(), "{}", res).ok();
    }
    Ok(())
}

fn main() {
    ok_or_exit(run())
}
