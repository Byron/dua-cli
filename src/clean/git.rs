use std::{
    collections::{HashMap, hash_map::Entry},
    path::{Path, PathBuf},
};

use anyhow::Context;
use gix::bstr::ByteSlice;

#[derive(Default)]
pub(super) struct Filter {
    repositories: HashMap<PathBuf, gix::Repository>,
}

impl Filter {
    pub(super) fn is_candidate(&mut self, path: &Path) -> anyhow::Result<bool> {
        let path = path.canonicalize()?;
        let Some(repo) = self.repository(&path)? else {
            return Ok(true);
        };
        let Some(workdir) = repo.workdir() else {
            return Ok(false);
        };
        let workdir = workdir.canonicalize()?;
        let relative = path.strip_prefix(&workdir)?;
        if relative.as_os_str().is_empty() {
            return Ok(false);
        }

        // gix-index 0.55 can panic before decoding an index shorter than its checksum.
        match repo.index_path().metadata() {
            Ok(metadata) => anyhow::ensure!(
                metadata.len() >= 12 + repo.object_hash().len_in_bytes() as u64,
                "Truncated Git index in {}",
                workdir.display()
            ),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        // A missing index is also possible in an existing repository, so consult HEAD.
        let index = repo.index_or_load_from_head_or_empty()?;
        let ignore_case = repo.filesystem_options()?.ignore_case;
        let relative_bytes =
            gix::path::to_unix_separators_on_windows(gix::path::into_bstr(relative));
        if has_tracked_paths(&index, &relative_bytes, ignore_case) {
            return Ok(false);
        }
        let mut excludes = repo.excludes(
            &index,
            None,
            gix::worktree::stack::state::ignore::Source::WorktreeThenIdMappingIfNotSkipped,
        )?;
        Ok(matches!(
            excludes
                .at_path(relative, Some(gix::index::entry::Mode::DIR))?
                .excluded_kind(),
            Some(gix::ignore::Kind::Expendable)
        ))
    }

    fn repository(&mut self, path: &Path) -> anyhow::Result<Option<&gix::Repository>> {
        for directory in path.ancestors() {
            let dot_git = directory.join(".git");
            // Discovery normally skips broken .git markers; cleanup must fail closed instead.
            let git_dir = if exists(&dot_git)? {
                dot_git
            } else if exists(&directory.join("HEAD"))? && exists(&directory.join("objects"))? {
                directory.to_owned()
            } else {
                continue;
            };
            let repo = match self.repositories.entry(git_dir) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    use gix::sec::trust::DefaultForLevel;

                    let trust = gix::sec::Trust::from_path_ownership(entry.key())?;
                    let options = gix::open::Options::default_for_level(trust)
                        .open_path_as_is(true)
                        .strict_config(true)
                        .config_overrides(["gitoxide.parsePrecious=true"]);
                    let repo = gix::open_opts(entry.key(), options).with_context(|| {
                        format!("Could not inspect Git repository {}", entry.key().display())
                    })?;
                    entry.insert(repo)
                }
            };
            return Ok(Some(repo));
        }
        Ok(None)
    }
}

