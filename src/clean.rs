use std::{
    collections::{BTreeSet, VecDeque},
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

mod git;

/// Conservatively protect Git markers in every ASCII case, including on case-sensitive disks.
pub(crate) fn is_git_dir_name(name: &OsStr) -> bool {
    name.as_encoded_bytes().eq_ignore_ascii_case(b".git")
}

/// Return whether a directory name is a known cleanup candidate.
#[must_use]
pub fn is_cleanup_dir_name(name: &OsStr) -> bool {
    [
        ".mypy_cache",
        ".pytest_cache",
        ".ruff_cache",
        ".tox",
        ".venv",
        ".zig-cache",
        "__pycache__",
        "node_modules",
        "target",
        "venv",
        "zig-cache",
        "zig-out",
    ]
    .binary_search_by(|candidate| OsStr::new(candidate).cmp(name))
    .is_ok()
}

pub(crate) enum DiscoveryEvent {
    Candidate { path: PathBuf, search_root: PathBuf },
    Progress { entries: u64, io_errors: u64 },
}

pub(crate) fn discover(
    input: Vec<PathBuf>,
    walk_options: crate::WalkOptions,
    depth: Option<usize>,
    pattern_root: Option<PathBuf>,
    mut send: impl FnMut(DiscoveryEvent) -> bool,
) {
    let mut progress = Progress::default();
    if !progress.send(&mut send, true) {
        return;
    }
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            progress.error(Path::new("."), err);
            progress.send(&mut send, true);
            return;
        }
    };
    let input = if input.is_empty() {
        vec![cwd.clone()]
    } else {
        input
    };
    let pattern_root = pattern_root.map(|root| {
        root.canonicalize().unwrap_or_else(|_| {
            gix::path::normalize(root.as_path().into(), &cwd)
                .map_or_else(|| root.clone(), |path| path.into_owned())
        })
    });
    let mut roots = Vec::new();
    for path in input {
        progress.entries += 1;
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() => match path.canonicalize() {
                Ok(path) => roots.push(path),
                Err(err) => progress.error(&path, err),
            },
            Ok(_) => {}
            Err(err) => progress.error(&path, err),
        }
        if !progress.send(&mut send, false) {
            return;
        }
    }
    roots.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| left.cmp(right))
    });
    roots.dedup();
    let mut searched = Vec::<PathBuf>::new();
    let mut candidates = BTreeSet::<PathBuf>::new();
    let mut git = git::Filter::default();
    // ponytail: one discovery reader; use directory workers if enumeration becomes the bottleneck.
    for root in roots {
        if root
            .components()
            .any(|component| is_git_dir_name(component.as_os_str()))
        {
            continue;
        }
        if depth.is_none() && searched.iter().any(|parent| root.starts_with(parent)) {
            continue;
        }
        searched.push(root.clone());
        let pattern_root = pattern_root.as_ref().unwrap_or(&root);
        let device = if walk_options.cross_filesystems {
            0
        } else {
            match crate::crossdev::init(&root) {
                Ok(device) => device,
                Err(err) => {
                    progress.error(&root, err);
                    continue;
                }
            }
        };
        // Finish an ancestor input before nested inputs, preserving each input's depth budget.
        let mut pending = VecDeque::from([(root.clone(), 0, None)]);
        while let Some((path, level, parent_has_cargo)) = pending.pop_front() {
            if !progress.send(&mut send, false) {
                return;
            }
            if path.file_name().is_some_and(is_git_dir_name)
                || path
                    .ancestors()
                    .any(|ancestor| candidates.contains(ancestor))
                || is_excluded(&path, pattern_root, &cwd, &walk_options)
            {
                continue;
            }
            if !walk_options.cross_filesystems {
                match crate::crossdev::init(&path) {
                    Ok(found) if found != device => continue,
                    Ok(_) => {}
                    Err(err) => {
                        progress.error(&path, err);
                        continue;
                    }
                }
            }
            let name = path.file_name().unwrap_or_default();
            let qualifies = is_cleanup_dir_name(name)
                && if name == "target" {
                    parent_has_cargo.unwrap_or_else(|| {
                        path.parent().is_some_and(|parent| {
                            has_regular_file(parent, "Cargo.toml", &mut progress, &mut send)
                        })
                    })
                } else if name == ".venv" || name == "venv" {
                    has_regular_file(&path, "pyvenv.cfg", &mut progress, &mut send)
                } else {
                    true
                };
            if progress.cancelled {
                return;
            }
            if qualifies {
                match git.is_candidate(&path) {
                    Ok(true) => {
                        candidates.insert(path.clone());
                        if !send(DiscoveryEvent::Candidate {
                            path,
                            search_root: pattern_root.clone(),
                        }) {
                            return;
                        }
                        continue;
                    }
                    Ok(false) => {}
                    Err(err) => {
                        progress.error(&path, err);
                        continue;
                    }
                }
            }
            if depth.is_some_and(|limit| level >= limit) {
                continue;
            }
            let Some(entries) = read_directory(&path, &mut progress, &mut send) else {
                if progress.cancelled {
                    return;
                }
                continue;
            };
            // The complete batch identifies repository markers before processing any children.
            // Bare repository contents are administrative, whereas worktrees remain searchable.
            let has = |name: &str, directory: bool| {
                entries.iter().any(|(entry_name, kind)| {
                    entry_name == name
                        && if directory {
                            kind.is_dir()
                        } else {
                            kind.is_file()
                        }
                })
            };
            if has("HEAD", false) && has("objects", true) && has("refs", true) {
                continue;
            }
            let has_cargo = has("Cargo.toml", false);
            pending.extend(entries.into_iter().filter_map(|(name, kind)| {
                (kind.is_dir() && !is_git_dir_name(&name))
                    .then(|| (path.join(name), level + 1, Some(has_cargo)))
            }));
        }
    }
    progress.send(&mut send, true);
}

