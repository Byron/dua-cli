use crate::{
    ByteFormat,
    snapshot::{DecodedEntry, Replay, ReplayEntries},
};
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::{
    borrow::Cow,
    cmp::Ordering,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct KeyPart {
    name: Vec<u8>,
    sibling_ordinal: u64,
}

#[derive(Clone, Copy)]
struct Current {
    depth: usize,
    size: u128,
    is_dir: bool,
}

struct Cursor<'a, R> {
    entries: ReplayEntries<'a, R>,
    current: Option<Current>,
    root_ordinal: usize,
    roots_seen: usize,
    key: Vec<KeyPart>,
    path: PathBuf,
    path_depth: usize,
}

#[derive(Clone, Copy)]
enum Change {
    Added(u128),
    Removed(u128),
    Modified { sign: char, magnitude: u128 },
}

struct Location<'a> {
    root_ordinal: usize,
    key: &'a [KeyPart],
    path: &'a Path,
    depth: usize,
    is_dir: bool,
}

#[derive(Default)]
struct TreeState {
    root_ordinal: Option<usize>,
    key: Vec<KeyPart>,
}

#[derive(Default)]
struct Summary {
    additions: Vec<SummaryEntry>,
    removals: Vec<SummaryEntry>,
    additions_total: u64,
    removals_total: u64,
    changes_total: u64,
}

struct SummaryEntry {
    size: u128,
    path: PathBuf,
    is_dir: bool,
}

impl Summary {
    fn record(&mut self, change: Change, entry: Option<SummaryEntry>, limit: usize) {
        self.changes_total = self.changes_total.saturating_add(1);
        let (entries, total) = match change {
            Change::Added(_) => (&mut self.additions, &mut self.additions_total),
            Change::Removed(_) => (&mut self.removals, &mut self.removals_total),
            Change::Modified { .. } => return,
        };
        *total = total.saturating_add(1);
        if limit == 0 {
            return;
        }
        entries.push(entry.expect("additions and removals carry their summary entry"));
        entries.sort_by(|left, right| {
            right
                .size
                .cmp(&left.size)
                .then_with(|| left.path.cmp(&right.path))
        });
        entries.truncate(limit);
    }
}

impl<'a, R: Read> Cursor<'a, R> {
    fn new(entries: ReplayEntries<'a, R>, prefix: Option<&Path>) -> Result<Self> {
        let mut cursor = Self {
            entries,
            current: None,
            root_ordinal: 0,
            roots_seen: 0,
            key: Vec::new(),
            path: PathBuf::new(),
            path_depth: 0,
        };
        cursor.advance(prefix)?;
        Ok(cursor)
    }

    fn advance(&mut self, prefix: Option<&Path>) -> Result<()> {
        self.advance_raw()?;
        self.skip_to_prefix(prefix)
    }

    fn skip_to_prefix(&mut self, prefix: Option<&Path>) -> Result<()> {
        while self
            .current
            .is_some_and(|_| prefix.is_some_and(|prefix| !self.path.starts_with(prefix)))
        {
            self.advance_raw()?;
        }
        Ok(())
    }

    fn advance_raw(&mut self) -> Result<()> {
        let Some(DecodedEntry {
            depth,
            data,
            native_name,
            sibling_ordinal,
        }) = self.entries.next_entry()?
        else {
            self.current = None;
            return Ok(());
        };

        if depth == 0 {
            self.root_ordinal = self.roots_seen;
            self.roots_seen = self
                .roots_seen
                .checked_add(1)
                .context("snapshot contains too many roots")?;
            self.key.clear();
            self.path = data.name;
            self.path_depth = 0;
        } else {
            self.key.truncate(depth);
            while self.path_depth >= depth {
                self.path.pop();
                self.path_depth -= 1;
            }
            self.path.push(&data.name);
            self.path_depth = depth;
        }
        self.key.push(KeyPart {
            name: native_name,
            sibling_ordinal,
        });
        self.current = Some(Current {
            depth,
            size: data.size,
            is_dir: data.is_dir,
        });
        Ok(())
    }

    fn cmp_key<Rhs: Read>(&self, rhs: &Cursor<'_, Rhs>) -> Ordering {
        self.root_ordinal
            .cmp(&rhs.root_ordinal)
            .then_with(|| self.key.cmp(&rhs.key))
    }

    fn advance_past_current(&mut self, prefix: Option<&Path>) -> Result<()> {
        let Some(current) = self.current else {
            return Ok(());
        };
        self.advance_raw()?;
        if current.is_dir {
            while self
                .current
                .is_some_and(|entry| entry.depth > current.depth)
            {
                self.advance_raw()?;
            }
        }
        self.skip_to_prefix(prefix)
    }

    fn location(&self, current: Current) -> Location<'_> {
        Location {
            root_ordinal: self.root_ordinal,
            key: &self.key,
            path: &self.path,
            depth: current.depth,
            is_dir: current.is_dir,
        }
    }
}