fn exists(path: &Path) -> std::io::Result<bool> {
    match path.symlink_metadata() {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn has_tracked_paths(index: &gix::index::State, path: &[u8], ignore_case: bool) -> bool {
    let contains = |entry_path: &[u8], directory: &[u8]| {
        entry_path.get(..directory.len()).is_some_and(|prefix| {
            (if ignore_case {
                prefix.eq_ignore_ascii_case(directory)
            } else {
                prefix == directory
            }) && (entry_path.len() == directory.len()
                || entry_path.get(directory.len()) == Some(&b'/'))
        })
    };
    if ignore_case {
        // ponytail: scan case-folded indexes per candidate; cache folded paths if this becomes a bottleneck.
        return index.entries().iter().any(|entry| {
            let entry_path = entry.path(index).trim_end_with(|c| c == '/');
            contains(entry_path, path)
                || ((entry.mode.is_sparse() || entry.mode.is_submodule())
                    && contains(path, entry_path))
        });
    }
    if index.entry_index_by_path(path.as_bstr()).is_ok() || index.path_is_directory(path.as_bstr())
    {
        return true;
    }
    for (end, byte) in path.iter().enumerate() {
        if *byte != b'/' {
            continue;
        }
        // Sparse directory paths include a trailing slash; inspect every conflict stage.
        for ancestor in [&path[..end], &path[..=end]] {
            if index
                .prefixed_entries(ancestor.as_bstr())
                .is_some_and(|entries| {
                    entries
                        .iter()
                        .take_while(|entry| entry.path(index).as_bytes() == ancestor)
                        .any(|entry| entry.mode.is_sparse() || entry.mode.is_submodule())
                })
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use gix::index::entry::{Flags, Mode, Stage};

    use super::*;

    fn repository() -> Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        fs::create_dir_all(dir.path().join(".git/objects"))?;
        fs::create_dir_all(dir.path().join(".git/refs/heads"))?;
        fs::write(dir.path().join(".git/HEAD"), b"ref: refs/heads/main\n")?;
        fs::write(
            dir.path().join(".git/config"),
            b"[core]\nrepositoryformatversion = 0\nbare = false\nignorecase = false\n",
        )?;
        Ok(dir)
    }

    fn write_index(root: &Path, entries: &[(&str, Mode, Flags)]) -> Result<()> {
        let mut state = gix::index::State::new(gix::hash::Kind::Sha1);
        for (path, mode, flags) in entries {
            state.dangerously_push_entry(
                gix::index::entry::Stat::default(),
                gix::hash::Kind::Sha1.null(),
                *flags,
                *mode,
                (*path).into(),
            );
        }
        state.sort_entries();
        gix::index::File::from_state(state, root.join(".git/index"))
            .write(gix::index::write::Options::default())?;
        Ok(())
    }

    #[test]
    fn requires_ignored_untracked_directories() -> Result<()> {
        let dir = repository()?;
        let root = dir.path();
        fs::write(
            root.join(".gitignore"),
            b"ignored/\ntracked/\nconflicted/\nsubmodule/\nreplaced-file/\nreplaced-conflict/\ntarget/\n$precious/\n.zig-cache/\nzig-out/\n",
        )?;
        write_index(
            root,
            &[
                ("tracked/keep", Mode::FILE, Flags::empty()),
                ("conflicted/keep", Mode::FILE, Stage::Theirs.into()),
                ("submodule", Mode::COMMIT, Flags::empty()),
                ("replaced-file", Mode::FILE, Flags::empty()),
                ("replaced-conflict", Mode::FILE, Stage::Theirs.into()),
                ("target-other/keep", Mode::FILE, Flags::empty()),
                ("zig-out/keep", Mode::FILE, Flags::empty()),
            ],
        )?;
        let mut filter = Filter::default();
        for (name, expected) in [
            ("ignored", true),
            ("target", true),
            ("tracked", false),
            ("conflicted", false),
            ("submodule", false),
            ("replaced-file", false),
            ("replaced-conflict", false),
            ("precious", false),
            ("not-ignored-by-clean", false),
            (".zig-cache", true),
            ("zig-cache", false),
            ("zig-out", false),
        ] {
            let path = root.join(name);
            fs::create_dir(&path)?;
            assert_eq!(filter.is_candidate(&path)?, expected, "{name}");
        }
        assert!(!filter.is_candidate(root)?, "worktree roots are protected");
        let outside = tempfile::tempdir()?;
        assert!(filter.is_candidate(outside.path())?);

        fs::rename(root.join(".git"), root.join("git-data"))?;
        fs::write(
            root.join(".git"),
            format!("gitdir: {}\n", root.join("git-data").display()),
        )?;
        let mut filter = Filter::default();
        assert!(filter.is_candidate(&root.join("ignored"))?);
        assert!(!filter.is_candidate(&root.join("tracked"))?);
        assert!(
            !filter.is_candidate(root)?,
            "gitfile worktrees are protected"
        );
        Ok(())
    }

    #[test]
    fn consults_head_when_the_index_is_missing() -> Result<()> {
        let dir = repository()?;
        let root = dir.path();
        fs::write(root.join(".gitignore"), b"target/\n")?;
        fs::create_dir(root.join("target"))?;
        let repo = gix::open(root)?;
        let blob = repo.write_blob(b"keep")?;
        let tree = repo.write_object(gix::objs::Tree {
            entries: vec![gix::objs::tree::Entry {
                mode: gix::objs::tree::EntryKind::Blob.into(),
                filename: "target".into(),
                oid: blob.detach(),
            }],
        })?;
        let signature = gix::actor::SignatureRef {
            name: "Test".into(),
            email: "test@example.com".into(),
            time: "0 +0000",
        };
        repo.commit_as(
            signature,
            signature,
            "HEAD",
            "fixture",
            tree.detach(),
            std::iter::empty::<gix::ObjectId>(),
        )?;
        assert!(!root.join(".git/index").exists());
        assert!(!Filter::default().is_candidate(&root.join("target"))?);
        Ok(())
    }

    #[test]
    fn protects_case_folded_paths_and_sparse_ancestors() -> Result<()> {
        let dir = repository()?;
        let root = dir.path();
        fs::write(root.join(".gitignore"), b"target/\n")?;
        fs::create_dir_all(root.join("project/target"))?;
        fs::create_dir(root.join("target"))?;
        write_index(
            root,
            &[
                ("TARGET/keep", Mode::FILE, Flags::empty()),
                ("project/", Mode::DIR, Flags::SKIP_WORKTREE),
            ],
        )?;
        for ignore_case in [false, true] {
            fs::write(
                root.join(".git/config"),
                format!(
                    "[core]\nrepositoryformatversion = 0\nbare = false\nignorecase = {ignore_case}\n"
                ),
            )?;
            let mut filter = Filter::default();
            assert_eq!(filter.is_candidate(&root.join("target"))?, !ignore_case);
            assert!(!filter.is_candidate(&root.join("project/target"))?);
        }
        Ok(())
    }

    #[test]
    fn protects_submodule_descendants_without_a_git_marker() -> Result<()> {
        let dir = repository()?;
        let root = dir.path();
        fs::write(root.join(".gitignore"), b"node_modules/\n")?;
        fs::create_dir_all(root.join("sub/node_modules"))?;
        fs::create_dir_all(root.join("sub-other/node_modules"))?;
        assert!(!root.join("sub/.git").exists());
        for ignore_case in [false, true] {
            fs::write(
                root.join(".git/config"),
                format!(
                    "[core]\nrepositoryformatversion = 0\nbare = false\nignorecase = {ignore_case}\n"
                ),
            )?;
            for stage in [Stage::Unconflicted, Stage::Base, Stage::Ours, Stage::Theirs] {
                let path = if ignore_case { "SUB" } else { "sub" };
                write_index(root, &[(path, Mode::COMMIT, stage.into())])?;
                let mut filter = Filter::default();
                assert!(
                    !filter.is_candidate(&root.join("sub/node_modules"))?,
                    "gitlink ancestor at {stage:?}, ignore_case={ignore_case}"
                );
                assert!(filter.is_candidate(&root.join("sub-other/node_modules"))?);
            }
        }
        Ok(())
    }

    #[test]
    fn git_failures_do_not_fall_back_to_name_heuristics() -> Result<()> {
        let dir = repository()?;
        let root = dir.path();
        fs::write(root.join(".gitignore"), b"target/\n")?;
        fs::create_dir(root.join("target"))?;
        fs::write(root.join(".git/index"), b"broken index")?;
        assert!(
            Filter::default()
                .is_candidate(&root.join("target"))
                .is_err()
        );

        #[cfg(unix)]
        {
            fs::remove_file(root.join(".git/index"))?;
            fs::remove_file(root.join(".gitignore"))?;
            std::os::unix::fs::symlink(".gitignore", root.join(".gitignore"))?;
            assert!(
                Filter::default()
                    .is_candidate(&root.join("target"))
                    .is_err()
            );
        }

        let outside = tempfile::tempdir()?;
        fs::write(outside.path().join(".git"), b"not a gitdir file")?;
        fs::create_dir(outside.path().join("target"))?;
        assert!(
            Filter::default()
                .is_candidate(&outside.path().join("target"))
                .is_err()
        );
        Ok(())
    }
}
