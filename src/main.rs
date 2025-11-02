#![forbid(rust_2018_idioms, unsafe_code)]
use anyhow::Result;
use clap::{CommandFactory as _, Parser};
use dua::{TraversalSorting, canonicalize_ignore_dirs};
use log::info;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::OpenOptions;
use std::{fs, io, io::Write, path::PathBuf, process};

#[cfg(feature = "tui-crossplatform")]
use crate::interactive::input::input_channel;
#[cfg(feature = "tui-crossplatform")]
use crate::interactive::terminal::TerminalApp;

mod crossdev;
#[cfg(feature = "tui-crossplatform")]
mod interactive;
mod options;

fn stderr_if_tty() -> Option<io::Stderr> {
    if atty::is(atty::Stream::Stderr) {
        Some(io::stderr())
    } else {
        None
    }
}

fn main() -> Result<()> {
    use options::Command::*;

    let opt: options::Args = options::Args::parse_from(wild::args_os());

    if let Some(log_file) = &opt.log_file {
        log_panics::init();
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)?,
        )?;
        info!("dua options={opt:#?}");
    }

    let byte_format: dua::ByteFormat = opt.format.into();
    let mut walk_options = dua::WalkOptions {
        threads: opt.threads,
        apparent_size: opt.apparent_size,
        count_hard_links: opt.count_hard_links,
        sorting: TraversalSorting::None,
        cross_filesystems: !opt.stay_on_filesystem,
        ignore_dirs: canonicalize_ignore_dirs(&opt.ignore_dirs),
    };

    if walk_options.threads == 0 {
        // avoid using the global rayon pool, as it will keep a lot of threads alive after we are done.
        // Also means that we will spin up a bunch of threads per root path, instead of reusing them.
        walk_options.threads = num_cpus::get();
    }

    let res = match opt.command {
        #[cfg(feature = "tui-crossplatform")]
        Some(Interactive {
            no_entry_check,
            input,
        }) => {
            use anyhow::{Context, anyhow};
            use crosstermion::terminal::{AlternateRawScreen, tui::new_terminal};

            let no_tty_msg = "Interactive mode requires a connected terminal";
            if atty::isnt(atty::Stream::Stderr) {
                return Err(anyhow!(no_tty_msg));
            }

            let mut terminal = new_terminal(
                AlternateRawScreen::try_from(io::stderr()).with_context(|| no_tty_msg)?,
            )
            .with_context(|| "Could not instantiate terminal")?;

            let keys_rx = input_channel();
            let mut app = TerminalApp::initialize(
                &mut terminal,
                walk_options,
                byte_format,
                !no_entry_check,
                extract_paths_maybe_set_cwd(input, !opt.stay_on_filesystem)?,
            )?;
            app.traverse()?;

            let res = app.process_events(&mut terminal, keys_rx);

            let res = res.map(|r| {
                (
                    r,
                    app.window
                        .mark_pane
                        .take()
                        .map(|marked| marked.into_paths()),
                )
            });
            // Leak app memory to avoid having to wait for the hashmap to deallocate,
            // which causes a noticeable delay shortly before the the program exits anyway.
            std::mem::forget(app);

            drop(terminal);
            io::stderr().flush().ok();

            // Exit 'quickly' to avoid having to not have to deal with slightly different types in the other match branches
            std::process::exit(
                res.map(|(walk_result, paths)| {
                    if let Some(paths) = paths {
                        for path in paths {
                            println!("{}", path.display())
                        }
                    }
                    walk_result.to_exit_code()
                })
                .unwrap_or(0),
            );
        }
        Some(Aggregate {
            input,
            no_total,
            no_sort,
            statistics,
        }) => {
            let stdout = io::stdout();
            let stdout_locked = stdout.lock();
            let (res, stats) = dua::aggregate(
                stdout_locked,
                stderr_if_tty(),
                walk_options,
                !no_total,
                !no_sort,
                byte_format,
                extract_paths_maybe_set_cwd(input, !opt.stay_on_filesystem)?,
            )?;
            if statistics {
                writeln!(io::stderr(), "{stats:?}").ok();
            }
            res
        }
        Some(Completions { shell }) => {
            let mut cmd = options::Args::command();
            let dua = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, dua, &mut io::stdout());
            return Ok(());
        }
        None => {
            let stdout = io::stdout();
            let stdout_locked = stdout.lock();
            dua::aggregate(
                stdout_locked,
                stderr_if_tty(),
                walk_options,
                true,
                true,
                byte_format,
                extract_paths_maybe_set_cwd(opt.input, !opt.stay_on_filesystem)?,
            )?
            .0
        }
    };

    process::exit(res.to_exit_code());
}

fn extract_paths_maybe_set_cwd(
    mut paths: Vec<PathBuf>,
    cross_filesystems: bool,
) -> Result<Vec<PathBuf>, io::Error> {
    if paths.len() == 1 && paths[0].is_dir() {
        std::env::set_current_dir(&paths[0])?;
        paths.clear();
    }
    let device_id = std::env::current_dir()
        .ok()
        .and_then(|cwd| crossdev::init(&cwd).ok());

    if paths.is_empty() {
        cwd_dirlist().map(|paths| match device_id {
            Some(device_id) if !cross_filesystems => paths
                .into_iter()
                .filter(|p| match p.metadata() {
                    Ok(meta) => crossdev::is_same_device(device_id, &meta),
                    Err(_) => true,
                })
                .collect(),
            _ => paths,
        })
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
        .filter(|p| {
            if let Ok(meta) = p.symlink_metadata()
                && meta.file_type().is_symlink()
            {
                return false;
            };
            true
        })
        .collect();
    v.sort();
    Ok(v)
}