/// Compare two verified traversal snapshots without materializing either tree.
///
/// If `directories_only` is true, report aggregate directory changes instead of file changes.
/// `prefix` limits output to that stored path and its descendants.
/// `max_depth` limits the displayed depth below the selected roots while still reading all entries.
/// `summary_limit` bounds each largest-addition and largest-removal list; zero hides both.
/// Changes are streamed before their summary is printed.
#[allow(clippy::too_many_arguments)]
pub fn diff_snapshots<Old: Read + Seek, New: Read + Seek>(
    out: (impl io::Write, bool),
    old: &mut Replay<Old>,
    new: &mut Replay<New>,
    byte_format: ByteFormat,
    directories_only: bool,
    prefix: Option<&Path>,
    max_depth: Option<usize>,
    summary_limit: usize,
) -> Result<()> {
    let (mut out, out_is_terminal) = out;
    let (send, receive) = std::sync::mpsc::channel();
    let summary = std::thread::spawn(move || {
        let mut summary = Summary::default();
        for (change, entry) in receive {
            summary.record(change, entry, summary_limit);
        }
        summary
    });

    let mut state = TreeState::default();
    let walk_result = walk_changes(old, new, directories_only, prefix, |change, location| {
        let summary_entry = match change {
            _ if summary_limit == 0 => None,
            Change::Added(size) | Change::Removed(size) => Some(SummaryEntry {
                size,
                path: location.path.to_owned(),
                is_dir: location.is_dir,
            }),
            Change::Modified { .. } => None,
        };
        send.send((change, summary_entry))
            .map_err(|_| anyhow::anyhow!("summary collector stopped"))?;
        write_tree_entry(
            &mut out,
            &mut state,
            change,
            location,
            byte_format,
            out_is_terminal,
            prefix,
            max_depth,
        )?;
        Ok(())
    });
    drop(send);
    let summary = summary
        .join()
        .map_err(|_| anyhow::anyhow!("summary collector panicked"))?;
    walk_result?;
    if summary.changes_total == 0 {
        return Ok(());
    }
    writeln!(out)?;
    write_summary(&mut out, &summary, byte_format, out_is_terminal)?;
    Ok(())
}

fn write_summary(
    out: &mut impl io::Write,
    summary: &Summary,
    byte_format: ByteFormat,
    out_is_terminal: bool,
) -> io::Result<()> {
    let mut wrote_entries = false;
    for (title, entries, total, change) in [
        (
            "removals",
            &summary.removals,
            summary.removals_total,
            Change::Removed as fn(u128) -> Change,
        ),
        (
            "additions",
            &summary.additions,
            summary.additions_total,
            Change::Added as fn(u128) -> Change,
        ),
    ] {
        if entries.is_empty() {
            continue;
        }
        if wrote_entries {
            writeln!(out)?;
        }
        writeln!(
            out,
            "Largest {title} (showing {} of {total}):",
            entries.len()
        )?;
        for entry in entries {
            write_change(
                out,
                change(entry.size),
                &entry.path,
                entry.is_dir,
                0,
                false,
                (byte_format, out_is_terminal),
            )?;
        }
        wrote_entries = true;
    }
    if wrote_entries {
        writeln!(out)?;
    }
    writeln!(out, "Changes: {}", summary.changes_total)
}

