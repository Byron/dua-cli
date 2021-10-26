#![forbid(unsafe_code)]
use anyhow::Result;
use clap::Clap;
use dua::{ByteFormat, TraversalSorting};
use std::{fs, io, io::Write, path::PathBuf, process};

mod crossdev;
#[cfg(any(feature = "tui-unix", feature = "tui-crossplatform"))]
mod interactive;
mod options;

fn stderr_if_tty() -> Option<io::Stderr> {
    if atty::is(atty::Stream::Stderr) {
        Some(io::stderr())
    } else {
        None
    }
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64"),))]
fn derive_default_threads(threads: usize) -> usize {
    threads
}

/// On Apple Silicon, high-efficiency cores make file accesses slower over all, so avoid over committing for
/// this one.
///
/// On macos with apple silicon, the IO subsystem is entirely different and one thread can mostly max it out.
/// Thus using more threads just burns energy unnecessarily.
/// It's notable that `du` is very fast even on a single core and more power efficient than dua with a single core.
/// The default of '4' seems related to the amount of performance cores present in the system.
/// On everything else, it's usually a good idea to use as many threads as possible for noticeable speedups.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn derive_default_threads(threads: usize) -> usize {
    use sysinfo::{ProcessorExt, RefreshKind, SystemExt};
    if threads == 0 {
        sysinfo::System::new_with_specifics(RefreshKind::new().with_cpu())
            .processors()
            .get(0)
            .map_or(0, |p| match p.brand() {
                "Apple M1"|"Apple M1 Pro"|"Apple M1 Max" => 4,
                other => {
                    eprintln!(
                        "Couldn't auto-configure correct amount of threads for {}. Create an issue here: https://github.com/byron/dua-cli/issues",
                        other
                    );
                    0
                }
            })
    } else {
        threads
    }
}

fn main() -> Result<()> {
    use options::Command::*;

    let opt: options::Args = options::Args::parse_from(wild::args_os());
    let threads = derive_default_threads(opt.threads);
    let walk_options = dua::WalkOptions {
        threads,
        byte_format: opt.format.map(Into::into).unwrap_or(ByteFormat::Metric),
        apparent_size: opt.apparent_size,
        count_hard_links: opt.count_hard_links,
        sorting: TraversalSorting::None,
        cross_filesystems: !opt.stay_on_filesystem,
    };
    let res = match opt.command {
        #[cfg(any(feature = "tui-unix", feature = "tui-crossplatform"))]
        Some(Interactive { input }) => {
            use crate::interactive::{Interaction, TerminalApp};
            use anyhow::{anyhow, Context};
            use crosstermion::terminal::{tui::new_terminal, AlternateRawScreen};

            let no_tty_msg = "Interactive mode requires a connected terminal";
            if atty::isnt(atty::Stream::Stderr) {
                return Err(anyhow!(no_tty_msg));
            }

            let mut terminal = new_terminal(
                AlternateRawScreen::try_from(io::stderr()).with_context(|| no_tty_msg)?,
            )
            .with_context(|| "Could not instantiate terminal")?;
            let res = TerminalApp::initialize(
                &mut terminal,
                walk_options,
                paths_from(input, !opt.stay_on_filesystem)?,
                Interaction::Full,
            )?
            .map(|(keys_rx, mut app)| {
                let res = app.process_events(&mut terminal, keys_rx.into_iter());

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
                res
            });

            drop(terminal);
            io::stderr().flush().ok();

            // Exit 'quickly' to avoid having to not have to deal with slightly different types in the other match branches
            std::process::exit(
                res.transpose()?
                    .map(|(walk_result, paths)| {
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
                paths_from(input, !opt.stay_on_filesystem)?,
            )?;
            if statistics {
                writeln!(io::stderr(), "{:?}", stats).ok();
            }
            res
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
                paths_from(opt.input, !opt.stay_on_filesystem)?,
            )?
            .0
        }
    };

    process::exit(res.to_exit_code());
}

fn paths_from(paths: Vec<PathBuf>, cross_filesystems: bool) -> Result<Vec<PathBuf>, io::Error> {
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
            if let Ok(meta) = p.symlink_metadata() {
                if meta.file_type().is_symlink() {
                    return false;
                }
            };
            true
        })
        .collect();
    v.sort();
    Ok(v)
}
