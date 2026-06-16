use crate::interactive::path_of;
use dua::traverse::{Tree, TreeIndex};
use itertools::Itertools;
use petgraph::Direction;
use std::time::SystemTime;
use std::{cmp::Ordering, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;

/// Controls which modification time is used for mtime sorting.
#[derive(Default, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum MTimeSort {
    /// Use each entry's own modification time.
    #[default]
    Entry,
    /// Use the newest modification time among each entry's descendants.
    RecursiveChildrenNewest,
    /// Use the oldest modification time among each entry's descendants.
    RecursiveChildrenOldest,
}

impl MTimeSort {
    fn cycle(self) -> Self {
        use MTimeSort::*;
        match self {
            Entry => RecursiveChildrenNewest,
            RecursiveChildrenNewest => RecursiveChildrenOldest,
            RecursiveChildrenOldest => Entry,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    SizeDescending,
    SizeAscending,
    MTimeDescending(MTimeSort),
    MTimeAscending(MTimeSort),
    CountDescending,
    CountAscending,
    NameDescending,
    NameAscending,
}

impl SortMode {
    pub fn toggle_size(&mut self) {
        use SortMode::*;
        *self = match self {
            SizeDescending => SizeAscending,
            SizeAscending => SizeDescending,
            _ => SizeDescending,
        }
    }

    pub fn toggle_mtime(&mut self) {
        use SortMode::*;
        *self = match self {
            MTimeAscending(sort) => MTimeDescending(*sort),
            MTimeDescending(sort) => MTimeAscending(*sort),
            _ => MTimeDescending(MTimeSort::Entry),
        }
    }

    pub fn cycle_mtime_sort(&mut self) {
        match self {
            SortMode::MTimeAscending(sort) | SortMode::MTimeDescending(sort) => {
                *sort = sort.cycle();
            }
            _ => {}
        }
    }

    pub fn mtime_sort(self) -> Option<MTimeSort> {
        match self {
            SortMode::MTimeAscending(sort) | SortMode::MTimeDescending(sort) => Some(sort),
            _ => None,
        }
    }

    pub fn toggle_count(&mut self) {
        use SortMode::*;
        *self = match self {
            CountAscending => CountDescending,
            CountDescending => CountAscending,
            _ => CountDescending,
        }
    }

    pub fn toggle_name(&mut self) {
        use SortMode::*;
        *self = match self {
            NameAscending => NameDescending,
            NameDescending => NameAscending,
            _ => NameAscending,
        }
    }
}

pub struct EntryDataBundle {
    pub index: TreeIndex,
    pub name: PathBuf,
    pub size: u128,
    pub mtime: SystemTime,
    pub entry_count: Option<u64>,
    pub is_dir: bool,
    pub exists: bool,
}

pub enum EntryCheck {
    PossiblyCostlyLstat,
    Disabled,
}

impl EntryCheck {
    pub fn new(is_scanning: bool, allow_entry_check: bool) -> Self {
        if allow_entry_check && !is_scanning {
            EntryCheck::PossiblyCostlyLstat
        } else {
            EntryCheck::Disabled
        }
    }
}

/// Note that with `glob_root` present, we will not obtain metadata anymore as we might be seeing
/// a lot of entries. That way, displaying 250k entries is no problem.
pub fn sorted_entries(
    tree: &Tree,
    node_idx: TreeIndex,
    sorting: SortMode,
    glob_root: Option<TreeIndex>,
    check: EntryCheck,
) -> Vec<EntryDataBundle> {
    use SortMode::*;
    let mtime_sort = sorting.mtime_sort().unwrap_or_default();
    fn cmp_count(l: &EntryDataBundle, r: &EntryDataBundle) -> Ordering {
        l.entry_count
            .cmp(&r.entry_count)
            .then_with(|| l.name.cmp(&r.name))
    }
    fn cmp_name(l: &EntryDataBundle, r: &EntryDataBundle) -> Ordering {
        if l.is_dir && !r.is_dir {
            Ordering::Less
        } else if !l.is_dir && r.is_dir {
            Ordering::Greater
        } else {
            l.name.cmp(&r.name)
        }
    }
    tree.neighbors_directed(node_idx, Direction::Outgoing)
        .filter_map(|idx| {
            tree.node_weight(idx).map(|entry| {
                let use_glob_path = glob_root.is_some_and(|glob_root| glob_root == node_idx);
                let (path, exists, is_dir) = {
                    let path = path_of(tree, idx, glob_root);
                    if matches!(check, EntryCheck::Disabled) || glob_root == Some(node_idx) {
                        (path, true, entry.is_dir)
                    } else {
                        let meta = path.symlink_metadata();
                        (path, meta.is_ok(), meta.ok().is_some_and(|m| m.is_dir()))
                    }
                };
                EntryDataBundle {
                    index: idx,
                    name: if use_glob_path {
                        path
                    } else {
                        entry.name.clone()
                    },
                    size: entry.size,
                    mtime: mtime_for_sort(tree, idx, entry.mtime, mtime_sort),
                    entry_count: entry.entry_count,
                    exists,
                    is_dir,
                }
            })
        })
        .sorted_by(|l, r| match sorting {
            SizeDescending => r.size.cmp(&l.size),
            SizeAscending => l.size.cmp(&r.size),
            MTimeAscending(_) => l.mtime.cmp(&r.mtime),
            MTimeDescending(_) => r.mtime.cmp(&l.mtime),
            CountAscending => cmp_count(l, r),
            CountDescending => cmp_count(l, r).reverse(),
            NameAscending => cmp_name(l, r),
            NameDescending => cmp_name(l, r).reverse(),
        })
        .collect()
}

fn mtime_for_sort(
    tree: &Tree,
    node_idx: TreeIndex,
    entry_mtime: SystemTime,
    sort: MTimeSort,
) -> SystemTime {
    use MTimeSort::*;
    match sort {
        Entry => entry_mtime,
        RecursiveChildrenNewest => max_mtime_of_descendants(tree, node_idx).unwrap_or(entry_mtime),
        RecursiveChildrenOldest => min_mtime_of_descendants(tree, node_idx).unwrap_or(entry_mtime),
    }
}

fn max_mtime_of_descendants(tree: &Tree, node_idx: TreeIndex) -> Option<SystemTime> {
    mtime_of_descendants_with_ordering(tree, node_idx, Ordering::Greater)
}

fn min_mtime_of_descendants(tree: &Tree, node_idx: TreeIndex) -> Option<SystemTime> {
    mtime_of_descendants_with_ordering(tree, node_idx, Ordering::Less)
}

fn mtime_of_descendants_with_ordering(
    tree: &Tree,
    node_idx: TreeIndex,
    ordering: Ordering,
) -> Option<SystemTime> {
    let mut stack: Vec<_> = tree
        .neighbors_directed(node_idx, Direction::Outgoing)
        .collect();
    let mut selected_mtime: Option<SystemTime> = None;
    while let Some(idx) = stack.pop() {
        if let Some(entry) = tree.node_weight(idx) {
            selected_mtime = Some(selected_mtime.map_or(entry.mtime, |selected| {
                if entry.mtime.cmp(&selected) == ordering {
                    entry.mtime
                } else {
                    selected
                }
            }));
            stack.extend(tree.neighbors_directed(idx, Direction::Outgoing));
        }
    }
    selected_mtime
}

pub fn fit_string_graphemes_with_ellipsis(
    s: impl Into<String>,
    path_graphemes_count: usize,
    mut desired_graphemes: usize,
) -> (String, usize) {
    const ELLIPSIS: usize = 1;
    const MIN_GRAPHEMES_ON_SIDE: usize = 1;
    const MIN_LEN: usize = ELLIPSIS + MIN_GRAPHEMES_ON_SIDE;
    const USE_EXTENDED: bool = true;

    let s = s.into();
    desired_graphemes = desired_graphemes.max(MIN_LEN);

    debug_assert!(
        path_graphemes_count == s.graphemes(USE_EXTENDED).count(),
        "input grapheme count is actually correct"
    );

    let gc = path_graphemes_count;
    if gc <= desired_graphemes {
        return (s, gc);
    }

    let mut n = String::with_capacity(desired_graphemes);
    let to_be_removed = gc - desired_graphemes + ELLIPSIS;
    let gmi = s.graphemes(USE_EXTENDED);

    n.push('…');
    n.extend(gmi.skip(to_be_removed));
    (n, desired_graphemes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_string_inputs() {
        assert_eq!(
            ("aaa".into(), 3),
            fit_string_graphemes_with_ellipsis("aaa", 3, 4)
        );
        assert_eq!(
            ("…a".to_string(), 2),
            fit_string_graphemes_with_ellipsis("abbbba", 6, 1),
            "even amount of chars, desired too small"
        );
        assert_eq!(
            ("…ca".to_string(), 3),
            fit_string_graphemes_with_ellipsis("abbbbca", 7, 3),
            "uneven amount of chars, desired too small"
        );
        assert_eq!(
            ("… a".to_string(), 3),
            fit_string_graphemes_with_ellipsis("a    a", 6, 3),
            "spaces are counted as graphemes, too"
        );
    }
}