fn is_excluded(path: &Path, root: &Path, cwd: &Path, options: &crate::WalkOptions) -> bool {
    options
        .ignore_dirs
        .iter()
        .any(|ignored| path.starts_with(ignored))
        || options.ignore_patterns.as_ref().is_some_and(|patterns| {
            crate::pattern_relative_path(path, cwd, root)
                .is_some_and(|relative| patterns.is_excluded(relative, true))
        })
}

fn has_regular_file(
    path: &Path,
    name: &str,
    progress: &mut Progress,
    send: &mut impl FnMut(DiscoveryEvent) -> bool,
) -> bool {
    read_directory(path, progress, send).is_some_and(|entries| {
        entries
            .iter()
            .any(|(entry_name, kind)| entry_name == name && kind.is_file())
    })
}

fn read_directory(
    path: &Path,
    progress: &mut Progress,
    send: &mut impl FnMut(DiscoveryEvent) -> bool,
) -> Option<Vec<(OsString, fs::FileType)>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            progress.error(path, err);
            return None;
        }
    };
    let mut result = Vec::new();
    for entry in entries {
        progress.entries += 1;
        match entry.and_then(|entry| entry.file_type().map(|kind| (entry.file_name(), kind))) {
            Ok((name, kind)) => {
                if kind.is_dir()
                    || matches!(
                        name.to_str(),
                        Some("Cargo.toml" | "pyvenv.cfg" | "HEAD" | ".git")
                    )
                {
                    result.push((name, kind));
                }
            }
            Err(err) => progress.error(path, err),
        }
        if progress.entries.is_multiple_of(128) && !progress.send(send, false) {
            return None;
        }
    }
    Some(result)
}

struct Progress {
    entries: u64,
    io_errors: u64,
    last_sent: Instant,
    cancelled: bool,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            entries: 0,
            io_errors: 0,
            last_sent: Instant::now(),
            cancelled: false,
        }
    }
}

impl Progress {
    fn error(&mut self, path: &Path, error: impl std::fmt::Display) {
        self.io_errors += 1;
        log::warn!("Cannot search {}: {error}", path.display());
    }

    fn send(&mut self, send: &mut impl FnMut(DiscoveryEvent) -> bool, force: bool) -> bool {
        if force || self.last_sent.elapsed() >= Duration::from_millis(100) {
            self.cancelled = !send(DiscoveryEvent::Progress {
                entries: std::mem::take(&mut self.entries),
                io_errors: std::mem::take(&mut self.io_errors),
            });
            self.last_sent = Instant::now();
        }
        !self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, path::Path};

