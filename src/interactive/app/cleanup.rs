use std::collections::BTreeSet;

use dua::traverse::TreeIndex;

use super::EntryDataBundle;

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
        && dua::clean::is_cleanup_dir_name(
            entry
                .name
                .file_name()
                .unwrap_or_else(|| entry.name.as_os_str()),
        )
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
}
