use dua::traverse::{Tree, TreeIndex};
use std::time::SystemTime;
use std::{cmp::Ordering, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
        use MTimeSort::{Entry, RecursiveChildrenNewest, RecursiveChildrenOldest};
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
        use SortMode::{SizeAscending, SizeDescending};
        *self = match self {
            SizeDescending => SizeAscending,
            _ => SizeDescending,
        }
    }

    pub fn toggle_mtime(&mut self) {
        use SortMode::{MTimeAscending, MTimeDescending};
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
        use SortMode::{CountAscending, CountDescending};
        *self = match self {
            CountDescending => CountAscending,
            _ => CountDescending,
        }
    }

    pub fn toggle_name(&mut self) {
        use SortMode::{NameAscending, NameDescending};
        *self = match self {
            NameAscending => NameDescending,
            _ => NameAscending,
        }
    }
}

/// Filesystem entry data prepared for interactive views.
pub struct EntryDataBundle {
    /// Index of this entry in the traversal tree.
    pub index: TreeIndex,
    /// Display path for this entry in the current view.
    ///
    /// This is usually the entry name relative to its parent, but may be a
    /// multi-component path when the current view needs to show entries outside
    /// their immediate parent. Use `file_name()` when only the basename matters.
    pub name: PathBuf,
    /// Entry size in bytes, including recursive child sizes for directories.
    pub size: u128,
    /// Modification time used by the active view or sort mode.
    pub mtime: SystemTime,
    /// Recursive child entry count for directories, or `None` for files.
    pub entry_count: Option<u64>,
    /// Whether this entry currently resolves to a directory.
    pub is_dir: bool,
    /// Whether this entry still exists when metadata is checked for the view.
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

/// Full-path views skip per-entry metadata queries to keep large glob views responsive.
pub fn sorted_entries(
    tree: &Tree,
    indices: impl IntoIterator<Item = TreeIndex>,
    sorting: SortMode,
    use_full_path: bool,
    check: EntryCheck,
) -> Vec<EntryDataBundle> {
    use SortMode::{
        CountAscending, CountDescending, MTimeAscending, MTimeDescending, NameAscending,
        NameDescending, SizeAscending, SizeDescending,
    };
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
    let mtime_sort = sorting.mtime_sort().unwrap_or_default();
    let mut entries = indices
        .into_iter()
        .filter_map(|idx| {
            tree.entry(idx).map(|entry| {
                let data = entry.data;
                let (path, exists, is_dir) = {
                    let path = tree.path_of(idx);
                    if matches!(check, EntryCheck::Disabled) || use_full_path {
                        (path, true, data.is_dir)
                    } else {
                        let meta = path.symlink_metadata();
                        let exists = meta.is_ok();
                        (path, exists, meta.is_ok_and(|m| m.is_dir()))
                    }
                };
                EntryDataBundle {
                    index: idx,
                    name: if use_full_path {
                        path
                    } else {
                        entry.name.into_owned()
                    },
                    size: data.size,
                    mtime: mtime_for_sort(tree, idx, data.mtime, mtime_sort),
                    entry_count: data.entry_count,
                    exists,
                    is_dir,
                }
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|l, r| match sorting {
        SizeDescending => r.size.cmp(&l.size),
        SizeAscending => l.size.cmp(&r.size),
        MTimeAscending(_) => l.mtime.cmp(&r.mtime),
        MTimeDescending(_) => r.mtime.cmp(&l.mtime),
        CountAscending => cmp_count(l, r),
        CountDescending => cmp_count(l, r).reverse(),
        NameAscending => cmp_name(l, r),
        NameDescending => cmp_name(l, r).reverse(),
    });
    entries
}

fn mtime_for_sort(
    tree: &Tree,
    node_idx: TreeIndex,
    entry_mtime: SystemTime,
    sort: MTimeSort,
) -> SystemTime {
    use MTimeSort::{Entry, RecursiveChildrenNewest, RecursiveChildrenOldest};
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
    let mut stack: Vec<_> = tree.children(node_idx).collect();
    let mut selected_mtime: Option<SystemTime> = None;
    while let Some(idx) = stack.pop() {
        if let Some(entry) = tree.entry(idx) {
            selected_mtime = Some(selected_mtime.map_or(entry.mtime, |selected| {
                if entry.mtime.cmp(&selected) == ordering {
                    entry.mtime
                } else {
                    selected
                }
            }));
            stack.extend(tree.children(idx));
        }
    }
    selected_mtime
}

/// Keep a grapheme-aligned suffix within a terminal-cell budget, adding an ellipsis if shortened.
pub fn fit_string_graphemes_with_ellipsis(
    s: impl Into<String>,
    desired_width: usize,
) -> (String, usize) {
    let s = s.into();
    let width = s.width();
    if width <= desired_width {
        return (s, width);
    }
    if desired_width == 0 {
        return (String::new(), 0);
    }

    let mut suffix_start = s.len();
    let mut width = 1; // ellipsis
    for (index, grapheme) in s.grapheme_indices(true).rev() {
        let grapheme_width = grapheme.width();
        if width + grapheme_width > desired_width {
            break;
        }
        suffix_start = index;
        width += grapheme_width;
    }
    (format!("…{}", &s[suffix_start..]), width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_string_inputs() {
        assert_eq!(
            ("aaa".into(), 3),
            fit_string_graphemes_with_ellipsis("aaa", 4)
        );
        assert_eq!(
            ("…".to_string(), 1),
            fit_string_graphemes_with_ellipsis("abbbba", 1),
            "even amount of chars, desired too small"
        );
        assert_eq!(
            ("…ca".to_string(), 3),
            fit_string_graphemes_with_ellipsis("abbbbca", 3),
            "uneven amount of chars, desired too small"
        );
        assert_eq!(
            ("… a".to_string(), 3),
            fit_string_graphemes_with_ellipsis("a    a", 3),
            "spaces are counted as graphemes, too"
        );
    }

    #[test]
    fn fit_string_respects_display_width_and_grapheme_boundaries() {
        for (input, budget, expected, width) in [
            ("資料abc", 6, "…料abc", 6),
            ("資料abc", 5, "…abc", 4),
            ("👩‍💻abc", 4, "…abc", 4),
            ("cafe\u{301}", 3, "…fe\u{301}", 3),
            ("資料", 1, "…", 1),
            ("資料", 0, "", 0),
            ("資料", 4, "資料", 4),
        ] {
            assert_eq!(
                fit_string_graphemes_with_ellipsis(input, budget),
                (expected.to_string(), width),
                "{input:?} at width {budget}"
            );
        }
    }
}
