use clap_complete::Shell;
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

fn dft_format() -> ByteFormat {
    if cfg!(target_vendor = "apple") {
        ByteFormat::Metric
    } else {
        ByteFormat::Binary
    }
}

/// For some reason, on MacOS, too many threads are bad and 3 is the best these days on M4.
/// On M1 it was more like 4, but close enough.
#[cfg(target_os = "macos")]
const DEFAULT_THREADS: usize = 3;

#[cfg(not(target_os = "macos"))]
const DEFAULT_THREADS: usize = 0;

/// A tool to learn about disk usage, fast!
#[derive(Debug, clap::Parser)]
#[clap(name = "dua", version)]
#[clap(override_usage = "dua [FLAGS] [OPTIONS] [SUBCOMMAND] [INPUT]...")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The amount of threads to use. Defaults to 0, indicating the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[clap(short = 't', long = "threads", default_value_t = DEFAULT_THREADS, global = true, env = "DUA_THREADS")]
    pub threads: usize,

    /// The format with which to print byte counts.
    #[clap(
        short = 'f',
        long,
        value_enum,
        default_value_t = dft_format(),
        ignore_case = true,
        global = true,
        env = "DUA_FORMAT",
    )]
    pub format: ByteFormat,

    /// Display apparent size instead of disk usage.
    #[clap(short = 'A', long, global = true, env = "DUA_APPARENT_SIZE")]
    pub apparent_size: bool,

    /// Count hard-linked files each time they are seen
    #[clap(short = 'l', long, global = true, env = "DUA_COUNT_HARD_LINKS")]
    pub count_hard_links: bool,

    /// If set, we will not cross filesystems or traverse mount points
    #[clap(short = 'x', long, global = true, env = "DUA_STAY_ON_FILESYSTEM")]
    pub stay_on_filesystem: bool,

    /// One or more absolute directories to ignore. Note that these are not ignored if they are passed as input path.
    ///
    /// Hence, they will only be ignored if they are eventually reached as part of the traversal.
    #[clap(long = "ignore-dirs", short = 'i', value_parser, global = true, env = "DUA_IGNORE_DIRS")]
    #[cfg_attr(target_os = "linux", clap(default_values = &["/proc", "/dev", "/sys", "/run"]))]
    pub ignore_dirs: Vec<PathBuf>,

    /// One or more input files or directories. If unset, we will use all entries in the current working directory.
    #[clap(value_parser, global = true)]
    pub input: Vec<PathBuf>,

    /// Write a log file with debug information, including panics.
    #[clap(long, global = true, env = "DUA_LOG_FILE")]
    pub log_file: Option<PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Launch the terminal user interface
    #[cfg(feature = "tui-crossplatform")]
    #[clap(name = "interactive", visible_alias = "i")]
    Interactive {
        /// Do not check entries for presence when listing a directory to avoid slugging performance on slow filesystems.
        #[clap(long, short = 'e')]
        no_entry_check: bool,
    },
    /// Aggregate the consumed space of one or more directories or files
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
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate a completions-script for
        shell: Shell,
    },
}