    use super::*;

    fn options() -> crate::WalkOptions {
        crate::WalkOptions {
            threads: 1,
            count_hard_links: false,
            apparent_size: true,
            cross_filesystems: true,
            ignore_dirs: BTreeSet::new(),
            ignore_patterns: None,
            metadata_options: crate::TraversalOptions::default(),
        }
    }

    fn scan(
        input: Vec<PathBuf>,
        options: crate::WalkOptions,
        depth: Option<usize>,
    ) -> (Vec<PathBuf>, u64, u64) {
        let mut paths = Vec::new();
        let (mut entries, mut errors) = (0, 0);
        discover(input, options, depth, None, |event| {
            match event {
                DiscoveryEvent::Candidate { path, .. } => paths.push(path),
                DiscoveryEvent::Progress {
                    entries: count,
                    io_errors,
                } => {
                    entries += count;
                    errors += io_errors;
                }
            }
            true
        });
        paths.sort();
        (paths, entries, errors)
    }

    fn directory(root: &Path, path: &str) -> PathBuf {
        let path = root.join(path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn matches_cleanup_names() {
        for name in [
            ".mypy_cache",
            ".pytest_cache",
            ".ruff_cache",
            ".tox",
            ".venv",
            ".zig-cache",
            "__pycache__",
            "node_modules",
            "target",
            "venv",
            "zig-cache",
            "zig-out",
        ] {
            assert!(is_cleanup_dir_name(OsStr::new(name)), "{name}");
        }
        for name in ["build", "dist", "Target", ".git", "project/target"] {
            assert!(!is_cleanup_dir_name(OsStr::new(name)), "{name}");
        }
    }

    #[test]
    fn discovers_zig_caches_and_build_outputs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("build.zig"), "").unwrap();
        fs::write(root.join("build.zig.zon"), "").unwrap();
        let mut expected = Vec::new();
        for name in [".zig-cache", "zig-cache", "zig-out"] {
            let path = directory(&root, name);
            directory(&path, "nested/.zig-cache");
            expected.push(path);
        }
        let (paths, _, errors) = scan(vec![root], options(), None);
        assert_eq!(paths, expected);
        assert_eq!(errors, 0);
    }

