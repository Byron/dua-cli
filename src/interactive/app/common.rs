use crate::interactive::path_of;
use dua::traverse::{EntryData, Tree, TreeIndex};
use itertools::Itertools;
use petgraph::Direction;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    SizeDescending,
    SizeAscending,
    Alphabetically,
}

impl SortMode {
    pub fn toggle_size(&mut self) {
        use SortMode::*;
        *self = match self {
            SizeAscending => Alphabetically,
            SizeDescending => SizeAscending,
            Alphabetically => SizeDescending,
        }
    }
}

pub struct EntryDataBundle {
    pub index: TreeIndex,
    pub data: EntryData,
    pub is_dir: bool,
    pub exists: bool,
}

pub fn sorted_entries(
    tree: &Tree,
    node_idx: TreeIndex,
    sorting: SortMode,
    allow_disk_access: bool,
) -> Vec<EntryDataBundle> {
    use SortMode::*;
    tree.neighbors_directed(node_idx, Direction::Outgoing)
        .filter_map(|idx| {
            tree.node_weight(idx).map(|w| {
                let p = path_of(tree, idx);
                let pm = allow_disk_access
                    .then(|| p.symlink_metadata().ok())
                    .flatten();
                EntryDataBundle {
                    index: idx,
                    data: w.clone(),
                    exists: !allow_disk_access || pm.is_some(),
                    is_dir: pm.map_or_else(
                        || {
                            tree.neighbors_directed(idx, Direction::Outgoing)
                                .next()
                                .is_some()
                        },
                        |m| m.is_dir(),
                    ),
                }
            })
        })
        .sorted_by(|l, r| match sorting {
            SizeDescending => r.data.size.cmp(&l.data.size),
            SizeAscending => l.data.size.cmp(&r.data.size),
            Alphabetically => l.data.name.cmp(&r.data.name),
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
