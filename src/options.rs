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

const DEFAULT_DIFF_SUMMARY_LIMIT: usize = 5;

#[cfg(feature = "tui-crossplatform")]
fn parse_snapshot_compression_level(value: &str) -> Result<i32, String> {
    let level = value
        .parse::<i32>()
        .map_err(|_| format!("invalid compression level: {value}"))?;
    let maximum = gix::zlib::Compression::BEST.level();
    if gix::zlib::Compression::new(level).is_some() {
        Ok(level)
    } else {
        Err(format!("compression level must be between 0 and {maximum}"))
    }
}

/// Enough parallelism to keep filesystem work moving without saturating macOS with syscalls.
#[cfg(target_os = "macos")]
pub(crate) const DEFAULT_THREADS: usize = 8;

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
#[cfg_attr(
    target_os = "macos",
    expect(
        clippy::struct_excessive_bools,
        reason = "independent command-line switches map directly to booleans"
    )
)]
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

    /// Count fully shared APFS file clones only once. This costs about 6% performance.
    #[cfg(target_os = "macos")]
    #[clap(long, help_heading = "Traversal Options")]
    pub deduplicate_apfs_clones: bool,

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

    /// One or more files with gitignore-style patterns, whose matches are left out of the report.
    ///
    /// Patterns follow `.gitignore` syntax - `#` starts a comment, a trailing `/` matches
    /// directories only, a leading `/` anchors to the top, `**` spans directories, and `!`
    /// re-includes what an earlier pattern excluded. They are case-sensitively on all platforms.
    ///
    /// Files given later take precedence over files given earlier. Excluded directories are not
    /// descended into, so their contents cannot be re-included.
    #[clap(
        long = "ignore-from",
        value_parser,
        env = "DUA_IGNORE_FROM",
        value_name = "FILE",
        help_heading = "Traversal Options"
    )]
    pub ignore_from: Vec<PathBuf>,

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
        /// Write the completed traversal to this snapshot file.
        #[clap(long, value_name = "FILE", conflicts_with = "import")]
        export: Option<PathBuf>,
        /// Snapshot compression level when exporting. Use 0 to disable compression.
        #[clap(
            long,
            env = "DUA_SNAPSHOT_COMPRESSION_LEVEL",
            value_name = "LEVEL",
            default_value_t = 2,
            allow_hyphen_values = true,
            value_parser = parse_snapshot_compression_level
        )]
        compression: i32,
        /// Load a snapshot instead of traversing the filesystem.
        #[clap(
            long,
            value_name = "FILE",
            conflicts_with_all = [
                "input",
                "threads",
                "apparent_size",
                "count_hard_links",
                "stay_on_filesystem",
                "ignore_dirs",
                "ignore_from",
                "no_entry_check"
            ]
        )]
        #[cfg_attr(target_os = "macos", clap(conflicts_with = "deduplicate_apfs_clones"))]
        import: Option<PathBuf>,
        /// Do not check entries for presence when listing a directory to avoid slugging performance on slow filesystems.
        #[clap(long, short = 'e', conflicts_with = "import")]
        no_entry_check: bool,
        /// Exit automatically after traversal, optionally replaying a configured keybinding or compact character sequence first.
        #[clap(long, num_args = 0..=1, require_equals = true, default_missing_value = "")]
        once: Option<String>,
    },
    /// Compare two traversal snapshots
    Diff {
        /// Earlier traversal snapshot.
        #[clap(value_name = "OLD")]
        old: PathBuf,
        /// Later traversal snapshot.
        #[clap(value_name = "NEW")]
        new: PathBuf,
        /// The format with which to print byte counts.
        #[clap(short = 'f', long, value_enum, ignore_case = true)]
        format: Option<ByteFormat>,
        /// Report aggregate directory changes instead of file changes.
        #[clap(long)]
        directories_only: bool,
        /// Show only this stored path and its descendants.
        #[clap(long, value_name = "PATH")]
        prefix: Option<PathBuf>,
        /// Limit the tree to this many levels, summarizing hidden additions and removals. The root
        /// or selected prefix is the first level.
        #[clap(short = 'd', long, value_name = "DEPTH", value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..))]
        depth: Option<usize>,
        /// Include at most this many largest additions and removals in the summary. Use 0 to hide them.
        #[clap(long, value_name = "COUNT", default_value_t = DEFAULT_DIFF_SUMMARY_LIMIT)]
        summary_limit: usize,
    },
    /// Aggregate the consumed space of one or more directories or files
    #[clap(name = "aggregate", visible_alias = "a")]
    Aggregate {
        #[clap(flatten)]
        traversal: TraversalArgs,
        /// Load a traversal snapshot instead of scanning the filesystem.
        #[clap(
            long,
            value_name = "FILE",
            conflicts_with_all = [
                "input",
                "threads",
                "apparent_size",
                "count_hard_links",
                "stay_on_filesystem",
                "ignore_dirs",
                "ignore_from",
                "statistics"
            ]
        )]
        #[cfg_attr(target_os = "macos", clap(conflicts_with = "deduplicate_apfs_clones"))]
        import: Option<PathBuf>,
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
        /// Print folded stacks for flame-graph tools instead of a table or tree.
        #[clap(long, conflicts_with_all = ["statistics", "no_sort", "no_total"])]
        stack: bool,
        /// Print an indented tree that descends this many levels into each input, instead of the
        /// flat listing. With `--stack`, limit the folded output to the same depth. The inputs form
        /// the first level, so a depth of 1 lists just them.
        #[clap(short = 'd', long, conflicts_with = "statistics", value_name = "DEPTH", value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..))]
        depth: Option<usize>,
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
    use std::path::PathBuf;

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
    fn ignore_from_is_repeatable_and_available_to_subcommands() {
        let args = Args::try_parse_from([
            "dua",
            "--ignore-from",
            "global",
            "aggregate",
            "--ignore-from",
            "sub-one",
            "--ignore-from",
            "sub-two",
        ])
        .expect("ignore-from parses at both levels");

        assert_eq!(args.traversal.ignore_from, [PathBuf::from("global")]);
        let Some(super::Command::Aggregate { traversal, .. }) = args.command else {
            panic!("expected aggregate subcommand");
        };
        assert_eq!(
            traversal.ignore_from,
            [PathBuf::from("sub-one"), PathBuf::from("sub-two")]
        );
    }

    #[test]
    fn traversal_options_are_rejected_after_config_edit() {
        let err = Args::try_parse_from(["dua", "config", "edit", "--format", "metric"])
            .expect_err("config edit should not accept traversal options");

        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn diff_accepts_format_before_or_after_the_subcommand() {
        let before =
            Args::try_parse_from(["dua", "--format", "bytes", "diff", "old.dua", "new.dua"])
                .expect("global format parses");
        assert_eq!(before.traversal.format, Some(super::ByteFormat::Bytes));

        let after =
            Args::try_parse_from(["dua", "diff", "old.dua", "new.dua", "--format", "bytes"])
                .expect("diff format parses");
        let Some(super::Command::Diff {
            old,
            new,
            format,
            summary_limit,
            ..
        }) = after.command
        else {
            panic!("expected diff subcommand");
        };
        assert_eq!(old, PathBuf::from("old.dua"));
        assert_eq!(new, PathBuf::from("new.dua"));
        assert_eq!(format, Some(super::ByteFormat::Bytes));
        assert_eq!(summary_limit, super::DEFAULT_DIFF_SUMMARY_LIMIT);

        let directories =
            Args::try_parse_from(["dua", "diff", "old.dua", "new.dua", "--directories-only"])
                .expect("directory-only diff parses");
        assert!(matches!(
            directories.command,
            Some(super::Command::Diff {
                directories_only: true,
                ..
            })
        ));

        let prefixed = Args::try_parse_from([
            "dua",
            "diff",
            "old.dua",
            "new.dua",
            "--prefix",
            "root/subtree",
        ])
        .expect("prefixed diff parses");
        assert!(matches!(
            prefixed.command,
            Some(super::Command::Diff {
                prefix: Some(path),
                ..
            }) if path == std::path::Path::new("root/subtree")
        ));

        let depth = Args::try_parse_from(["dua", "diff", "old.dua", "new.dua", "--depth", "2"])
            .expect("diff depth parses");
        assert!(matches!(
            depth.command,
            Some(super::Command::Diff { depth: Some(2), .. })
        ));
        assert_eq!(
            Args::try_parse_from(["dua", "diff", "old.dua", "new.dua", "--depth", "0"])
                .expect_err("zero diff depth is invalid")
                .kind(),
            clap::error::ErrorKind::ValueValidation
        );

        let summary_limit =
            Args::try_parse_from(["dua", "diff", "old.dua", "new.dua", "--summary-limit", "27"])
                .expect("diff summary limit parses");
        assert!(matches!(
            summary_limit.command,
            Some(super::Command::Diff {
                summary_limit: 27,
                ..
            })
        ));
    }

    #[test]
    fn diff_rejects_traversal_options_after_the_subcommand() {
        let err = Args::try_parse_from(["dua", "diff", "old.dua", "new.dua", "--threads", "1"])
            .expect_err("diff should not accept traversal options");
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

    #[test]
    fn aggregate_snapshot_import_accepts_display_options() {
        let args = Args::try_parse_from([
            "dua",
            "aggregate",
            "--import",
            "scan.dua",
            "--format",
            "bytes",
            "--no-sort",
            "--no-total",
            "--depth",
            "2",
        ])
        .expect("snapshot import accepts aggregate display options");
        let Some(super::Command::Aggregate {
            import,
            traversal,
            depth,
            ..
        }) = args.command
        else {
            panic!("expected aggregate subcommand");
        };
        assert_eq!(import, Some(PathBuf::from("scan.dua")));
        assert_eq!(traversal.format, Some(super::ByteFormat::Bytes));
        assert_eq!(depth, Some(2));

        Args::try_parse_from([
            "dua",
            "aggregate",
            "--import",
            "scan.dua",
            "--stack",
            "--depth",
            "2",
        ])
        .expect("snapshot import supports folded stacks");
    }

    #[test]
    fn aggregate_import_rejects_traversal_inputs_and_statistics() {
        for args in [
            vec!["dua", "a", "--import", "scan.dua", "input"],
            vec!["dua", "a", "--import", "scan.dua", "--threads", "1"],
            vec!["dua", "a", "--import", "scan.dua", "--apparent-size"],
            vec!["dua", "a", "--import", "scan.dua", "--count-hard-links"],
            vec!["dua", "a", "--import", "scan.dua", "--stay-on-filesystem"],
            vec!["dua", "a", "--import", "scan.dua", "--ignore-dirs", "dir"],
            vec![
                "dua",
                "a",
                "--import",
                "scan.dua",
                "--ignore-from",
                "ignore",
            ],
            vec!["dua", "a", "--import", "scan.dua", "--stats"],
        ] {
            let err = Args::try_parse_from(args).expect_err("import option conflict");
            assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
        }
    }

    #[cfg(feature = "tui-crossplatform")]
    #[test]
    fn interactive_snapshot_options_parse() {
        let args = Args::try_parse_from([
            "dua",
            "interactive",
            "--import",
            "scan.dua",
            "--format",
            "bytes",
            "--once=",
        ])
        .expect("snapshot import accepts display options");
        let Some(super::Command::Interactive {
            import,
            export,
            compression,
            traversal,
            ..
        }) = args.command
        else {
            panic!("expected interactive subcommand");
        };
        assert_eq!(import, Some(PathBuf::from("scan.dua")));
        assert_eq!(export, None);
        assert_eq!(compression, 2);
        assert_eq!(traversal.format, Some(super::ByteFormat::Bytes));

        let args =
            Args::try_parse_from(["dua", "interactive", "--export", "scan.dua", "somewhere"])
                .expect("snapshot export accepts traversal inputs");
        let Some(super::Command::Interactive {
            import,
            export,
            compression,
            traversal,
            ..
        }) = args.command
        else {
            panic!("expected interactive subcommand");
        };
        assert_eq!(import, None);
        assert_eq!(export, Some(PathBuf::from("scan.dua")));
        assert_eq!(compression, 2);
        assert_eq!(traversal.input, [PathBuf::from("somewhere")]);

        assert!(matches!(
            Args::try_parse_from([
                "dua",
                "interactive",
                "--export",
                "scan.dua",
                "--compression",
                "0"
            ])
            .unwrap()
            .command,
            Some(super::Command::Interactive { compression: 0, .. })
        ));
        assert!(matches!(
            Args::try_parse_from([
                "dua",
                "interactive",
                "--export",
                "scan.dua",
                "--compression",
                "7"
            ])
            .unwrap()
            .command,
            Some(super::Command::Interactive { compression: 7, .. })
        ));
        for invalid in ["-1", "10"] {
            assert_eq!(
                Args::try_parse_from([
                    "dua",
                    "interactive",
                    "--export",
                    "scan.dua",
                    "--compression",
                    invalid,
                ])
                .unwrap_err()
                .kind(),
                clap::error::ErrorKind::ValueValidation
            );
        }
    }

    #[cfg(feature = "tui-crossplatform")]
    #[test]
    fn interactive_import_rejects_traversal_inputs_and_export() {
        for args in [
            vec!["dua", "i", "--import", "scan.dua", "input"],
            vec!["dua", "i", "--import", "scan.dua", "--export", "other.dua"],
            vec!["dua", "i", "--import", "scan.dua", "--threads", "1"],
            vec!["dua", "i", "--import", "scan.dua", "--apparent-size"],
            vec!["dua", "i", "--import", "scan.dua", "--count-hard-links"],
            vec!["dua", "i", "--import", "scan.dua", "--stay-on-filesystem"],
            vec!["dua", "i", "--import", "scan.dua", "--ignore-dirs", "dir"],
            vec![
                "dua",
                "i",
                "--import",
                "scan.dua",
                "--ignore-from",
                "ignore",
            ],
            vec!["dua", "i", "--import", "scan.dua", "--no-entry-check"],
        ] {
            let err = Args::try_parse_from(args).expect_err("import option conflict");
            assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
        }
    }

    #[cfg(all(feature = "tui-crossplatform", target_os = "macos"))]
    #[test]
    fn interactive_import_rejects_apfs_deduplication() {
        let err = Args::try_parse_from([
            "dua",
            "i",
            "--import",
            "scan.dua",
            "--deduplicate-apfs-clones",
        ])
        .expect_err("import option conflict");
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
