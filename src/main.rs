extern crate failure;
extern crate failure_tools;
#[macro_use]
extern crate structopt;

use failure::Error;
use failure_tools::ok_or_exit;
use std::{io, path::PathBuf};
use structopt::StructOpt;

mod options {
    use std::path::PathBuf;

    #[derive(Debug, StructOpt)]
    #[structopt(name = "dua", about = "A tool to learn about disk usage, fast!")]
    pub struct Args {
        #[structopt(subcommand)]
        pub command: Option<Command>,

        /// The amount of threads to use. Defaults to the amount of logical processors.
        /// Set to 1 to use only a single thread.
        #[structopt(short = "t", long = "threads")]
        pub threads: Option<usize>,
    }

    #[derive(Debug, StructOpt)]
    pub enum Command {
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
    };
    let res = match opt.command {
        Some(Aggregate { input: _ }) => unimplemented!(),
        None => dua::aggregate(stdout_locked, walk_options, vec![PathBuf::from(".")]),
    }?;

    if res.num_errors > 0 {
        writeln!(io::stderr(), "{}", res).ok();
    }
    Ok(())
}

fn main() {
    ok_or_exit(run())
}
