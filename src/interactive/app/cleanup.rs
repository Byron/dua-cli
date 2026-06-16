use std::{collections::BTreeSet, ffi::OsStr};

use dua::traverse::TreeIndex;

use super::EntryDataBundle;

/// Refresh with: `sed -n '/^const CLEANUP_DIR_NAMES_SORTED/,/^];/p' src/interactive/app/cleanup.rs | rg -o '"[^"]+"' | LC_ALL=C sort | sed 's/^/    /; s/$/,/'`.
const CLEANUP_DIR_NAMES_SORTED_FOR_BISECT: &[&str] = &[
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".tox",
    ".venv",
    "__pycache__",
    "node_modules",
    "target",
    "venv",
];

/// Return the indices of existing directories that match known cleanup names.
pub fn cleanup_candidates(entries: &[EntryDataBundle]) -> BTreeSet<TreeIndex> {
    entries
        .iter()
        .filter(|entry| is_cleanup_candidate(entry))
        .map(|entry| entry.index)
        .collect()
}

fn is_cleanup_candidate(entry: &EntryDataBundle) -> bool {
    entry.exists
        && entry.is_dir
        && is_cleanup_dir_name(
            entry
                .name
                .file_name()
                .unwrap_or_else(|| entry.name.as_os_str()),
        )
}

fn is_cleanup_dir_name(name: &OsStr) -> bool {
    CLEANUP_DIR_NAMES_SORTED_FOR_BISECT
        .binary_search_by(|candidate| OsStr::new(candidate).cmp(name))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(name: &str, is_dir: bool) -> EntryDataBundle {
        EntryDataBundle {
            index: TreeIndex::new(0),
            name: PathBuf::from(name),
            size: 0,
            mtime: std::time::SystemTime::UNIX_EPOCH,
            entry_count: None,
            is_dir,
            exists: true,
        }
    }

    #[test]
    fn identifies_conservative_cleanup_directories() {
        for name in ["target", "node_modules", "__pycache__", ".venv"] {
            assert!(is_cleanup_candidate(&entry(name, true)));
        }
        for path in ["project/target", "project/node_modules", "project/.venv"] {
            assert!(
                is_cleanup_candidate(&entry(path, true)),
                "file paths are expected as `name` field."
            );
        }
    }

    #[test]
    fn ignores_files_and_ambiguous_build_outputs() {
        for name in ["target", "build", "dist"] {
            assert!(!is_cleanup_candidate(&entry(name, false)));
        }
        assert!(!is_cleanup_candidate(&entry("build", true)));
        assert!(!is_cleanup_candidate(&entry("dist", true)));
    }

    #[test]
    fn cleanup_directory_names_remain_sorted_for_bisection() {
        assert!(
            CLEANUP_DIR_NAMES_SORTED_FOR_BISECT
                .windows(2)
                .all(|names| OsStr::new(names[0]) < OsStr::new(names[1])),
            "CLEANUP_DIR_NAMES_SORTED must remain sorted for binary search"
        );
    }
}