    #[test]
    fn discovers_qualified_directories_and_prunes_claimed_subtrees() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let node = directory(&root, "web/node_modules");
        directory(&node, "nested/node_modules");
        let target = directory(&root, "rust/target");
        fs::write(root.join("rust/Cargo.toml"), "").unwrap();
        let venv = directory(&root, "python/.venv");
        fs::write(venv.join("pyvenv.cfg"), "").unwrap();
        let cache = directory(&root, "misc/__pycache__");
        let below_target = directory(&root, "ordinary/target/nested/node_modules");
        let below_venv = directory(&root, "ordinary/venv/nested/node_modules");
        directory(&root, "ordinary/venv/pyvenv.cfg");
        directory(&root, "metadata/.git/node_modules");
        fs::write(root.join("target"), "a file is not a candidate").unwrap();
        assert_eq!(
            scan(vec![target.clone()], options(), Some(0)).0,
            vec![target.clone()]
        );
        let (paths, entries, errors) = scan(vec![root], options(), None);
        let mut expected = vec![node, target, venv, cache, below_target, below_venv];
        expected.sort();
        assert_eq!(paths, expected);
        assert!(entries > paths.len() as u64);
        assert_eq!(errors, 0);
    }

    #[test]
    fn depth_limits_discovery_and_an_input_candidate_has_depth_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let direct = directory(&root, "node_modules");
        let nested = directory(&root, "project/__pycache__");
        assert!(scan(vec![root.clone()], options(), Some(0)).0.is_empty());
        assert_eq!(
            scan(vec![root.clone()], options(), Some(1)).0,
            vec![direct.clone()]
        );
        let mut expected = vec![direct.clone(), nested.clone()];
        expected.sort();
        assert_eq!(
            scan(vec![root.clone(), root.join("project")], options(), Some(1)).0,
            expected
        );
        assert_eq!(scan(vec![root], options(), None).0, expected);
        assert_eq!(
            scan(vec![direct.clone()], options(), Some(0)).0,
            vec![direct]
        );
    }

    #[test]
    fn overlapping_inputs_do_not_duplicate_or_nest_candidates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let artifact = directory(&root, "project/node_modules");
        let nested = directory(&artifact, "inner/node_modules");
        assert_eq!(
            scan(
                vec![nested, artifact.clone(), root.clone(), root.join(".")],
                options(),
                None
            )
            .0,
            vec![artifact]
        );
    }

    #[test]
    fn exclusions_keep_the_original_search_root_scope() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        directory(&root, "node_modules");
        let nested = directory(&root, "project/node_modules");
        directory(&root, "ignored/__pycache__");
        directory(&root, "bare.git/objects");
        directory(&root, "bare.git/refs");
        fs::write(root.join("bare.git/HEAD"), "ref: refs/heads/main\n").unwrap();
        directory(&root, "bare.git/node_modules");
        let patterns = root.join("patterns");
        fs::write(&patterns, "/node_modules/\n").unwrap();
        let mut opts = options();
        opts.ignore_patterns = crate::IgnorePatterns::from_files(&[patterns]).unwrap();
        opts.ignore_dirs.insert(root.join("ignored"));
        assert_eq!(scan(vec![root], opts, None).0, vec![nested]);
    }

    #[test]
    fn refresh_uses_the_supplied_discovery_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let artifact = directory(&root, "project/node_modules");
        let project = root.join("project");
        let patterns = root.join("patterns");
        fs::write(&patterns, "/node_modules/\n").unwrap();
        let mut opts = options();
        opts.ignore_patterns = crate::IgnorePatterns::from_files(&[patterns]).unwrap();
        let mut found = false;
        discover(
            vec![artifact.clone()],
            opts.clone(),
            None,
            Some(project),
            |event| {
                found |= matches!(event, DiscoveryEvent::Candidate { .. });
                true
            },
        );
        assert!(!found, "refresh must not change the anchored pattern base");
        discover(
            vec![artifact.clone()],
            opts,
            None,
            Some(root.clone()),
            |event| {
                if let DiscoveryEvent::Candidate { path, search_root } = event {
                    assert_eq!(path, artifact);
                    assert_eq!(search_root, root);
                    found = true;
                }
                true
            },
        );
        assert!(found);
    }

    #[test]
    fn explicit_paths_inside_git_administration_are_pruned() {
        for marker in [".git", ".GIT", ".Git"] {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path().canonicalize().unwrap();
            directory(&root, &format!("{marker}/objects/node_modules"));
            let (paths, entries, errors) =
                scan(vec![root.join(marker).join("objects")], options(), None);
            assert!(paths.is_empty());
            assert_eq!(entries, 1);
            assert_eq!(errors, 0);
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_and_symlink_markers_are_not_followed() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let real = directory(&root, "real/__pycache__");
        symlink(root.join("real"), root.join("node_modules")).unwrap();
        directory(&root, "rust/target");
        fs::write(root.join("manifest"), "").unwrap();
        symlink(root.join("manifest"), root.join("rust/Cargo.toml")).unwrap();
        let venv = directory(&root, "python/venv");
        symlink(root.join("manifest"), venv.join("pyvenv.cfg")).unwrap();
        assert_eq!(scan(vec![root.clone()], options(), None).0, vec![real]);
        assert!(
            scan(vec![root.join("node_modules")], options(), None)
                .0
                .is_empty()
        );
    }

    #[test]
    fn progress_reports_empty_search_errors_and_allows_cancellation() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("ordinary-file"), "").unwrap();
        let (paths, entries, errors) =
            scan(vec![root.clone(), root.join("missing")], options(), None);
        assert!(paths.is_empty());
        assert!(entries > 0);
        assert_eq!(errors, 1);
        directory(&root, "node_modules");
        let mut calls = 0;
        discover(vec![root], options(), None, None, |_| {
            calls += 1;
            false
        });
        assert_eq!(calls, 1);
    }
}
