use std::collections::BTreeSet;

use dua::traverse::TreeIndex;

use super::{EntryDataBundle, tree_view::TreeView};

#[cfg(feature = "git")]
pub fn gitignored_entries(
    tree_view: &TreeView<'_>,
    view_root: TreeIndex,
    entries: &[EntryDataBundle],
) -> BTreeSet<TreeIndex> {
    use std::path::{Path, PathBuf};

    fn absolute_path(path: PathBuf, cwd: &Path) -> PathBuf {
        if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        }
    }

    fn mode(entry: &EntryDataBundle) -> gix::index::entry::Mode {
        if entry.is_dir {
            gix::index::entry::Mode::DIR
        } else {
            gix::index::entry::Mode::FILE
        }
    }

    let current_path = tree_view.path_of(view_root);
    let current_path = if current_path.as_os_str().is_empty() {
        Path::new(".").to_owned()
    } else {
        current_path
    };

    let Ok(repo) = gix::discover(&current_path) else {
        return BTreeSet::new();
    };
    let Ok(cwd) = std::env::current_dir() else {
        return BTreeSet::new();
    };
    let Some(workdir) = repo.workdir() else {
        return BTreeSet::new();
    };
    let workdir = absolute_path(workdir.to_owned(), &cwd);
    let Ok(index) = repo.index_or_empty() else {
        return BTreeSet::new();
    };
    let Ok(mut excludes) = repo.excludes(
        &index,
        None,
        gix::worktree::stack::state::ignore::Source::WorktreeThenIdMappingIfNotSkipped,
    ) else {
        return BTreeSet::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            let path = absolute_path(tree_view.path_of(entry.index), &cwd);
            let relative_path = path.strip_prefix(&workdir).ok()?;
            let platform = excludes.at_path(relative_path, Some(mode(entry))).ok()?;
            platform.is_excluded().then_some(entry.index)
        })
        .collect()
}

#[cfg(not(feature = "git"))]
pub fn gitignored_entries(
    _tree_view: &TreeView<'_>,
    _view_root: TreeIndex,
    _entries: &[EntryDataBundle],
) -> BTreeSet<TreeIndex> {
    BTreeSet::new()
}
