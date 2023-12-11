use crate::interactive::path_of;
use dua::traverse::{EntryData, Tree, TreeIndex};
use itertools::Itertools;
use petgraph::Direction;
use std::path::Path;
use std::{cmp::Ordering, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    SizeDescending,
    SizeAscending,
    MTimeDescending,
    MTimeAscending,
    CountDescending,
    CountAscending,
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
            MTimeAscending => MTimeDescending,
            MTimeDescending => MTimeAscending,
            _ => MTimeDescending,
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
}

pub struct EntryDataBundle {
    pub index: TreeIndex,
    pub data: EntryData,
    pub is_dir: bool,
    pub exists: bool,
    pub glob_name: Option<PathBuf>,
}

impl EntryDataBundle {
    pub fn name(&self) -> &Path {
        self.glob_name.as_deref().unwrap_or(&self.data.name)
    }
}

pub fn sorted_entries(
    tree: &Tree,
    node_idx: TreeIndex,
    sorting: SortMode,
    glob_root: Option<TreeIndex>,
) -> Vec<EntryDataBundle> {
    use SortMode::*;
    fn cmp_count(l: &EntryDataBundle, r: &EntryDataBundle) -> Ordering {
        l.data
            .entry_count
            .cmp(&r.data.entry_count)
            .then_with(|| l.data.name.cmp(&r.data.name))
    }
    tree.neighbors_directed(node_idx, Direction::Outgoing)
        .filter_map(|idx| {
            tree.node_weight(idx).map(|w| {
                let mut use_glob_path = false;
                if let Some(glob_root) = glob_root {
                    use_glob_path = node_idx == glob_root;
                }
                let p = path_of(tree, idx, glob_root);
                let pm = p.symlink_metadata();
                EntryDataBundle {
                    index: idx,
                    data: w.clone(),
                    exists: pm.is_ok(),
                    is_dir: pm.ok().map_or(false, |m| m.is_dir()),
                    glob_name: if use_glob_path { Some(p) } else { None },
                }
            })
        })
        .sorted_by(|l, r| match sorting {
            SizeDescending => r.data.size.cmp(&l.data.size),
            SizeAscending => l.data.size.cmp(&r.data.size),
            MTimeAscending => l.data.mtime.cmp(&r.data.mtime),
            MTimeDescending => r.data.mtime.cmp(&l.data.mtime),
            CountAscending => cmp_count(l, r),
            CountDescending => cmp_count(l, r).reverse(),
        })
        .collect()
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
