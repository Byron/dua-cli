extern crate failure;
extern crate failure_tools;
extern crate structopt;

use structopt::StructOpt;

use dua::ByteFormat;
use failure::Error;
use failure_tools::ok_or_exit;
use std::{io, path::PathBuf, process};

mod options;

fn run() -> Result<(), Error> {
    use options::Command::*;

    let opt: options::Args = options::Args::from_args();
    let stdout = io::stdout();
    let stdout_locked = stdout.lock();
    let walk_options = dua::WalkOptions {
        threads: opt.threads.unwrap_or(0),
        format: opt.format.map(Into::into).unwrap_or(ByteFormat::Metric),
    };
    let res = match opt.command {
        Some(Aggregate {
            input,
            no_total,
            no_sort,
        }) => dua::aggregate(stdout_locked, walk_options, !no_total, !no_sort, input),
        None => dua::aggregate(
            stdout_locked,
            walk_options,
            true,
            true,
            if opt.input.len() == 0 {
                vec![PathBuf::from(".")]
            } else {
                opt.input
            },
        ),
    }?;

    if res.num_errors > 0 {
        process::exit(1);
    }
    Ok(())
}

fn main() {
    ok_or_exit(run())
}
