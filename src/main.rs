extern crate failure;
extern crate failure_tools;
extern crate structopt;

use crate::interactive::TerminalApp;
use dua::{ByteFormat, Color, TraversalSorting};
use failure::{Error, ResultExt};
use failure_tools::ok_or_exit;
use std::{fs, io, io::Write, path::PathBuf, process};
use structopt::StructOpt;
use termion::{input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui_react::Terminal;

mod interactive;
mod options;

fn run() -> Result<(), Error> {
    use options::Command::*;

    let opt: options::Args = options::Args::from_args();
    let stdout = io::stdout();
    let stdout_locked = stdout.lock();
    let walk_options = dua::WalkOptions {
        threads: opt.threads.unwrap_or(0),
        byte_format: opt.format.map(Into::into).unwrap_or(ByteFormat::Metric),
        color: if atty::is(atty::Stream::Stdout) {
            Color::Terminal
        } else {
            Color::None
        },
        sorting: TraversalSorting::None,
    };
    let res = match opt.command {
        Some(Interactive { input }) => {
            let mut terminal = {
                let stdout = io::stdout().into_raw_mode()?;
                let stdout = AlternateScreen::from(stdout);
                let backend = TermionBackend::new(stdout);
                Terminal::new(backend)
                    .with_context(|_| "Interactive mode requires a connected terminal")?
            };
            let mut app = TerminalApp::initialize(&mut terminal, walk_options, paths_from(input)?)?;
            app.process_events(&mut terminal, termion::async_stdin().keys())?
        }
        Some(Aggregate {
            input,
            no_total,
            no_sort,
            statistics,
        }) => {
            let (res, stats) = dua::aggregate(
                stdout_locked,
                walk_options,
                !no_total,
                !no_sort,
                paths_from(input)?,
            )?;
            if statistics {
                writeln!(io::stderr(), "{:?}", stats).ok();
            }
            res
        }
        None => {
            dua::aggregate(
                stdout_locked,
                walk_options,
                true,
                true,
                paths_from(opt.input)?,
            )?
            .0
        }
    };

    if res.num_errors > 0 {
        process::exit(1);
    }
    Ok(())
}

fn paths_from(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, io::Error> {
    if paths.is_empty() {
        cwd_dirlist()
    } else {
        Ok(paths)
    }
}

fn cwd_dirlist() -> Result<Vec<PathBuf>, io::Error> {
    let mut v: Vec<_> = fs::read_dir(".")?
        .filter_map(|e| {
            e.ok()
                .and_then(|e| e.path().strip_prefix(".").ok().map(ToOwned::to_owned))
        })
        .collect();
    v.sort();
    Ok(v)
}

fn main() {
    ok_or_exit(run())
}