fn walk_changes<Old: Read + Seek, New: Read + Seek>(
    old: &mut Replay<Old>,
    new: &mut Replay<New>,
    directories_only: bool,
    prefix: Option<&Path>,
    mut on_change: impl FnMut(Change, Location<'_>) -> Result<()>,
) -> Result<()> {
    let mut old = Cursor::new(old.entries()?, prefix)?;
    let mut new = Cursor::new(new.entries()?, prefix)?;

    loop {
        let ordering = match (old.current, new.current) {
            (Some(_), Some(_)) => Some(old.cmp_key(&new)),
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
            (None, None) => None,
        };
        match ordering {
            Some(Ordering::Less) => {
                let Some(entry) = old.current else {
                    return Ok(());
                };
                if !directories_only || entry.is_dir {
                    on_change(Change::Removed(entry.size), old.location(entry))?;
                }
                old.advance_past_current(prefix)?;
            }
            Some(Ordering::Greater) => {
                let Some(entry) = new.current else {
                    return Ok(());
                };
                if !directories_only || entry.is_dir {
                    on_change(Change::Added(entry.size), new.location(entry))?;
                }
                new.advance_past_current(prefix)?;
            }
            Some(Ordering::Equal) => {
                let (Some(old_entry), Some(new_entry)) = (old.current, new.current) else {
                    return Ok(());
                };
                if old_entry.is_dir == new_entry.is_dir {
                    if old_entry.is_dir == directories_only && old_entry.size != new_entry.size {
                        let (sign, magnitude) = if new_entry.size > old_entry.size {
                            ('+', new_entry.size - old_entry.size)
                        } else {
                            ('-', old_entry.size - new_entry.size)
                        };
                        on_change(
                            Change::Modified { sign, magnitude },
                            new.location(new_entry),
                        )?;
                    }
                    old.advance(prefix)?;
                    new.advance(prefix)?;
                } else {
                    if !directories_only || old_entry.is_dir {
                        on_change(Change::Removed(old_entry.size), old.location(old_entry))?;
                    }
                    if !directories_only || new_entry.is_dir {
                        on_change(Change::Added(new_entry.size), new.location(new_entry))?;
                    }
                    old.advance_past_current(prefix)?;
                    new.advance_past_current(prefix)?;
                }
            }
            None => return Ok(()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn write_tree_entry(
    out: &mut impl io::Write,
    state: &mut TreeState,
    change: Change,
    location: Location<'_>,
    byte_format: ByteFormat,
    out_is_terminal: bool,
    prefix: Option<&Path>,
    max_depth: Option<usize>,
) -> io::Result<()> {
    let mut hierarchy = location
        .path
        .ancestors()
        .take(location.depth + 1)
        .collect::<Vec<_>>();
    hierarchy.reverse();
    let base_depth = prefix
        .and_then(|prefix| hierarchy.iter().position(|path| *path == prefix))
        .unwrap_or(0);
    let common_depth = if state.root_ordinal == Some(location.root_ordinal) {
        state
            .key
            .iter()
            .zip(location.key)
            .take_while(|(left, right)| left == right)
            .count()
    } else {
        0
    };
    let visible_depth = location
        .depth
        .min(base_depth.saturating_add(max_depth.unwrap_or(usize::MAX)));
    let change_is_visible = visible_depth == location.depth;
    let context_count = if change_is_visible {
        location.depth
    } else {
        visible_depth + 1
    };

    for (depth, path) in hierarchy
        .iter()
        .enumerate()
        .take(context_count)
        .skip(common_depth.max(base_depth))
    {
        write_context(
            out,
            display_path(path, depth == base_depth),
            depth - base_depth,
            !change_is_visible && depth == visible_depth,
            out_is_terminal,
        )?;
    }

    if change_is_visible {
        write_change(
            out,
            change,
            display_path(location.path, location.depth == base_depth),
            location.is_dir,
            location.depth - base_depth,
            location.is_dir
                && matches!(change, Change::Modified { .. })
                && max_depth.is_some_and(|max_depth| location.depth - base_depth == max_depth),
            (byte_format, out_is_terminal),
        )?;
    }
    state.root_ordinal = Some(location.root_ordinal);
    state.key.clear();
    state.key.extend_from_slice(&location.key[..=visible_depth]);
    Ok(())
}

fn write_context(
    out: &mut impl io::Write,
    path: &Path,
    depth: usize,
    has_hidden_changes: bool,
    out_is_terminal: bool,
) -> io::Result<()> {
    let path = path_for_output(path);
    let suffix = directory_suffix(path.as_ref());
    let hidden = if has_hidden_changes { " …" } else { "" };
    let line = format!("{}{path}{suffix}{hidden}", "  ".repeat(depth));
    if out_is_terminal {
        writeln!(out, "{}", line.cyan())
    } else {
        writeln!(out, "{line}")
    }
}

fn write_change(
    out: &mut impl io::Write,
    change: Change,
    path: &Path,
    is_dir: bool,
    depth: usize,
    has_hidden_changes: bool,
    (byte_format, out_is_terminal): (ByteFormat, bool),
) -> io::Result<()> {
    let (kind, sign, magnitude) = match change {
        Change::Added(size) => ('+', "", size),
        Change::Removed(size) => ('-', "", size),
        Change::Modified { sign, magnitude } => {
            ('~', if sign == '+' { "+" } else { "-" }, magnitude)
        }
    };
    let path = path_for_output(path);
    let suffix = if is_dir {
        directory_suffix(path.as_ref())
    } else {
        ""
    };
    let hidden = if has_hidden_changes { " …" } else { "" };
    let line = format!(
        "{}{kind} {sign}{} {path}{suffix}{hidden}",
        "  ".repeat(depth),
        byte_format.display(magnitude)
    );
    if !out_is_terminal {
        return writeln!(out, "{line}");
    }
    match change {
        Change::Added(_) => writeln!(out, "{}", line.green()),
        Change::Removed(_) => writeln!(out, "{}", line.red()),
        Change::Modified { .. } => writeln!(out, "{}", line.yellow()),
    }
}

fn display_path(path: &Path, full: bool) -> &Path {
    if full {
        path
    } else {
        path.file_name().map_or(path, Path::new)
    }
}

fn directory_suffix(path: &str) -> &'static str {
    if path.chars().last().is_some_and(std::path::is_separator) {
        ""
    } else {
        std::path::MAIN_SEPARATOR_STR
    }
}

fn path_for_output(path: &Path) -> Cow<'_, str> {
    let path = path.to_string_lossy();
    if path.chars().any(char::is_control) {
        path.chars()
            .map(|character| {
                if character.is_control() {
                    '\u{FFFD}'
                } else {
                    character
                }
            })
            .collect::<String>()
            .into()
    } else {
        path
    }
}
