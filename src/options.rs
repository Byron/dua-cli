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
pub(crate) const DEFAULT_THREADS: usize = 3;

#[cfg(not(target_os = "macos"))]
pub(crate) const DEFAULT_THREADS: usize = 0;

#[cfg(target_os = "linux")]
pub(crate) const DEFAULT_IGNORE_DIRS: &[&str] = &["/proc", "/dev", "/sys", "/run"];

#[cfg(not(target_os = "linux"))]
pub(crate) const DEFAULT_IGNORE_DIRS: &[&str] = &[];

/// A tool to learn about disk usage, fast!
#[derive(Debug, clap::Parser)]
#[command(name = "dua", version, subcommand_precedence_over_arg = true)]
#[command(override_usage = "dua [FLAGS] [OPTIONS] [SUBCOMMAND] [INPUT]...")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    #[clap(flatten)]
    pub traversal: TraversalArgs,

    /// Write a log file with debug information, including panics.
    #[clap(long, global = true, env = "DUA_LOG_FILE")]
    pub log_file: Option<PathBuf>,
}

impl TraversalArgs {
    pub fn byte_format(&self, config: &dua::Config) -> LibraryByteFormat {
        self.format
            .map(LibraryByteFormat::from)
            .or(config.format)
            .unwrap_or_else(|| dft_format().into())
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct TraversalArgs {
    /// The amount of threads to use. Defaults to 0, indicating the amount of logical processors.
    /// Set to 1 to use only a single thread.
    #[clap(
        short = 't',
        long = "threads",
        default_value_t = DEFAULT_THREADS,
        env = "DUA_THREADS",
        help_heading = "Traversal Options"
    )]
    pub threads: usize,

    /// The format with which to print byte counts.
    #[clap(
        short = 'f',
        long,
        value_enum,
        ignore_case = true,
        env = "DUA_FORMAT",
        help_heading = "Traversal Options"
    )]
    pub format: Option<ByteFormat>,

    /// Display apparent size instead of disk usage.
    #[clap(
        short = 'A',
        long,
        env = "DUA_APPARENT_SIZE",
        help_heading = "Traversal Options"
    )]
    pub apparent_size: bool,

    /// Count hard-linked files each time they are seen
    #[clap(
        short = 'l',
        long,
        env = "DUA_COUNT_HARD_LINKS",
        help_heading = "Traversal Options"
    )]
    pub count_hard_links: bool,

    /// If set, we will not cross filesystems or traverse mount points
    #[clap(
        short = 'x',
        long,
        env = "DUA_STAY_ON_FILESYSTEM",
        help_heading = "Traversal Options"
    )]
    pub stay_on_filesystem: bool,

    /// One or more absolute directories to ignore. Note that these are not ignored if they are passed as input path.
    ///
    /// Hence, they will only be ignored if they are eventually reached as part of the traversal.
    #[clap(
        long = "ignore-dirs",
        short = 'i',
        value_parser,
        env = "DUA_IGNORE_DIRS",
        help_heading = "Traversal Options"
    )]
    #[cfg_attr(target_os = "linux", clap(default_values = DEFAULT_IGNORE_DIRS))]
    pub ignore_dirs: Vec<PathBuf>,

    /// One or more input files or directories. If unset, we will use all entries in the current working directory.
    #[clap(value_parser)]
    pub input: Vec<PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Launch the terminal user interface
    #[cfg(feature = "tui-crossplatform")]
    #[clap(name = "interactive", visible_alias = "i")]
    Interactive {
        #[clap(flatten)]
        traversal: TraversalArgs,
        /// Do not check entries for presence when listing a directory to avoid slugging performance on slow filesystems.
        #[clap(long, short = 'e')]
        no_entry_check: bool,
        /// Exit automatically after traversal, optionally replaying the given single-character keys first.
        #[clap(long, num_args = 0..=1, require_equals = true, default_missing_value = "")]
        once: Option<String>,
    },
    /// Aggregate the consumed space of one or more directories or files
    #[clap(name = "aggregate", visible_alias = "a")]
    Aggregate {
        #[clap(flatten)]
        traversal: TraversalArgs,
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
    /// Configuration related commands
    Config {
        /// Operation to perform on configuration.
        #[clap(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum ConfigCommand {
    /// Open the configuration file in `$EDITOR`.
    ///
    /// If the file does not exist, it will be created with default values first.
    Edit,
    /// Print the default configuration file.
    ///
    /// Use `--reset` to overwrite the active configuration file with these defaults.
    ShowDefault {
        /// Destructively overwrite the active configuration file with the default configuration.
        ///
        /// Local changes will be lost without option to recover.
        #[clap(long = "reset-with-default", visible_alias = "reset")]
        reset_with_default: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::{CommandFactory, Parser};

    #[test]
    fn clap() {
        Args::command().debug_assert();
    }

    #[test]
    fn traversal_options_are_accepted_without_a_subcommand() {
        Args::try_parse_from(["dua", "--format", "metric", "--threads", "1"])
            .expect("root traversal accepts traversal options");
    }

    #[test]
    fn traversal_options_are_accepted_by_aggregate() {
        Args::try_parse_from(["dua", "aggregate", "--format", "metric", "--threads", "1"])
            .expect("aggregate accepts traversal options");
    }

    #[test]
    fn traversal_options_before_aggregate_still_parse_as_subcommand() {
        let args = Args::try_parse_from(["dua", "--format", "metric", "aggregate", "--stats", "."])
            .expect("root traversal options can precede aggregate");

        let Some(super::Command::Aggregate {
            statistics,
            traversal,
            ..
        }) = args.command
        else {
            panic!("expected aggregate subcommand");
        };
        assert!(statistics);
        assert_eq!(traversal.input, [std::path::PathBuf::from(".")]);
    }

    #[test]
    fn traversal_options_are_rejected_after_config_edit() {
        let err = Args::try_parse_from(["dua", "config", "edit", "--format", "metric"])
            .expect_err("config edit should not accept traversal options");

        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn log_file_is_accepted_by_config_edit() {
        Args::try_parse_from(["dua", "config", "edit", "--log-file", "dua.log"])
            .expect("log-file is globally available");
    }

    #[test]
    fn config_show_default_accepts_reset() {
        Args::try_parse_from(["dua", "config", "show-default"]).expect("show-default is available");
        Args::try_parse_from(["dua", "config", "show-default", "--reset"])
            .expect("show-default accepts reset");
    }

    #[test]
    fn traversal_options_have_their_own_help_heading() {
        let mut cmd = Args::command();
        let root_help = cmd.render_long_help().to_string();
        assert!(root_help.contains("Traversal Options"));
        assert!(root_help.contains("--format"));
        assert!(root_help.contains("--log-file"));

        let aggregate_help = cmd
            .find_subcommand_mut("aggregate")
            .expect("aggregate subcommand")
            .render_long_help()
            .to_string();
        assert!(aggregate_help.contains("Traversal Options"));
        assert!(aggregate_help.contains("--format"));
    }

    #[test]
    fn format_uses_config_when_not_set_on_cli() {
        let args = Args::try_parse_from(["dua"]).expect("root traversal parses");
        let config = dua::Config {
            format: Some(dua::ByteFormat::MB),
            ..Default::default()
        };

        assert_eq!(args.traversal.byte_format(&config), dua::ByteFormat::MB);
    }

    #[test]
    fn cli_format_overrides_config() {
        let args = Args::try_parse_from(["dua", "--format", "metric"])
            .expect("root traversal parses with format");
        let config = dua::Config {
            format: Some(dua::ByteFormat::MB),
            ..Default::default()
        };

        assert_eq!(args.traversal.byte_format(&config), dua::ByteFormat::Metric);
    }
}
