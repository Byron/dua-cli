# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 2.43.1 (2026-08-29)

### Bug Fixes

 - <csr-id-1f2575a9d176c03701034a88e1e2737fc2c90c1f/> stream folded stacks during traversal
   <!-- agent -->
   Unlimited-depth aggregate --stack retained every traversal node and emitted only
   after walking, allowing memory to grow with entry count.
   
   Write each exclusive-size line as its parent-first event is integrated and
   retain only root accounting. Keep depth-limited mode unchanged because it must
   roll hidden descendants into visible frames.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#382](https://github.com/Byron/dua-cli/issues/382)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#382](https://github.com/Byron/dua-cli/issues/382)**
    - Stream folded stacks during traversal ([`1f2575a`](https://github.com/Byron/dua-cli/commit/1f2575a9d176c03701034a88e1e2737fc2c90c1f))
 * **Uncategorized**
    - Merge pull request #384 from Byron/stacks-memory ([`5cad619`](https://github.com/Byron/dua-cli/commit/5cad6197066cef8e74a0decd86d5da61261eef76))
</details>

## 2.43.0 (2026-08-25)

Thanks to our contributors, there are not one, but two (a joke, a human is writing this with fleshy fingers :D) headline features:

1. `dua aggregate --depth N` shows an indented tree-like view, instead of a flat list
2. `dua aggregate --stack | inferno-flamegraph > fire.svg` outputs a flame-graph compatible stack listing, for pretty visualisations of the directory. It also works with `--depth N`.

On the side, on macOS, the bytes output of `dua` now matches `du` byte-perfectly, and for the first time it's possible to pipe `dua`
output into a file without terminal escape sequences (*why did this take me so long again?*).

### New Features

 - <csr-id-74bf377839b840f157cb2871c7435faf4070a509/> Add `dua aggregate --depth N` for indented tree output

### Bug Fixes

 - <csr-id-b1e16a0cc5ea3792b477f5b38a2d98e88b4f53c3/> don't output colors when stdout isn't a terminal.
   This facilitates piping into a file.
 - <csr-id-dd9e0cec2cf4488977c5b2cf2283f7a116929124/> --stats printed the u128 sentinel as the smallest file size
   The sentinel reset was guarded by entries_traversed == 0, but an entry
   whose metadata cannot be read still counts as traversed while never
   contributing a size. Reset when the sentinel is still in place instead.
 - <csr-id-f9b6200c896d2737050dca725fb1659e778b1627/> Apply --ignore-dirs to prepared aggregate roots
 - <csr-id-72faa62bd54b65864c650d0e44983b1ea74c9df2/> Apply --ignore-dirs to paths dua expanded into roots itself

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 28 commits contributed to the release over the course of 9 calendar days.
 - 10 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #378 from ChrisJr404/feature/stacks-folded-output ([`886f3ce`](https://github.com/Byron/dua-cli/commit/886f3cece7af7cdabadba104cb0e93551103f86e))
    - Fix folded stack path assertions on Windows ([`52df4c5`](https://github.com/Byron/dua-cli/commit/52df4c5ce9282ef1ecdfd0c8ba81ee25f5cc9966))
    - Progress support for `--stack` and `--depth N` ([`e476971`](https://github.com/Byron/dua-cli/commit/e47697186419d425dc78a3a764af81ff955533e3))
    - Review ([`a14f2b7`](https://github.com/Byron/dua-cli/commit/a14f2b799120b0f49dd3d00dee1fa05f080492b5))
    - Add 'stacks' subcommand emitting folded stacks for flame graphs ([`d40a44d`](https://github.com/Byron/dua-cli/commit/d40a44dcdca048db89e8ffc79f6699d48fda8957))
    - Merge pull request #380 from ChrisJr404/feature/aggregate-tree-depth ([`7cfa98c`](https://github.com/Byron/dua-cli/commit/7cfa98ca644d0dae848af0b935d2628bbc2c6e29))
    - Review ([`136a86a`](https://github.com/Byron/dua-cli/commit/136a86aa836762b2262ec0535f4f3765d555c765))
    - Add `dua aggregate --depth N` for indented tree output ([`74bf377`](https://github.com/Byron/dua-cli/commit/74bf377839b840f157cb2871c7435faf4070a509))
    - Merge pull request #373 from VXNCXNX/fix/root-path-ending-in-dotdot ([`c4cda95`](https://github.com/Byron/dua-cli/commit/c4cda95c7248ec41738d7c7ff763c343fe326350))
    - Don't output colors when stdout isn't a terminal. ([`b1e16a0`](https://github.com/Byron/dua-cli/commit/b1e16a0cc5ea3792b477f5b38a2d98e88b4f53c3))
    - Merge pull request #375 from tamird/perf-skip-apfs-apparent-size ([`06b4237`](https://github.com/Byron/dua-cli/commit/06b4237f9163856c2f95d2af8fa362c5a19e2aac))
    - Review ([`511bb35`](https://github.com/Byron/dua-cli/commit/511bb35a77938bb507e4dbdeeb3b8ec65ca92f85))
    - Merge pull request #372 from VXNCXNX/fix/smallest-file-sentinel-leak ([`95ab634`](https://github.com/Byron/dua-cli/commit/95ab63420e5a1fc950a8ae8cfb76f47171f9c81b))
    - Merge pull request #374 from tamird/perf-skip-empty-ignore-realpath ([`83d467a`](https://github.com/Byron/dua-cli/commit/83d467ae55025b71ea8964997ec872dc37c8e0db))
    - Review ([`5bf9204`](https://github.com/Byron/dua-cli/commit/5bf92044dbaf1fe6b617f401d2e61bab1d5e7ace))
    - Review ([`de9f8a0`](https://github.com/Byron/dua-cli/commit/de9f8a0b6937d12b3402f88b3c2e23fdf3e14327))
    - Merge pull request #376 from tamird/perf-stream-prepared-roots ([`bed32a4`](https://github.com/Byron/dua-cli/commit/bed32a42acec620b61f0b2fc17248a0d9b355630))
    - Skip path resolution without ignored directories ([`6c1c511`](https://github.com/Byron/dua-cli/commit/6c1c511a2293e320ef59ac86c8dd44ed81ed066c))
    - Avoid materiailizing vector ([`dbf33e6`](https://github.com/Byron/dua-cli/commit/dbf33e64d43f70bdff3234cccf873b2cfcf812ee))
    - Skip clone metadata for apparent sizes ([`18f6395`](https://github.com/Byron/dua-cli/commit/18f6395bf31e0419f22d8b52b9de6501dc50393c))
    - --stats printed the u128 sentinel as the smallest file size ([`dd9e0ce`](https://github.com/Byron/dua-cli/commit/dd9e0cec2cf4488977c5b2cf2283f7a116929124))
    - Merge pull request #371 from tamird/macos-apfs-clone-accounting ([`7231d83`](https://github.com/Byron/dua-cli/commit/7231d838d6dad0f4a2ac649959788ba6ec844853))
    - Review ([`0acd5fa`](https://github.com/Byron/dua-cli/commit/0acd5fa421d287a1a8ef8c6d6efc8244e0622940))
    - Count fully shared APFS clones once ([`6b3a231`](https://github.com/Byron/dua-cli/commit/6b3a23176f69f5f18b3eb87529d585dfb2f5b9cf))
    - Merge pull request #370 from VXNCXNX/fix/ignore-dirs-on-expanded-roots ([`cfa5ed2`](https://github.com/Byron/dua-cli/commit/cfa5ed28b255a5359fbe0145db01888a63a264c5))
    - Apply --ignore-dirs to prepared aggregate roots ([`f9b6200`](https://github.com/Byron/dua-cli/commit/f9b6200c896d2737050dca725fb1659e778b1627))
    - Review ([`1c41b46`](https://github.com/Byron/dua-cli/commit/1c41b463ec28f54a3ab9907c88c419adb6c03a4e))
    - Apply --ignore-dirs to paths dua expanded into roots itself ([`72faa62`](https://github.com/Byron/dua-cli/commit/72faa62bd54b65864c650d0e44983b1ea74c9df2))
</details>

## 2.42.1 (2026-08-15)

Starting directories with a large amount of files, like 50k, now see a 5x speedup on macOS and Windows as bulk-reading is also done there. Note also that this is still a small absolute difference, 100ms vs 500ms, but a good demonstration of how much large trees with a lot of such directories will benefit  by this, as these small absolute improvements accumulate.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #369 from tamird/macos-prepared-roots ([`3b1659e`](https://github.com/Byron/dua-cli/commit/3b1659e47ae28b3facd1d7d261132023b2509d1a))
    - Review ([`ffb2a2e`](https://github.com/Byron/dua-cli/commit/ffb2a2ef0fc3b95c4d2e0ef07f228699aa750a44))
    - Reuse bulk metadata for macOS aggregation roots ([`d45a2f8`](https://github.com/Byron/dua-cli/commit/d45a2f85451d900b19770f1e64bddd201e9d7429))
</details>

## 2.42.0 (2026-08-14)

The headline or this release is ~30% better traversal performance on macOS due to the usage
of bulk-metadata APIs on supported filesytems.

### Bug Fixes

 - <csr-id-a2d375d7fdb4eecc287ad87715a3f021951e3e6d/> sanitize control characters in marked path output
   dua-cli's interactive TUI is built on ratatui, which protects the
   paths it renders on screen. But marking a file for deletion and then
   quitting prints that file's path directly to the terminal after the
   TUI has already released terminal control, bypassing ratatui's
   protective rendering entirely. A scanned file's name has no character
   restrictions, so a crafted file name can inject terminal escape
   sequences into the printed path.
   
   Add sanitize_for_display (src/main.rs), replacing control characters
   with the Unicode replacement character, applied at the sole print site
   for marked paths.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 9 calendar days.
 - 10 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #367 from tamird/macos-native-traversal ([`e68868c`](https://github.com/Byron/dua-cli/commit/e68868c1fd28d089baa91126998dc25fb5216e23))
    - Review ([`872b6be`](https://github.com/Byron/dua-cli/commit/872b6beffaf3f354dee3e1f670d4fe45615b76a8))
    - Avoid per-entry macOS metadata queries ([`7d115c0`](https://github.com/Byron/dua-cli/commit/7d115c014eff2f85ab60e82266f4f1beb4782598))
    - Merge pull request #366 from carfeii/fix/sanitize-marked-path-output ([`f515d36`](https://github.com/Byron/dua-cli/commit/f515d36174efeba2cdc2148fbb684f201df9ddad))
    - Review ([`8521d0d`](https://github.com/Byron/dua-cli/commit/8521d0d0231a6e95b7b9cc350b2eb2fc570a2833))
    - Sanitize control characters in marked path output ([`a2d375d`](https://github.com/Byron/dua-cli/commit/a2d375d7fdb4eecc287ad87715a3f021951e3e6d))
    - Merge pull request #362 from Byron/dua-lib ([`b6e7caf`](https://github.com/Byron/dua-cli/commit/b6e7cafd305c150834eb887e1de99bcdd3fca85d))
    - Extract filesystem walker into dua-lib ([`3b1c8cf`](https://github.com/Byron/dua-cli/commit/3b1c8cfbf206d92f60a33049dd741251024a027f))
</details>

## 2.41.1 (2026-08-04)

A one-line change that makes all the difference: Windows TUI performance isn't sluggish anymore as the drawing is now buffered!

### Bug Fixes

 - <csr-id-bccddb7e78965e8bd06c83de6094bdd92ff5e81a/> buffer terminal frame output

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #361 from larsch/fix/buffer-tui-output ([`c05f608`](https://github.com/Byron/dua-cli/commit/c05f608107e9a1a4e2a36adf5552f7efa0d46440))
    - Buffer terminal frame output ([`bccddb7`](https://github.com/Byron/dua-cli/commit/bccddb7e78965e8bd06c83de6094bdd92ff5e81a))
    - Merge pull request #360 from Byron/windows-performance ([`f73cc51`](https://github.com/Byron/dua-cli/commit/f73cc5113b9f334287e42169d781d70b5f7c50af))
    - Preserve Windows verbatim paths ([`381824f`](https://github.com/Byron/dua-cli/commit/381824fac7c5d457a0b360156b7831f1b73281a0))
</details>

## 2.41.0 (2026-08-03)

There are two major features: 30x and more performance on Windows, and `--ignore-from <file>` support.
This makes this release the best one yet, and I do hope that I can last a week or more until the next one.

### New Features

 - <csr-id-27d16f0aed7d26efdb8af50e82b30a709d8be19c/> add --ignore-from to exclude paths with gitignore-style patterns
   Reads gitignore-syntax patterns from one or more files and leaves everything
   they match out of the report, in both aggregate and interactive mode. This is
   the equivalent of rsync's --exclude-from and restic's --exclude-file, so the
   same pattern file can answer "how much of this would actually be backed up?".
   
   Matching is powered by gix-ignore, which is already a dependency, so negation,
   anchoring, `**` and directory-only patterns all behave exactly like Git.
   Excluded directories are pruned from the walk rather than only hidden, and
   excluded top-level paths are dropped before the walk so they are absent from
   the report instead of appearing as empty.

### Bug Fixes

 - <csr-id-b4d944ea98da33ac10602fbd4f36b1bb2bc4859e/> propagate background root device errors
   <!-- agent -->
   Background traversal replaced failed root-device lookups with device ID
   zero. Readable root metadata could then be rejected as cross-device while the
   traversal incorrectly retained a successful error count.
   
   Skip roots whose lookup fails and carry their per-root errors in the traversal
   completion event, restoring the previous statistics and exit status. Keep
   interleaved overlapping and duplicate roots isolated by including each root path
   Arc allocation in private directory-map keys while preserving public traversal
   event types.
 - <csr-id-01d7e00a0e55499d7c8696f49f764c5145a48fd8/> propagate aggregate root device errors
   <!-- agent -->
   A failed root-device lookup was replaced with device ID zero after the parallel
   traversal rewrite. For a dangling symlink, symlink metadata remained readable
   but failed the device check, hiding the initialization error and producing a
   successful exit status.
   
   Count the lookup failure on its root, mark that root complete for ordered
   output, and omit it from traversal. Flush completed roots after traversal
   because failed roots emit no completion event.
   
   Keep all roots in the shared
   parallel traversal pool; when hard links are deduplicated, per-root attribution
   is scheduling-dependent while the total remains correct, in the name of performance.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release.
 - 2 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#302](https://github.com/Byron/dua-cli/issues/302)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#302](https://github.com/Byron/dua-cli/issues/302)**
    - Avoid per-entry Windows metadata queries ([`9831b1e`](https://github.com/Byron/dua-cli/commit/9831b1ed6870d0cf7683f59816f212b3328f5f20))
 * **Uncategorized**
    - Merge pull request #359 from Byron/windows-performance ([`bdee013`](https://github.com/Byron/dua-cli/commit/bdee013b32fe7aa838044dfa3031610fe4ce39a9))
    - Review ([`d784e02`](https://github.com/Byron/dua-cli/commit/d784e024a6cf40ee6d67fd27c6becc45dca54c13))
    - Simplify platform-specific walker entries ([`f94fe6d`](https://github.com/Byron/dua-cli/commit/f94fe6db10e53171ce1b917ac5fb126d56c5d18d))
    - Address CI comment about reader type complexity ([`542e36a`](https://github.com/Byron/dua-cli/commit/542e36a0a6b2f33422c4e86cc556ffa792955a4e))
    - Use Result::is_ok_and in entry checks ([`29b9c8c`](https://github.com/Byron/dua-cli/commit/29b9c8c15272f0ca1c1b8db83a5608fc8bc533bd))
    - Optimize Windows directory metadata scheduling ([`e36a66d`](https://github.com/Byron/dua-cli/commit/e36a66d182df56be38e9e66d813bcfce07fe3794))
    - Merge pull request #358 from pelazas/feat/ignore-from ([`1db53ae`](https://github.com/Byron/dua-cli/commit/1db53ae58d14c4368a12063bbad00eb6420e6949))
    - Review ([`af83aa5`](https://github.com/Byron/dua-cli/commit/af83aa50f78d038f02857b182a1e7895243a10fc))
    - Add --ignore-from to exclude paths with gitignore-style patterns ([`27d16f0`](https://github.com/Byron/dua-cli/commit/27d16f0aed7d26efdb8af50e82b30a709d8be19c))
    - Merge pull request #357 from Byron/crossdev-fix ([`d7b1503`](https://github.com/Byron/dua-cli/commit/d7b1503a8102f123cc3eb9ffbffdb1e906907d98))
    - Propagate background root device errors ([`b4d944e`](https://github.com/Byron/dua-cli/commit/b4d944ea98da33ac10602fbd4f36b1bb2bc4859e))
    - Propagate aggregate root device errors ([`01d7e00`](https://github.com/Byron/dua-cli/commit/01d7e00a0e55499d7c8696f49f764c5145a48fd8))
</details>

## 2.40.1 (2026-08-01)

Even more performance, up to 15%. That's it now, trying to hit 1 week without a release ;).

### Bug Fixes

 - <csr-id-3b7415b9fa161ef20dd8dd77e7cb367a7c8a216b/> avoid aggregate progress flicker
   <!-- agent -->
   Track whether aggregate progress has actually been rendered before clearing the
   terminal line. Keep sorted multi-root progress visible until final output, while
   clearing before immediate unsorted results, and cover fast roots that previously
   emitted repeated invisible erase sequences.

### Performance

 - <csr-id-4dba3ad3ac7122bdb82efdb54a49ae811e4f37f9/> parallelize filesystem traversal
   <!-- agent -->
   Split completion-order directory entries into small stealable metadata jobs
   and use idle-aware relay waking with LIFO local queues. Submit all top-level
   roots to the same worker pool, carrying a root index through descendant jobs and
   result batches so aggregate and interactive traversal retain per-root accounting
   without serial barriers.
   
   Also, track pending work per root and emit completion events after all of a
   root's batches are delivered. Flush the longest completed input-order prefix so
   --no-sort prints results immediately without serializing the shared worker pool.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #356 from Byron/performance ([`fea714d`](https://github.com/Byron/dua-cli/commit/fea714dd6c018c7ebf16209d2678b162204e6bce))
    - Parallelize filesystem traversal ([`4dba3ad`](https://github.com/Byron/dua-cli/commit/4dba3ad3ac7122bdb82efdb54a49ae811e4f37f9))
    - Avoid aggregate progress flicker ([`3b7415b`](https://github.com/Byron/dua-cli/commit/3b7415b9fa161ef20dd8dd77e7cb367a7c8a216b))
    - Upgrade ratatui to 0.30.2 ([`995571d`](https://github.com/Byron/dua-cli/commit/995571d5ceb0202ad14d41fe0067da3a5dc4bd44))
    - Upgrade byte-unit to 5.2.5 ([`f113e06`](https://github.com/Byron/dua-cli/commit/f113e063bd946068896c090b15e937dbb74450db))
</details>

## 2.40.0 (2026-07-31)

The headline feature is 40% more scanning speed in my particular scenario. More cores on Linux should now scale much better as well, so I wouldn't be surprised if it's even faster for you.

### New Features

 - <csr-id-8ada93f8b000108ce1ede69f76f1905b05bcc303/> Replace jwalk with a work-stealing directory walker for up to 40% more scan speed
   <!-- agent -->
   Replace jwalk and Rayon with a crate-local walker built on crossbeam-deque
   and standard-library threads. Directory reads and metadata collection run on
   stealable worker queues, while a bounded channel limits completed batches held
   ahead of the iterator. The walker never follows symlinks and no longer sorts
   entries because consumers sort their final results where needed.
   
   Provide two delivery modes on the shared implementation: aggregate scans and
   recursive deletion use completion order for maximum throughput and continuous
   progress; interactive traversal uses parent-first delivery so ancestor sizes
   and entry counts grow throughout the scan. Recursive deletion remains parallel
   through scoped standard-library workers.
   
   Raise the macOS default from three to eight filesystem workers based on
   warm-cache measurements.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #355 from Byron/workstealing ([`0b21b22`](https://github.com/Byron/dua-cli/commit/0b21b22f90dc4c7ed031561c11465e7250b9e971))
    - Thanks clippy ([`9e36f03`](https://github.com/Byron/dua-cli/commit/9e36f030b882258e70fa4daf5591146d934f72eb))
    - Replace jwalk with a work-stealing directory walker for up to 40% more scan speed ([`8ada93f`](https://github.com/Byron/dua-cli/commit/8ada93f8b000108ce1ede69f76f1905b05bcc303))
</details>

## 2.39.1 (2026-07-30)

This release is merely to allow attestations to be used, and you should be able to validate the binary origin with:

- `gh attestation verify ./dua-v2.39.1-aarch64-apple-darwin.tar.gz --repo Byron/dua-cli`

## 2.39.0 (2026-07-28)

The main feature this release is parallel deletion, both for collecting files to be deleted as well as the deletion itself. On my local disk, it now reaches 144k files/s overall deletion performance.

### New Features

 - <csr-id-b04f9f3662a17a473a3a59de0b0122c618341345/> delete files in parallel
   Recursive deletion now uses the same worker count selected with `--threads` for
   the initial filesystem scan, to accelerate the collection of files to be deleted,
   as well as the deletion of files.
   
   <!-- agent -->
   When marked directories are deleted, `dua` now:
   
   1. walks the selected tree without (following symbolic links as usual)
   2. collects regular files and symbolic links;
   3. removes those entries *across the configured workers*;
   4. removes directories from deepest to shallowest after their contents are gone on *a single thread*.
   
   A value of `1` keeps deletion effectively serial. Larger values allow
   independent file removals to overlap, which is most useful for large directory
   trees or filesystems with concurrent metadata operations.

### Bug Fixes

 - <csr-id-f2a957a81a24b066677ec0b4b7b31ea4ab736173/> open errors and cleanup/gitignored footer labels

### Refactor

 - <csr-id-7e9d0a8bed224efd68c7520109f1cf646700c867/> use jwalk for parallel, symlink-safe deletion
   Replace the hand-rolled stack traversal in delete_directory_recursively
   with a jwalk WalkDir (follow_links=false, skip_hidden=false). This is
   the same walker the rest of dua uses for scanning, so deletion now
   benefits from parallel traversal on multi-core machines.
   
   Behaviour preserved:
   - Symlinks are removed without following them (remove_file on the link).
   - Directories are removed deepest-first so each remove_dir sees an
   empty directory.
   - Error counting and byte accounting unchanged.
   
   Adds unit tests covering: single file, nested tree, symlink safety,
   and missing-path error reporting.

### Test

 - <csr-id-48fb6df5e096fb96b8ad757f3ddf1fcc1c8392c1/> Make traversal tree tests filesystem-independent
   That way, tests will work locally.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 4 calendar days.
 - 8 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #353 from Solaris-star/fix/43-jwalk-parallel-deletion ([`0f55a5c`](https://github.com/Byron/dua-cli/commit/0f55a5ce8673e7206a9e20257f0a789be9b0b929))
    - Delete files in parallel ([`b04f9f3`](https://github.com/Byron/dua-cli/commit/b04f9f3662a17a473a3a59de0b0122c618341345))
    - Review ([`cd82444`](https://github.com/Byron/dua-cli/commit/cd82444b1484ad3467630bf4e67297a25a14df2b))
    - Make traversal tree tests filesystem-independent ([`48fb6df`](https://github.com/Byron/dua-cli/commit/48fb6df5e096fb96b8ad757f3ddf1fcc1c8392c1))
    - Use jwalk for parallel, symlink-safe deletion ([`7e9d0a8`](https://github.com/Byron/dua-cli/commit/7e9d0a8bed224efd68c7520109f1cf646700c867))
    - Merge pull request #352 from l0rush1/main ([`496bd78`](https://github.com/Byron/dua-cli/commit/496bd789e2e382d8d3a2ddfdda7b51e3017cd602))
    - Rustfmt annotation_message match arm ([`3dba5a4`](https://github.com/Byron/dua-cli/commit/3dba5a4814339a15a74382443947704079743e79))
    - Open errors and cleanup/gitignored footer labels ([`f2a957a`](https://github.com/Byron/dua-cli/commit/f2a957a81a24b066677ec0b4b7b31ea4ab736173))
    - Fix: interactive exit code and footer entries/s ([`1b3f60d`](https://github.com/Byron/dua-cli/commit/1b3f60de81aa131424d1e6ad9cfc3432eea1846e))
</details>

## 2.38.1 (2026-07-20)

This release fixes a long-standing bug where `NO_COLOR=1` would make all styling disapear, including
the selection indicator itself. Now it's usable, finally.

### Bug Fixes

 - <csr-id-ad19f3fe13322d548f645c16f30f3ac4553f812b/> only strip colors when `NO_COLOR` is enabled

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 6 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#238](https://github.com/Byron/dua-cli/issues/238)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#238](https://github.com/Byron/dua-cli/issues/238)**
    - Only strip colors when `NO_COLOR` is enabled ([`ad19f3f`](https://github.com/Byron/dua-cli/commit/ad19f3fe13322d548f645c16f30f3ac4553f812b))
 * **Uncategorized**
    - Merge pull request #351 from Byron/fix-colors ([`9c9364d`](https://github.com/Byron/dua-cli/commit/9c9364ddff40096c099c59518bfc5ff6ac22ab15))
</details>

## 2.38.0 (2026-07-14)

### New Features

 - <csr-id-d800f22144e470c3822f668b6cc20dab2a0b1df1/> notify after interactive work when unfocused
   Interactive scans, refreshes, deletion, and trash operations can take long
   enough that users leave the terminal, but dua previously had no way to signal
   completion without exiting the TUI.
   
   Track terminal focus events and emit sanitized OSC 777 notifications only when
   the terminal is unfocused. Include concise entry, byte, duration, and error
   statistics, and add a notifications config section whose scan_finished and
   delete_finished switches default to true.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 15 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#347](https://github.com/Byron/dua-cli/issues/347)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#347](https://github.com/Byron/dua-cli/issues/347)**
    - Notify after interactive work when unfocused ([`d800f22`](https://github.com/Byron/dua-cli/commit/d800f22144e470c3822f668b6cc20dab2a0b1df1))
 * **Uncategorized**
    - Merge pull request #348 from Byron/notify-when-done-and-unfocussed ([`92f4502`](https://github.com/Byron/dua-cli/commit/92f45023d52d8341f515bf0acd3d539332f391f5))
</details>

## 2.37.1 (2026-06-29)

### Bug Fixes

 - <csr-id-a83583c40b3cea909823510a0ccf6c92255c0e02/> degrade entries title on narrow terminals
   improve the interactive top-bar so narrow terminal
   sizes degrade gracefully. Statistics should disappear when the current path
   needs the space, and the path should compact by removing the fewest consecutive
   middle components needed to fit.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 8 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #344 from Byron/interactive-path-display-degradation ([`157996b`](https://github.com/Byron/dua-cli/commit/157996be3292081d46f9e0db30598bad0e5c700c))
    - Degrade entries title on narrow terminals ([`a83583c`](https://github.com/Byron/dua-cli/commit/a83583c40b3cea909823510a0ccf6c92255c0e02))
</details>

## 2.37.0 (2026-06-21)

The hallmark change in this release is support for "precious files", a form
of Git-ignored file that `gitoxide` doesn't consider expendable.

### Chore

 - <csr-id-199f16a8064bfc89e6773286d6ed31fd9b25f120/> remove the `git` Cargo feature
   For simpler code, and nobody needed that anyway. Can be re-introduced
   if that changes.

### New Features

 - <csr-id-b572b9603ac32489bc2c94e8f0e03852ab1a0400/> make cleanup heuristics configurable
   Add `cleanup_heuristics` configuration option for interactive mode,
   to allow turning it off mainly as it default to 'on'.
 - <csr-id-9b96b02930bc72b6b3f868d9855a33df8b84135e/> Allow disabling Git support using the configuration file.
   **`gitignore` Configuration Option**
   
   A new config option, `gitignore`, now lets users control Git-ignored entry
   detection in interactive mode. It is defined as an optional boolean in the
   config (`Option<bool>`), and if left unset it defaults to enabled behavior (same
   as previous behavior).
   
   **Usage**
   
   You can control it in your config file as follows:
   
   ```toml
   gitignore = true
   
   gitignore = false
   ```
   
   **Motivation**
   
   This makes Git-ignored behavior configurable without requiring feature flags or
   build-time changes, while preserving existing behavior for users who do not set
   the option.
 - <csr-id-be06d778d521f64d720b41d2bd30fedb5db85ef9/> enable precious ignore syntax through Gitoixde
   When opening the Git repository for ignored-entry
   detection, add a config override so precious ignore syntax works out of the box.
   
   **Usage**
   
   ```gitignore
   preciousFile
   $preciousFile
   disposable
   ```
   
   Now `preciousFile` won't show up in `dua` for auto-marking,
   only `disposable` will.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 4 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #340 from Byron/precious-support ([`568eb05`](https://github.com/Byron/dua-cli/commit/568eb05c042b58e61f7385cc318e67c571fedeea))
    - Make cleanup heuristics configurable ([`b572b96`](https://github.com/Byron/dua-cli/commit/b572b9603ac32489bc2c94e8f0e03852ab1a0400))
    - Allow disabling Git support using the configuration file. ([`9b96b02`](https://github.com/Byron/dua-cli/commit/9b96b02930bc72b6b3f868d9855a33df8b84135e))
    - Remove the `git` Cargo feature ([`199f16a`](https://github.com/Byron/dua-cli/commit/199f16a8064bfc89e6773286d6ed31fd9b25f120))
    - Enable precious ignore syntax through gix ([`be06d77`](https://github.com/Byron/dua-cli/commit/be06d778d521f64d720b41d2bd30fedb5db85ef9))
</details>

## 2.36.0 (2026-06-17)

The headline feature is the optional localization of the interactive help screen, selected from the standard POSIX locale
environment variables (`LC_ALL` > `LC_MESSAGES` > `LANG`). English remains the default; Japanese
(`ja`) is now available for UTF-8 locales and locales without an explicit codeset, e.g. `LANG=ja_JP.UTF-8 dua i`.

### Bug Fixes

 - <csr-id-08bda9ff2eaa06b9a5ec5943cc3dc653b828cdb5/> Make `message` color yellow, instead of red.
   It's less alarming, red should only be used to signal 'danger'.
 - <csr-id-d02a65ef733fb2a71178cdc4892eb283ef0f9fc6/> don't show unapplicable global options in `config` subcommand.

### New Features

 - <csr-id-db3e267d7aaff9eedf0d44021811b71b6cef74dc/> add `config show-default` sub-command with option to reset configuration file
   Add `dua config show-default` to print the current built-in default
   configuration, making newly introduced configuration keys discoverable without
   opening the editor.
   
   Support `dua config show-default --overwrite-with-default` to overwrite the active configuration
   file with the built-in defaults while keeping stdout reserved for the default
   TOML content.
 - <csr-id-c1a759fea60d79d0e2d3eb3ce077fbf901ce7145/> allow to set the `format` in the configuration file.
 - <csr-id-e902598d052d018d6b5c85840470275d3d8e345b/> optional i18n for the interactive help screen
   Localize the interactive help pane via the standard POSIX locale
   environment variables, honoring the usual precedence
   LC_ALL > LC_MESSAGES > LANG, with English as the default. Japanese is
   the first added translation. No new dependencies; only the help screen
   is translated.
   
   - New `i18n` module: a `Language` enum, env detection split into a pure
   `detect()` plus a thin `from_env()`, and an `EN`/`JA` translation table.
   - The help pane resolves the language with `Language::from_env()` when it
   is rendered (only while the pane is open).
   - Key names, the `^` continuation markers and the symbolic legend stay
   untranslated; the block title is localized.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 1 day passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#336](https://github.com/Byron/dua-cli/issues/336)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#336](https://github.com/Byron/dua-cli/issues/336)**
    - Don't show unapplicable global options in `config` subcommand. ([`d02a65e`](https://github.com/Byron/dua-cli/commit/d02a65ef733fb2a71178cdc4892eb283ef0f9fc6))
 * **Uncategorized**
    - Merge pull request #339 from Byron/fix-336 ([`9975c07`](https://github.com/Byron/dua-cli/commit/9975c078a3d6f7e1677b349ce973e9237c7dd915))
    - Make `message` color yellow, instead of red. ([`08bda9f`](https://github.com/Byron/dua-cli/commit/08bda9ff2eaa06b9a5ec5943cc3dc653b828cdb5))
    - Address auto-review ([`e5627c0`](https://github.com/Byron/dua-cli/commit/e5627c0887d9d8b97116e03df2001a80aad55344))
    - Add `config show-default` sub-command with option to reset configuration file ([`db3e267`](https://github.com/Byron/dua-cli/commit/db3e267d7aaff9eedf0d44021811b71b6cef74dc))
    - Allow to set the `format` in the configuration file. ([`c1a759f`](https://github.com/Byron/dua-cli/commit/c1a759fea60d79d0e2d3eb3ce077fbf901ce7145))
    - Merge pull request #334 from bellsmarket/feat/help-i18n ([`c171b44`](https://github.com/Byron/dua-cli/commit/c171b444b91a4c6877ce2e591c5b2e9dc23a2bae))
    - Address auto-review comments ([`e180b1f`](https://github.com/Byron/dua-cli/commit/e180b1f31c5441830e4d90ac481d2319a67cca26))
    - Review ([`71f4532`](https://github.com/Byron/dua-cli/commit/71f4532952d3c8cced61ac5486594e57d5af7867))
    - Optional i18n for the interactive help screen ([`e902598`](https://github.com/Byron/dua-cli/commit/e902598d052d018d6b5c85840470275d3d8e345b))
</details>

## 2.35.0 (2026-06-16)

### New Features

 - <csr-id-a346eff59949f51b84e66c4ab5a6d818a30400c2/> Add gitignore-aware cleanup marking
   Ignored files and directories are detected from the repository’s ignore rules, including `.gitignore`. Git-ignored entries are shown dimmed in the entries list.
   
   Press `I` to mark all currently visible Git-ignored entries, or `i` to disable Git support.
   
   Ignored directories are handled recursively: if a directory such as `target/` is ignored, entries shown after entering that directory are treated as ignored as well.
   
   Git-ignored entries are separate from built-in cleanup candidates. An entry can be both Git-ignored and a cleanup candidate; in that case, both visual styles apply.
 - <csr-id-a6482de5a5efc924cd89bfc005f3f56ce0c086bc/> add `dua i --once[="keys"]` to make it easier to debug interactive mode in the real.
   Run interactive mode once, print the final TUI to the main terminal screen, then exit.
   
   ```bash
   dua i --once
   dua i --once=<keys>
   ```
   
   `<keys>` is optional. Each character is replayed after traversal finishes:
   
   ```bash
   dua i --once=jko
   ```
   
   Acts like pressing `j`, `k`, then `o`.
   
   Because `--once` does not use the alternate screen, the output stays visible in scrollback.
 - <csr-id-cb11cac4f564e14a4739faa68ad92fec0c8fab8b/> add interactive cleanup candidate marking with `X`
   Interactive mode can now highlight and select common cleanup directories in the current view with `shift + X`.
   
   When browsing a directory, `dua` detects existing directories with well-known cleanup names, including:
   
   - `target`
   - `node_modules`
   - `__pycache__`
   - `.pytest_cache`
   - `.mypy_cache`
   - `.ruff_cache`
   - `.tox`
   - `.venv`
   - `venv`
   
   Press `X` to mark all detected cleanup candidates in the current directory.
   The marked entries then appear in the mark pane, where they can be reviewed before using the existing delete or trash actions.
   
   Cleanup detection is intentionally conservative. Ambiguous names such as `build` and `dist` are not selected automatically.
 - <csr-id-5991f981782ebcb8c02e5b36eb3151aab0e9d40c/> cycle modified time display modes
   Users can now enable the modified-time column without changing the current sort:
   
   1. Press `M` while sorting by size, count, or name.
   2. The modified-time column is shown.
   3. Press `M` again to hide it.
   
   To sort by modified time:
   
   1. Press `m` to sort by mtime descending.
   2. Press `m` again to sort by mtime ascending.
   3. Press `M` while mtime sorting is active to cycle the mtime strategy:
   - normal entry mtime
   - newest descendant mtime
   - oldest descendant mtime
   
   The selected mtime strategy is preserved when toggling between ascending and descending mtime sort.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 116 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#328](https://github.com/Byron/dua-cli/issues/328), [#331](https://github.com/Byron/dua-cli/issues/331)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#328](https://github.com/Byron/dua-cli/issues/328)**
    - Add interactive cleanup candidate marking with `X` ([`cb11cac`](https://github.com/Byron/dua-cli/commit/cb11cac4f564e14a4739faa68ad92fec0c8fab8b))
 * **[#331](https://github.com/Byron/dua-cli/issues/331)**
    - Cycle modified time display modes ([`5991f98`](https://github.com/Byron/dua-cli/commit/5991f981782ebcb8c02e5b36eb3151aab0e9d40c))
 * **Uncategorized**
    - Merge pull request #338 from Byron/auto-clean ([`7c31299`](https://github.com/Byron/dua-cli/commit/7c312997094da38567cf7bebe04c34a6cc953384))
    - Add gitignore-aware cleanup marking ([`a346eff`](https://github.com/Byron/dua-cli/commit/a346eff59949f51b84e66c4ab5a6d818a30400c2))
    - Add `dua i --once[="keys"]` to make it easier to debug interactive mode in the real. ([`a6482de`](https://github.com/Byron/dua-cli/commit/a6482de5a5efc924cd89bfc005f3f56ce0c086bc))
    - Merge pull request #337 from Byron/recursive-mod-date ([`99840d0`](https://github.com/Byron/dua-cli/commit/99840d08b8518207590883fbb0bab765b0a4675e))
</details>

## 2.34.0 (2026-02-20)

<csr-id-3dc120fcf193945546ad62f91ae7792c4830c151/>
<csr-id-a1aaaa59a5a1e7b4cee7affdb0ff4fb2f3da4fc3/>

This upcoming release improves day-to-day usability with a new configuration file.

For users, the main additions are:

- A persistent configuration file for `dua` with keyboard settings under `[keys]`.
- A new `dua config edit` command to open the configuration in `$EDITOR`.
- Automatic creation of the configuration directory/file with sensible defaults when editing for the first time.

Configuration defaults and behavior in this release:

- `keys.esc_navigates_back` now defaults to `true`. This is a change from previous versions where it was `false` implicitly.

You are welcome to contribute more settings as you see fit.

### New Features

 - <csr-id-c72cb529a6191aa76d180d3dff1af2c2bd29e31c/> add `dua` configuration file, with setting for ESC going back only.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 45 calendar days.
 - 46 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #320 from tonisives/feat/disable-esc ([`9d2fac5`](https://github.com/Byron/dua-cli/commit/9d2fac55c30d6bb02dfe2961de6183b7b607d6d4))
    - Refactor ([`f275703`](https://github.com/Byron/dua-cli/commit/f2757037548f9765e2c8b5ee132fe3417e00cce8))
    - Apply suggestions from Copilot code review ([`fd3468b`](https://github.com/Byron/dua-cli/commit/fd3468bcb226eef12bd8050c7cfae32b18f7f673))
    - Add `dua` configuration file, with setting for ESC going back only. ([`c72cb52`](https://github.com/Byron/dua-cli/commit/c72cb529a6191aa76d180d3dff1af2c2bd29e31c))
    - Merge pull request #318 from Byron/copilot/remove-crosstermion-tui-react ([`7480277`](https://github.com/Byron/dua-cli/commit/74802778a45c8b2f108566c712e3dd733f6ac0a9))
    - Remove crosstermion and tui-react dependencies ([`a1aaaa5`](https://github.com/Byron/dua-cli/commit/a1aaaa59a5a1e7b4cee7affdb0ff4fb2f3da4fc3))
    - Merge pull request #317 from musicinmybrain/no-atty ([`6c0203c`](https://github.com/Byron/dua-cli/commit/6c0203c5d3c6744949de92749944c57cf61ad2fc))
    - Replace atty with standard-library functionality (since Rust 1.70) ([`31aaa0c`](https://github.com/Byron/dua-cli/commit/31aaa0cde6807ffb8351d07b9aa0e4899a128f39))
    - Merge pull request #315 from Byron/copilot/switch-time-crate-to-jiff ([`017e716`](https://github.com/Byron/dua-cli/commit/017e716c100dd1d2154efe43bf4f5f8f69b7ce8f))
    - Refactor ([`60812a2`](https://github.com/Byron/dua-cli/commit/60812a27b67e72e0170e2a0d15ecb4313096dfcd))
    - Replace simplelog with fern and jiff for timestamped logging ([`3dc120f`](https://github.com/Byron/dua-cli/commit/3dc120fcf193945546ad62f91ae7792c4830c151))
    - Cargo fmt ([`d8db05a`](https://github.com/Byron/dua-cli/commit/d8db05a5ec65412aceea259247cc438d7db220f6))
</details>

## 2.33.0 (2026-01-05)

### New Features

<csr-id-2f720cf5610c215dc4fbb7cd270fe055fd403b42/>

 - <csr-id-85c7c7218cbb70b0626f430afd03ed819387e9e2/> Add environment variable support for all global arguments
   - `DUA_THREADS` → `--threads`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 64 calendar days.
 - 69 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #309 from Byron/copilot/add-env-support-global-arguments ([`72f149c`](https://github.com/Byron/dua-cli/commit/72f149ce0aa2c88bad71d22d46b864a38362e10a))
    - Add environment variable support for all global arguments ([`85c7c72`](https://github.com/Byron/dua-cli/commit/85c7c7218cbb70b0626f430afd03ed819387e9e2))
    - Merge pull request #307 from Byron/copilot/mark-global-arguments-in-clap ([`a2973a6`](https://github.com/Byron/dua-cli/commit/a2973a655e9c33f08d17dcecde1f6ef6827f1182))
    - Mark shared arguments as global for general accessibility ([`2f720cf`](https://github.com/Byron/dua-cli/commit/2f720cf5610c215dc4fbb7cd270fe055fd403b42))
    - Merge pull request #298 from Byron/updates ([`4bb7ebd`](https://github.com/Byron/dua-cli/commit/4bb7ebd7028f378b3dda6a439403a35f9ce44318))
    - Cargo fmt ([`38d985e`](https://github.com/Byron/dua-cli/commit/38d985eebb9c9a791524b7f7835dde01271827a7))
    - Upgrade the rustc version and switch to edition 2024 ([`ccd0b74`](https://github.com/Byron/dua-cli/commit/ccd0b74b92a21fef65b8ea94667100c71183ebe9))
</details>

## 2.32.2 (2025-10-28)

### Bug Fixes

 - <csr-id-847af46ba643c53b8d5aa7a9a3abd9ff37032311/> don't let 'q' quit instantly if it's still collecting files.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Don't let 'q' quit instantly if it's still collecting files. ([`847af46`](https://github.com/Byron/dua-cli/commit/847af46ba643c53b8d5aa7a9a3abd9ff37032311))
</details>

## 2.32.1 (2025-10-28)

'q' to quit is now more usable as it will insta-quit if the traversal took less than 10s and
if nothing is still marked for deletion.

This makes it easy to use in 'quick-view' scenarios.

### Bug Fixes

 - <csr-id-d769de92b7abc842dab45141750e809b1141ba26/> instantly quit when no items marked for deletion

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 43 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #297 from Byron/copilot/fix-instant-quit-on-q ([`938ae33`](https://github.com/Byron/dua-cli/commit/938ae33195498ab3451d83dac90eeb80302b0d12))
    - Only quit immediately if the traversal didn't take too long. ([`7f27170`](https://github.com/Byron/dua-cli/commit/7f271701240d89799b3dd6a8f95cc613dd5c1340))
    - Refactor ([`b710cb1`](https://github.com/Byron/dua-cli/commit/b710cb164615b6c68416ce8bb882e41ea12bd0df))
    - Instantly quit when no items marked for deletion ([`d769de9`](https://github.com/Byron/dua-cli/commit/d769de92b7abc842dab45141750e809b1141ba26))
</details>

## 2.32.0 (2025-09-15)

### New Features

 - <csr-id-bbe368f3c33cf58625e0f2a24f198ee8a6f836a6/> `Ctrl+f` in the glob prompt now toggles the mode from case-insensitive to sensitive.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 44 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #293 from Byron/copilot/fix-2a5bb691-5ca0-4cf5-af1c-895f4fcb1f06 ([`91bc45d`](https://github.com/Byron/dua-cli/commit/91bc45da799e7bd41d75f71a67091c6537de7ef1))
    - `Ctrl+f` in the glob prompt now toggles the mode from case-insensitive to sensitive. ([`bbe368f`](https://github.com/Byron/dua-cli/commit/bbe368f3c33cf58625e0f2a24f198ee8a6f836a6))
    - Implement case-sensitive glob search with '^f' shortcut ([`32ab50f`](https://github.com/Byron/dua-cli/commit/32ab50f5b91cb9e4b4e4fa342d6a36adc944c14e))
</details>

## 2.31.0 (2025-08-02)

This release prominently adds a prompt that shows before quitting the app. When you pressed esc or q, the status bar will show the prompt first. To really quit, you need to press esc or q again. You can also cancel the quit operation by pressing any key else. Meanwhile, ctrl-c still quits the app directly since it's a combination key.

That way, `dua` will not cause users to accidentally quit the app when they only want to dismiss some other panels. It's especially frustrating if the scan took a long time

### New Features

 - <csr-id-f3c9bf65b97ac029d444e32fe23f5976b0c480b2/> prompt before quitting
   This release prominently adds a prompt that shows before quitting the app. When you pressed esc or q, the status bar will show the prompt first. To really quit, you need to press esc or q again. You can also cancel the quit operation by pressing any key else. Meanwhile, ctrl-c still quits the app directly since it's a combination key.
   
   That way, `dua` will not cause users to accidentally quit the app when they only want to dismiss some other panels. It's especially frustrating if the scan took a long time.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release.
 - 84 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Prompt before quitting ([`f3c9bf6`](https://github.com/Byron/dua-cli/commit/f3c9bf65b97ac029d444e32fe23f5976b0c480b2))
    - Prompt user before quitting ([`b096939`](https://github.com/Byron/dua-cli/commit/b09693973a34152a15f2dd599ff48ffbd1e8965e))
    - Re-introduce io::ErrorKind matching. ([`f93f120`](https://github.com/Byron/dua-cli/commit/f93f1205fa4fea33016a66645c8b5ec5c25a4f5c))
    - Merge pull request #288 from fgimian/completions ([`1b7f535`](https://github.com/Byron/dua-cli/commit/1b7f535eb25be4fba4f64efb21efdd74895dbce0))
    - Thanks clippy ([`f983e60`](https://github.com/Byron/dua-cli/commit/f983e6080371ed190ae1b3884e4034812d3d528c))
    - Refactor ([`a0f78b2`](https://github.com/Byron/dua-cli/commit/a0f78b2a9d35097f65d3debb0eeffae8dc15e893))
    - Add the ability to generate shell completions ([`e919541`](https://github.com/Byron/dua-cli/commit/e9195412c08e47fc518b69b57116754fa2fa5a3e))
    - Merge pull request #285 from kianmeng/fix-typos ([`63b129b`](https://github.com/Byron/dua-cli/commit/63b129b1addbac7f4b238529875d88062ab68dfb))
    - Fix typos ([`d9d643e`](https://github.com/Byron/dua-cli/commit/d9d643e63dc7996d88eb54a9dc8bafbf7198c69f))
</details>

## 2.30.1 (2025-05-10)

In this release, the size of directories is also taken into consideration, for more realistic sizes similar to what `du` does.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 27 calendar days.
 - 103 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #284 from joehasson/feat/include-directory-inodes-in-size-calculations ([`b5b411b`](https://github.com/Byron/dua-cli/commit/b5b411b2757d61ebdd64f2254cac002234ed1d5d))
    - Include directory inode in directory size aggregation ([`a93b28e`](https://github.com/Byron/dua-cli/commit/a93b28ead02714bb80cda296e4b4ad2a1248ba0e))
    - Thanks clippy ([`49bbd2c`](https://github.com/Byron/dua-cli/commit/49bbd2c05d091ef344feb83e6a25d825267025e7))
</details>

## 2.30.0 (2025-01-27)

<csr-id-c1dc1b26735279e976d36597bfe45eb3557458fe/>

### New Features

 - <csr-id-73224e63bc21d1ffa416b3e685a95c04afb72657/> allow sorting by name in interactive mode

### Bug Fixes

 - <csr-id-0a4d09eae898c80f8f81bbf8f8c652883d9424e7/> formatting in src/interactive/app/handlers.rs
 - <csr-id-8933be4fa8a801a7f79d994d735eee1105bd30ba/> on MacOS use only 3 threads by default.
   Otherwise, it would get very slow and the difference is enormous.
   16 threads for example take 4.1s on a workload, whereas this only takes
   550ms with 3 threads.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 55 calendar days.
 - 85 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #275 from joehasson/feat/interactive-mode-sort-by-name ([`364f732`](https://github.com/Byron/dua-cli/commit/364f73206dc89277496486da6a6b462fb38e1262))
    - Thanks clippy ([`092a6c5`](https://github.com/Byron/dua-cli/commit/092a6c53cdd0b01f4041f7a79c736b27a1c2a3ce))
    - Allow sorting by name in interactive mode ([`73224e6`](https://github.com/Byron/dua-cli/commit/73224e63bc21d1ffa416b3e685a95c04afb72657))
    - Merge pull request #271 from hamirmahal/style/simplify-some-statements-for-readability ([`3bc25bd`](https://github.com/Byron/dua-cli/commit/3bc25bd5e337bdebce706a89e0fe4227d9997b9a))
    - Formatting in src/interactive/app/handlers.rs ([`0a4d09e`](https://github.com/Byron/dua-cli/commit/0a4d09eae898c80f8f81bbf8f8c652883d9424e7))
    - Simplify some statements for readability ([`c1dc1b2`](https://github.com/Byron/dua-cli/commit/c1dc1b26735279e976d36597bfe45eb3557458fe))
    - On MacOS use only 3 threads by default. ([`8933be4`](https://github.com/Byron/dua-cli/commit/8933be4fa8a801a7f79d994d735eee1105bd30ba))
</details>

## 2.29.4 (2024-11-03)

<csr-id-44d25a64475ff861875fe97c4612356eb697f4bf/>

## 2.29.3 (2024-11-03)

<csr-id-25a6ad73a6571bffe7fac56c61ff2e52ccda0b53/>
<csr-id-c66e585ec73707d113d481ae2627187c9071539d/>
<csr-id-fa203b1b955b896d989eb46e72f13fd5e6cd6120/>

## 2.29.2 (2024-08-10)

A maintenance release without user-facing changes.

## 2.29.1 (2024-08-10)

<csr-id-f0b9a8e07b24d963116da4dfaa3338a4d4e8a3bf/>

This is a maintenance release without user-facing changes.

### Bug Fixes

 - <csr-id-46ebf149548f10c1b144f596aa715062787fd141/> clippy warning

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 153 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'olastor/main' ([`80c14a9`](https://github.com/Byron/dua-cli/commit/80c14a9cd28e5a18f5e9df517f6a3332d90e7c30))
    - Thanks clippy ([`7ddbfbe`](https://github.com/Byron/dua-cli/commit/7ddbfbe37a56b845cc437e60509cb5bb6a89fe01))
    - Merge pull request #246 from matta/use-ratatui-terminal ([`ced3b4f`](https://github.com/Byron/dua-cli/commit/ced3b4f5e375278dbee52319eac8750b14eb328a))
    - Replace tui_react::Terminal with tui::Terminal ([`1350c2f`](https://github.com/Byron/dua-cli/commit/1350c2f5d7e7bd79909fe78584008385dec1b794))
    - Merge pull request #247 from matta/fix-clippy ([`e3aff9d`](https://github.com/Byron/dua-cli/commit/e3aff9d987a09910b52dbce84c0de806d4233b04))
    - Clippy warning ([`46ebf14`](https://github.com/Byron/dua-cli/commit/46ebf149548f10c1b144f596aa715062787fd141))
</details>

## 2.29.0 (2024-03-10)

### New Features

 - <csr-id-0c511ffa0f15e16520353ff712f6bcc11318e379/> Add scrollbar to the main entries list.
   That way it's easier to grasp how long the list is, and how fast one is
   traversing is.

### Bug Fixes

 - <csr-id-caa1e7261bad1b0e2b10628aa14c9d2b6868a14a/> avoid crashes when the terminal is resized to unusually small sizes.
 - <csr-id-24a6c29b3f48289cb6374aa66e84357edb5d0d54/> mark-pane help bar now shows closest to the selected item.
   Previously this would only work in the first screen, but not when
   the list was long enough for scrolling.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release.
 - 47 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Avoid crashes when the terminal is resized to unusually small sizes. ([`caa1e72`](https://github.com/Byron/dua-cli/commit/caa1e7261bad1b0e2b10628aa14c9d2b6868a14a))
    - Mark-pane help bar now shows closest to the selected item. ([`24a6c29`](https://github.com/Byron/dua-cli/commit/24a6c29b3f48289cb6374aa66e84357edb5d0d54))
    - Fix possible overflow during substraction in mark pane ([`a94c7d3`](https://github.com/Byron/dua-cli/commit/a94c7d31ec152ff2427092054b99d8c4f3f74cfd))
    - Add scrollbar for mark list ([`5fe858d`](https://github.com/Byron/dua-cli/commit/5fe858d771d286204d2ed911533869223ea20d2c))
    - Add scrollbar to the main entries list. ([`0c511ff`](https://github.com/Byron/dua-cli/commit/0c511ffa0f15e16520353ff712f6bcc11318e379))
    - Avoid iterating a potentially long list doubly ([`fd797e8`](https://github.com/Byron/dua-cli/commit/fd797e86787ca1675e0f0406828c06506b4b1a11))
    - Add scrollbar for main list ([`120a08a`](https://github.com/Byron/dua-cli/commit/120a08aefeed9581f5d9110861b15ee0cbcd5831))
    - Merge pull request #231 from gosuwachu/dev/pwach/fix-clippy ([`250fdc4`](https://github.com/Byron/dua-cli/commit/250fdc420e12634a195f23f461dda07c998cacea))
    - Fixes clippy error in rust 1.76 ([`85c00cd`](https://github.com/Byron/dua-cli/commit/85c00cd44f7e3dbd862c5d02a7f8310de7ead670))
</details>

## 2.28.0 (2024-01-23)

### New Features

 - <csr-id-78b9a8e22568c902132ed98d32e223ff71eb7b06/> add `dua i --no-entry-check` flag.
   With it, in interactive mode, entries will not be checked for presence.
   
   This can avoid laggy behaviour when switching between directories
   as `lstat` calls will not run, which can be slow on some filesystems.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#226](https://github.com/Byron/dua-cli/issues/226), [#227](https://github.com/Byron/dua-cli/issues/227)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#226](https://github.com/Byron/dua-cli/issues/226)**
    - Make builds with rustc 1.72 work ([`600bee2`](https://github.com/Byron/dua-cli/commit/600bee234edd4e7922017c26927a6f135a02c335))
 * **[#227](https://github.com/Byron/dua-cli/issues/227)**
    - Add `dua i --no-entry-check` flag. ([`78b9a8e`](https://github.com/Byron/dua-cli/commit/78b9a8e22568c902132ed98d32e223ff71eb7b06))
 * **Uncategorized**
    - Merge branch 'no-entry-check' ([`d837d72`](https://github.com/Byron/dua-cli/commit/d837d720e3b1e204043b8d89447db0d65ae000ba))
</details>

## 2.27.2 (2024-01-22)

### Bug Fixes

 - <csr-id-67c5bdb74cfcf8cab647888afec26cd09ccf543a/> allow `/` (glob-mode) while scanning.
   This will possibly lead to incomplete results, but I find being
   able to use ones muscle-memory more important than preventing
   dealing with incomplete results.
   
   What happens to me is usually to type `/` followed by `target/`
   which tends to select all current entries for deletion.
 - <csr-id-c70ca81f007f925c7891340d0d0e763bcfc4114d/> don't check entry metadata while a scan is in progress
   Previously each time the UI refreshes, every 250ms, it display
   entries but also check their metadata to assure they exist.
   
   This could lead to performance loss when the displayed folder
   has a lot of entries.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#223](https://github.com/Byron/dua-cli/issues/223)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#223](https://github.com/Byron/dua-cli/issues/223)**
    - Don't check entry metadata while a scan is in progress ([`c70ca81`](https://github.com/Byron/dua-cli/commit/c70ca81f007f925c7891340d0d0e763bcfc4114d))
 * **Uncategorized**
    - Merge branch 'fix-overhead' ([`7a4b271`](https://github.com/Byron/dua-cli/commit/7a4b27153c2cb47caca87e28c5e178921c3a3fd9))
    - Allow `/` (glob-mode) while scanning. ([`67c5bdb`](https://github.com/Byron/dua-cli/commit/67c5bdb74cfcf8cab647888afec26cd09ccf543a))
</details>

## 2.27.1 (2024-01-21)

### Bug Fixes

 - <csr-id-f70d1a8e6ace812a7949cd7d0299507b71306d48/> Explicit refreshes with 'r and 'R' now work with multiple root paths as will.
   This can happen in cases of `dua i root-a root-b` for instance.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Explicit refreshes with 'r and 'R' now work with multiple root paths as will. ([`f70d1a8`](https://github.com/Byron/dua-cli/commit/f70d1a8e6ace812a7949cd7d0299507b71306d48))
    - Refactor ([`9d976d0`](https://github.com/Byron/dua-cli/commit/9d976d0d76fcf45d1e0672bc5c1533b000a46ebf))
    - Cargo fmt ([`99b5443`](https://github.com/Byron/dua-cli/commit/99b5443f2f8821b0a285320c8ec3f982722cfff8))
    - Tests for refresh & selection ([`dcff2ee`](https://github.com/Byron/dua-cli/commit/dcff2eebed4422f3103d99eac6bd91e56df327c6))
    - Fix refresh with multiple input paths ([`65f6735`](https://github.com/Byron/dua-cli/commit/65f6735b7a0761b1371bcede86e9b46b9920bb5c))
    - Test glob pane open/close ([`7efd77e`](https://github.com/Byron/dua-cli/commit/7efd77e6dd3d442f198ef50967ab50524ca22ffd))
    - Tests for shwing/hiding additional columns ([`dbab511`](https://github.com/Byron/dua-cli/commit/dbab511ff68d8cc7d8e4906db3c2472dd8305b77))
</details>

## 2.27.0 (2024-01-17)

### New Features

 - <csr-id-bed351ed2190e50e2932278b9b13b83c2969401b/> Press `r` or `R` for refresh
   Lower-case `r` will refresh the currently selected entry, while upper-case `R`
   will refresh the entire displayed directory, and all entries in it.
   
   Further, what was called `item` is now called `entry` across the
   user-interface.
 - <csr-id-1544e8dffeacb55940deae2d06534d8a500765d4/> show and hide mtime and item count columns with 'M' and 'C' respectively

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 41 commits contributed to the release.
 - 12 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#96](https://github.com/Byron/dua-cli/issues/96)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#96](https://github.com/Byron/dua-cli/issues/96)**
    - Press `r` or `R` for refresh ([`bed351e`](https://github.com/Byron/dua-cli/commit/bed351ed2190e50e2932278b9b13b83c2969401b))
    - Prepare for (R)efresh support ([`1812227`](https://github.com/Byron/dua-cli/commit/181222745ed50b7346bfd082473168634e01fa99))
 * **Uncategorized**
    - Refactor ([`18a725d`](https://github.com/Byron/dua-cli/commit/18a725dc5af97841afd06dcd4c8469e1d7ea873c))
    - Make `Shift+r` do more than 'r' ([`f1fc13e`](https://github.com/Byron/dua-cli/commit/f1fc13ec8e2af583d0ce4eb541e260e9045c8cf2))
    - Preserve selected element after refresh ([`99e5384`](https://github.com/Byron/dua-cli/commit/99e53849dd6096d05ab4962e1ed5440efcae83f3))
    - Various updates based on the code review feedback: ([`c3d665d`](https://github.com/Byron/dua-cli/commit/c3d665d40264c819be66a5e290a87fb9f2007cf8))
    - Cargo clippy & fmt ([`ad7abd8`](https://github.com/Byron/dua-cli/commit/ad7abd83261d5db6b59fbf9d55a24020c531f157))
    - Fix tests ([`6b24912`](https://github.com/Byron/dua-cli/commit/6b2491200cbabb846f6566cb58eeb8b859a776e0))
    - Exit glob mode if view root is the same as glob root ([`253f720`](https://github.com/Byron/dua-cli/commit/253f720ff81e675d071fd0da8562ddf8ed1626f8))
    - Fix updating item count ([`13614a9`](https://github.com/Byron/dua-cli/commit/13614a9a8989df2dfd434e04a0d9ba132ee79244))
    - Fixed tests ([`69f14af`](https://github.com/Byron/dua-cli/commit/69f14af5403dd17597cfaabf074bf158beabeda3))
    - Remove debug comments ([`9f37e1c`](https://github.com/Byron/dua-cli/commit/9f37e1ca5e9635cb2ebd1c4d543d59340a5c77e8))
    - Refresh all in view vs selected ([`06ee3ab`](https://github.com/Byron/dua-cli/commit/06ee3ab6e7b116c50aabe64c642ff128bbc2fb9a))
    - Fix file count ([`eeae2bc`](https://github.com/Byron/dua-cli/commit/eeae2bc238871a5883624ced30a5ee43b4f8fdfb))
    - Fix traversal stats ([`96ef242`](https://github.com/Byron/dua-cli/commit/96ef242d3b00dfb46800b179595114fecb62fa35))
    - Moved traversal stats to separate type ([`969e64b`](https://github.com/Byron/dua-cli/commit/969e64bbde872d0598b1ebf6278f5d55e152f7b1))
    - Traverse children vs parent & fix parent node size after refresh ([`226cbb8`](https://github.com/Byron/dua-cli/commit/226cbb8b2d6388ddd7a7e48fdac1a4db2ee75474))
    - Add `R` to trigger a full refresh (PoC) ([`30d8dd5`](https://github.com/Byron/dua-cli/commit/30d8dd5fb54ef6db8b4444524407f15db25d7b02))
    - Make WalkOptions available in State so it can re-use it for additional walks. ([`0ad90ba`](https://github.com/Byron/dua-cli/commit/0ad90ba23e59b98ccca198ce075e582c93d13c5c))
    - Merge branch 'show_columns' ([`1a54d95`](https://github.com/Byron/dua-cli/commit/1a54d95bd6e60bd5b071c772324c7a8540d250f6))
    - Show and hide mtime and item count columns with 'M' and 'C' respectively ([`1544e8d`](https://github.com/Byron/dua-cli/commit/1544e8dffeacb55940deae2d06534d8a500765d4))
    - Refactor ([`30da672`](https://github.com/Byron/dua-cli/commit/30da672a83c1063eb6f4c5483cb47f5d69c1dc35))
    - Clippy ([`c4efba8`](https://github.com/Byron/dua-cli/commit/c4efba87179636afeb26e472353a029a4030086c))
    - Fixed tests ([`d903ea6`](https://github.com/Byron/dua-cli/commit/d903ea67a4f77c9483aed7bda1ef6694ee4465da))
    - Fmt ([`6c63bf5`](https://github.com/Byron/dua-cli/commit/6c63bf5a33ebb6b98516ca9a96796facfdab2277))
    - Clippy ([`f74a40a`](https://github.com/Byron/dua-cli/commit/f74a40a7212bde94bae9ff0ee1947a5b1478fb93))
    - New Traversal ([`9eaa961`](https://github.com/Byron/dua-cli/commit/9eaa96144bc72de6515c30fc32961a2807b247c7))
    - Fmt ([`b3236dc`](https://github.com/Byron/dua-cli/commit/b3236dcb3db927f3709e9355b218f42327a66a99))
    - Clippy ([`8aaa05a`](https://github.com/Byron/dua-cli/commit/8aaa05ada6169860cd083a24764bc2c5915b220b))
    - Started fixing tests... ([`5abb9d7`](https://github.com/Byron/dua-cli/commit/5abb9d7e8d18799caa4a2f3823e06b77bdb27133))
    - Remove commented out code ([`7378bd8`](https://github.com/Byron/dua-cli/commit/7378bd8bb1887379688eafe00a773521a7c32c9b))
    - First working version ([`b52f66e`](https://github.com/Byron/dua-cli/commit/b52f66e4cd48bc670b1b98a4a713e280b63d9432))
    - Cargo fmt ([`0cd5ea9`](https://github.com/Byron/dua-cli/commit/0cd5ea9612005ff724226ba502c2bea8ff4f0486))
    - Update entries ([`bb511b5`](https://github.com/Byron/dua-cli/commit/bb511b538c7d75b02d598d495b307a83a11f53c0))
    - Wip ([`51b67ff`](https://github.com/Byron/dua-cli/commit/51b67ff9d009a56272448d1fee1951f30b1de678))
    - Clean-up init function ([`13c381b`](https://github.com/Byron/dua-cli/commit/13c381bebc6a64e553ec11793ec8880f868e712c))
    - Move ByteFormat out of WalkOptions ([`e53036a`](https://github.com/Byron/dua-cli/commit/e53036ad84b71e1121588929fe4653a7ababbf67))
    - Move AppState to separate file ([`feec3eb`](https://github.com/Byron/dua-cli/commit/feec3eb37d50c4b927ae3f948159693f134edf4b))
    - Move TerminalApp to separate file ([`5123cf5`](https://github.com/Byron/dua-cli/commit/5123cf584ab68c0a2f491580289c7243e8651bfa))
    - Scan disabled ([`cf3c507`](https://github.com/Byron/dua-cli/commit/cf3c507bb43221066acf96cde778b66bbd578669))
    - No Interactive enum ([`807916c`](https://github.com/Byron/dua-cli/commit/807916ced6e4ec195e0c3805181f3ccd78d69ce3))
</details>

## 2.26.0 (2024-01-05)

### New Features

 - <csr-id-3c8a31b50da8230bb9268b857e00d0c90e8cb786/> responsive and buttery-smooth UI while scanning in interactive mode.
   Using `dua i` the GUI would populate and is fully usable even while the scan
   is in progress, which is fantastic when scanning big disks which can take several minutes.
   
   However, previously is was quite janky as the refresh loop was bound to receiving
   entries to process, which sometimes stalled for many seconds.
   
   Now the GUI refresh is uncoupled from receiving traversal entries, and it will
   update when the user presses a key or 250ms pass without any input, causing
   it to respond immediately.
   
   Thanks so much for contributing, [@unixzii](https://github.com/unixzii)!

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#209](https://github.com/Byron/dua-cli/issues/209)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#209](https://github.com/Byron/dua-cli/issues/209)**
    - Responsive and buttery-smooth UI while scanning in interactive mode. ([`3c8a31b`](https://github.com/Byron/dua-cli/commit/3c8a31b50da8230bb9268b857e00d0c90e8cb786))
 * **Uncategorized**
    - Refactor ([`0651cae`](https://github.com/Byron/dua-cli/commit/0651cae13b43104402ed9d90147ee8c63fe83b61))
    - Optimize UI responsiveness during scanning state ([`983ba61`](https://github.com/Byron/dua-cli/commit/983ba6172604b83c2e4efad0f03273206a43c5db))
</details>

## 2.25.0 (2024-01-03)

<csr-id-e992659db17f275b48e555afd6b18df737401f01/>
<csr-id-729e7e92410b138f2778ef70f0f59a439028ac29/>

### New Features

 - <csr-id-6fbe17ff51360d62086aa265a0baa9288175cb84/> add `--log-file` flag to keep track of some debug info, which includes panics.
   Previously, when `dua i` was used, panics would be hard to observe, if at all,
   as they would print to the alternate screen. Now, when the `--log-file dua.log`
   is specified, the panic will be emitted into the log file instead and thus won't
   be lost anymore.
   
   This may help with debugging in future.

### Bug Fixes

 - <csr-id-49f98f537bf0ac41a7b1992094103f6d36f135f8/> `--ignore-dirs` now work as expected.
   Previously they would need to be specified as relative to the traversal root, which
   was unintuitive and would lead to ignores not working for many.
   
   Even though this was done for performance to avoid canonicalization, we do now
   perform a more performance version of canonicalization so the overall performance
   should be acceptable nonetheless.
   
   Also note that ignored directories are now logged when using a `--log-file`.
 - <csr-id-20e85c1ebe7ce3a5254fe2675a52cb5d321f1e34/> consistent language across the application and improved style of the Help pane.
   Generally, what was called `entry` is now called `item`, consistently.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 7 calendar days.
 - 8 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#196](https://github.com/Byron/dua-cli/issues/196)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#196](https://github.com/Byron/dua-cli/issues/196)**
    - `--ignore-dirs` now work as expected. ([`49f98f5`](https://github.com/Byron/dua-cli/commit/49f98f537bf0ac41a7b1992094103f6d36f135f8))
 * **Uncategorized**
    - Use `gix-path` for more control and performance. ([`93f0f61`](https://github.com/Byron/dua-cli/commit/93f0f61b3042b933f099714e3a6d336497eb18ba))
    - Refactor ([`7905b48`](https://github.com/Byron/dua-cli/commit/7905b48f2f9ca981a6c617ced3a151e79cab9739))
    - Fix ignore dirs wip ([`e2d5a34`](https://github.com/Byron/dua-cli/commit/e2d5a34b5b6d8212b53d60ceea20324eba08cb2a))
    - Merge branch 'logging' ([`196f0d6`](https://github.com/Byron/dua-cli/commit/196f0d62f32aacc2d393ef2929305a831a150520))
    - Add `--log-file` flag to keep track of some debug info, which includes panics. ([`6fbe17f`](https://github.com/Byron/dua-cli/commit/6fbe17ff51360d62086aa265a0baa9288175cb84))
    - Enforce Rust 2021 style ([`45d886a`](https://github.com/Byron/dua-cli/commit/45d886a6b2c194a5a68961b428f8db2c8daf06a8))
    - Merge branch 'help-language-consistency' ([`0a0dfe6`](https://github.com/Byron/dua-cli/commit/0a0dfe65c4a7bd8851841edf488296966ba27bf0))
    - Consistent language across the application and improved style of the Help pane. ([`20e85c1`](https://github.com/Byron/dua-cli/commit/20e85c1ebe7ce3a5254fe2675a52cb5d321f1e34))
    - Option to enable debug logs ([`4482e1d`](https://github.com/Byron/dua-cli/commit/4482e1de9808a8d662b93b3af907b90000e9f1ae))
    - Keep consistent language/punctuation/case throughout the app. ([`1e6db58`](https://github.com/Byron/dua-cli/commit/1e6db588723dbbc96bc2f083e915d08bdf1b4ddf))
</details>

## 2.24.2 (2023-12-26)

### Bug Fixes

 - <csr-id-b5b8aa26b648d8a034667bca8320ba7952a27780/> avoid duplicate key input on windows.
   On Windows, key-states like press/release/repeat are made available
   separately, which means we should avoid responding to key-releases
   as it would incorrectly double the actual user inputs.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#203](https://github.com/Byron/dua-cli/issues/203)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#203](https://github.com/Byron/dua-cli/issues/203)**
    - Avoid duplicate key input on windows. ([`b5b8aa2`](https://github.com/Byron/dua-cli/commit/b5b8aa26b648d8a034667bca8320ba7952a27780))
    - Upgrade to latest verison of tui-crates and native crossterm events. ([`90b65d5`](https://github.com/Byron/dua-cli/commit/90b65d59f5dde888f81c42e3c812670929b1740a))
 * **Uncategorized**
    - Merge branch 'tui-crates-upgrade' ([`edbb446`](https://github.com/Byron/dua-cli/commit/edbb446100405d16c19059d6ced096144f8bb54e))
</details>

## 2.24.1 (2023-12-25)

### Bug Fixes

 - <csr-id-8ae727e462b38541636c8e03d140953cad8f34cf/> keep checking for existance of entries outside of the glob top-level.
   The glob top-level is used to display all search results which means
   that there can be a lot of them, which would unnecessarily slow down
   the search operation.
   
   Previously it would never check for the existence of an entry in glob mode,
   but now it will do so outside of the top-level.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Keep checking for existance of entries outside of the glob top-level. ([`8ae727e`](https://github.com/Byron/dua-cli/commit/8ae727e462b38541636c8e03d140953cad8f34cf))
</details>

## 2.24.0 (2023-12-24)

<csr-id-9123ee7e648fab654520c33df672c053d5797966/>

This release adds long-awaited globbing support, just hit the `/` key to get started.

You want to find the biggest `.git` directories? Just type `/.git/<enter>` and you are done.
What about all target directories? Just write `target/` to the glob search prompt and it's done.
What about all directories ending in `*.rs/`?
Oh, by accident you typed `*.rs` and now there is a list of a quarter million of entries? No problem,
it's near instant even with millions of files to search or hundreds of thousands to display.

> Note that glob-mode can be exited only by pressing `ESC` when the glob prompt has focus.

Special thanks go to [the contributor](https://github.com/gosuwachu) who made this feature happen,
along with many other improvements. Now `dua` feels refreshed for 2024, and is much more versatile.

Happy holidays!

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 24 commits contributed to the release.
 - 13 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#197](https://github.com/Byron/dua-cli/issues/197)

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#197](https://github.com/Byron/dua-cli/issues/197)**
    - EntryData size test passes on 32-bit ([`9123ee7`](https://github.com/Byron/dua-cli/commit/9123ee7e648fab654520c33df672c053d5797966))
 * **Uncategorized**
    - Merge branch 'glob-review' ([`1c4d6a7`](https://github.com/Byron/dua-cli/commit/1c4d6a77c9f439782446d5d5f791fe9e809de0e7))
    - Use `gix-glob` for matching; support for matching dirs only. ([`2e1858c`](https://github.com/Byron/dua-cli/commit/2e1858ca519fd2a6fbf4839a23abcf17588dcc32))
    - Remove treeview abstraction in favor of something simpler ([`3804a1f`](https://github.com/Byron/dua-cli/commit/3804a1f8e70e1f64977d1fcac20d6541aa5956d7))
    - Refactor glob widget ([`b945a1e`](https://github.com/Byron/dua-cli/commit/b945a1e2613b5b0b2eed85f7c9f34942ab3c4a29))
    - More copy-on-write for entries ([`bc56664`](https://github.com/Byron/dua-cli/commit/bc566649e6941340c2bdbcd178ac73a6a6512f68))
    - Refactor shortening ([`8fae939`](https://github.com/Byron/dua-cli/commit/8fae93966f916291bece3e5673ca83cefa702069))
    - Thanks clippy ([`b431ec3`](https://github.com/Byron/dua-cli/commit/b431ec38f318a50a1b636e72ffed768e9ba1e4c5))
    - Shorten long paths so that they fit on the screen ([`7660d64`](https://github.com/Byron/dua-cli/commit/7660d6497f3810856a65d203d2b6e97b708dc632))
    - Show error message on empty search result ([`360a0d7`](https://github.com/Byron/dua-cli/commit/360a0d72302afb5b068525ef0cec18c21df1b46a))
    - Glob most used keys ([`ff07f39`](https://github.com/Byron/dua-cli/commit/ff07f3935bc0a82e52bc169d2739a9bb603d86b8))
    - Fix formatting ([`0a344fa`](https://github.com/Byron/dua-cli/commit/0a344fa063bdffe7165e8bab6b8a1b8adbac9dce))
    - Fix cursor rendering ([`aaa27e8`](https://github.com/Byron/dua-cli/commit/aaa27e860508e564d82b43295baa4290b53eb87f))
    - Small code review fixes ([`49aecb9`](https://github.com/Byron/dua-cli/commit/49aecb9245054446ac1b338ea1cc29831e72d5e0))
    - Use appropriate tree view when listing entries ([`7244bac`](https://github.com/Byron/dua-cli/commit/7244bac0fc51697ed6be6597dee82a26da222c23))
    - Replace EntryData in EntryDataBundle with individual properties ([`f3b5d00`](https://github.com/Byron/dua-cli/commit/f3b5d00549be57b5da03f3220057b887372ff254))
    - Implements glob search mode ([`df6a02c`](https://github.com/Byron/dua-cli/commit/df6a02cd8fdbe693f507ab34a89227431d7c112e))
    - Merge branch 'add_missing_slash_in_root_dir' ([`9a15867`](https://github.com/Byron/dua-cli/commit/9a158676da9087cd734db6d401fcb98c0e98904c))
    - Make clear why roots were special cased, and try to restore that behaviour. ([`94c008f`](https://github.com/Byron/dua-cli/commit/94c008fe8bd5ff836049f8d5d18478d41bfca9c3))
    - Adds the missing '/' prefix for root directories ([`101a377`](https://github.com/Byron/dua-cli/commit/101a37761952f094a782fb34850c82070565125b))
    - Merge branch 'app_state_init_refactor' ([`f23a57f`](https://github.com/Byron/dua-cli/commit/f23a57fa9c16276525c315c875729c9ef9920fdf))
    - Minior refactor ([`6f09882`](https://github.com/Byron/dua-cli/commit/6f09882fddf8eddc0331671a3176b613d827d4e3))
    - Refactors AppState initialization during app startup ([`238bc5f`](https://github.com/Byron/dua-cli/commit/238bc5f956d220f90197112c82ec71781cd0aa4d))
    - Merge pull request #198 from cinerea0/fix-32bit-test ([`1b838f9`](https://github.com/Byron/dua-cli/commit/1b838f9a057782fd6f11d47d09ae3f77c6bf082d))
</details>

## 2.23.0 (2023-12-11)

### New Features

 - <csr-id-98d5b5a2728e640f9d553648812df379c5534395/> display the total count of entries-to-be-deleted in the mark pane.
   This allows to better estimate how much work will be needed to perform
   the deletion.
   
   For example, when marking 3 items for deletion, previously one would see
   `3 items marked`, but now one will see all items and sub-items, like
   `120k`items marked`, which reflects the work that will be done much more
   precisely.
 - <csr-id-3241022a730dab89f13cbefbefdb583fd6a00994/> Add total size to header bar and change to aggregate, human-readable item count.
   This changes the display from `(2034 items)` to
   `(2k items, 213 MB)`, providing an overview of the total amount
   of storage used along with the total amount of files on a particular
   hiearchy level.

### Bug Fixes

 - <csr-id-192460e5bc72781be1d238912c5d590bfed706cf/> single files will not cause IO error
   Running `dua <filename>` will once again provide size information
   about that filename.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 2 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#194](https://github.com/Byron/dua-cli/issues/194)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#194](https://github.com/Byron/dua-cli/issues/194)**
    - Single files will not cause IO error ([`192460e`](https://github.com/Byron/dua-cli/commit/192460e5bc72781be1d238912c5d590bfed706cf))
 * **Uncategorized**
    - Merge branch 'main_windows_render_refactor' ([`2da2e2e`](https://github.com/Byron/dua-cli/commit/2da2e2e7d264d19cc67ccee6bd8658f7a87901c1))
    - Refactor ([`49772d1`](https://github.com/Byron/dua-cli/commit/49772d17dca72006e602f8707121b3378f948981))
    - Display the total count of entries-to-be-deleted in the mark pane. ([`98d5b5a`](https://github.com/Byron/dua-cli/commit/98d5b5a2728e640f9d553648812df379c5534395))
    - Refactor ([`81eadf8`](https://github.com/Byron/dua-cli/commit/81eadf8cdfcfa964401b5cf5d1e80cc21ec4441f))
    - Calculates mark pane item count consistently with the rest of the app ([`2c69ea1`](https://github.com/Byron/dua-cli/commit/2c69ea1faf40499431616e632e02351a22bac249))
    - Refactors MainWindow render to make it more readable ([`8740d4b`](https://github.com/Byron/dua-cli/commit/8740d4b332290b7fa661b157ed190df9f40ad349))
    - Merge branch 'upgrades' ([`a9dd549`](https://github.com/Byron/dua-cli/commit/a9dd549dc85faf17ce211ff0ab5be4c9863440ed))
    - Upgrade to latest crossterm; switch to `ratatui` from `tui` ([`af2aa61`](https://github.com/Byron/dua-cli/commit/af2aa61813578ecc9f6ccaba5e94049fc6ddf727))
    - Merge branch 'total_item_count' ([`ba2efe4`](https://github.com/Byron/dua-cli/commit/ba2efe48f327c92c021879cded7651d83cf99cec))
    - Add total size to header bar and change to aggregate, human-readable item count. ([`3241022`](https://github.com/Byron/dua-cli/commit/3241022a730dab89f13cbefbefdb583fd6a00994))
    - Displays total item count ([`7b7bad5`](https://github.com/Byron/dua-cli/commit/7b7bad5564d0e87eea4b4bd2d32066063a13b554))
</details>

## 2.22.0 (2023-12-09)

### New Features

 - <csr-id-45ccb7cb5a4765190ea6b8d02e0b29f63b1bd702/> Press `c` to sort by count of entries in a directory.
   That way it's easy to spot places that have a lot of (possibly small) files,
   which otherwise would remain under the radar when sorting by size.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Press `c` to sort by count of entries in a directory. ([`45ccb7c`](https://github.com/Byron/dua-cli/commit/45ccb7cb5a4765190ea6b8d02e0b29f63b1bd702))
    - Assure sorting of entry-counts takes files into consideration. ([`8439ba7`](https://github.com/Byron/dua-cli/commit/8439ba703d7f16b2a8f5bd0348b63b26a5fbe689))
    - Refactor ([`9fb3113`](https://github.com/Byron/dua-cli/commit/9fb3113d788ff746873bd67f6ed508ec1fcf1b02))
    - Adds keybinding for 'c' to toggle sorting by number of items ([`8df0b4c`](https://github.com/Byron/dua-cli/commit/8df0b4c5dc5ee3f512f8812dff709a77cfb18f2f))
    - Merge branch 'column_render' ([`bf4da4e`](https://github.com/Byron/dua-cli/commit/bf4da4e1c4444fb490f85516efc518bb238e1652))
    - Refactor ([`bbcd308`](https://github.com/Byron/dua-cli/commit/bbcd30886f71fcb6e804d3f4170c5ae332c181ea))
    - Fix visual changes ([`b8ad16b`](https://github.com/Byron/dua-cli/commit/b8ad16b493c29c56d94f6ec01a9dc790687a1bdb))
    - Refactors entries panel by moving code to separate functions ([`b5b6aba`](https://github.com/Byron/dua-cli/commit/b5b6abae35a5f205cd57e172c7aa4e9dd16d2053))
</details>

## 2.21.0 (2023-12-06)

### New Features

 - <csr-id-de4c2b3bd368fd032319b606b84fa488299bc9e1/> With a single path provided as root, pretend it's the current working dir
   This makes it seem like the user started the directory walk directly in the given directory,
   which is more intuitive than the previous approach only showed the given directory as
   top-level directory.
   
   Note that this change only affects invocations like `dua <dir>` or `dua i <dir>`.
 - <csr-id-dd523e389bcc940a5d3e72099bb0c76f40371164/> press `m` to sort by modification date, ascending and descending.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 11 calendar days.
 - 15 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#110](https://github.com/Byron/dua-cli/issues/110), [#141](https://github.com/Byron/dua-cli/issues/141), [#179](https://github.com/Byron/dua-cli/issues/179), [#186](https://github.com/Byron/dua-cli/issues/186)

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#110](https://github.com/Byron/dua-cli/issues/110)**
    - With a single path provided as root, pretend it's the current working dir ([`de4c2b3`](https://github.com/Byron/dua-cli/commit/de4c2b3bd368fd032319b606b84fa488299bc9e1))
    - Assure `device_id` is taken from the final CWD ([`74e6d42`](https://github.com/Byron/dua-cli/commit/74e6d4222a7f70253f1d69eb8e7cf94114827852))
 * **[#141](https://github.com/Byron/dua-cli/issues/141)**
    - Press `m` to sort by modification date, ascending and descending. ([`dd523e3`](https://github.com/Byron/dua-cli/commit/dd523e389bcc940a5d3e72099bb0c76f40371164))
 * **[#179](https://github.com/Byron/dua-cli/issues/179)**
    - Press `m` to sort by modification date, ascending and descending. ([`dd523e3`](https://github.com/Byron/dua-cli/commit/dd523e389bcc940a5d3e72099bb0c76f40371164))
 * **[#186](https://github.com/Byron/dua-cli/issues/186)**
    - Assure `device_id` is taken from the final CWD ([`74e6d42`](https://github.com/Byron/dua-cli/commit/74e6d4222a7f70253f1d69eb8e7cf94114827852))
 * **Uncategorized**
    - Thanks clippy ([`0c4d31b`](https://github.com/Byron/dua-cli/commit/0c4d31b406b2c988af3f17fc79b0cf3d7364a910))
    - Skip through single root directory ([`e9fb2fd`](https://github.com/Byron/dua-cli/commit/e9fb2fda3478fefa38bdb9d176380bae5545dbc6))
    - Fix tests on Windows ([`1b7457e`](https://github.com/Byron/dua-cli/commit/1b7457e0301db3029e1b4beb52acfb99fe408174))
    - Hide mtime column by default, unless enabled ([`0f8377a`](https://github.com/Byron/dua-cli/commit/0f8377a450b02bad317eed59d1593007aa5c0bed))
    - Adds keybinding 'm' to toggle sorting by modified time ([`2bd06be`](https://github.com/Byron/dua-cli/commit/2bd06be9ee5ad8e1a747544899b299a53a950940))
    - Add test to assure memory consumption of EntryData doesn't change unexpectedly. ([`adebd00`](https://github.com/Byron/dua-cli/commit/adebd00daa409da67d2f252b966e2dba632acda3))
</details>

## 2.20.3 (2023-11-21)

### Bug Fixes

 - <csr-id-7ab0070dcfda573cfbdc8451ddba5fcf15067132/> mark-pane now doesn't double-count sizes anymore.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Mark-pane now doesn't double-count sizes anymore. ([`7ab0070`](https://github.com/Byron/dua-cli/commit/7ab0070dcfda573cfbdc8451ddba5fcf15067132))
    - Fixes marking parent directory for deletion counts children twice ([`f7086cc`](https://github.com/Byron/dua-cli/commit/f7086cc0836bd091552a83d8faabf937fb4c6cf8))
</details>

## 2.20.2 (2023-11-20)

### Bug Fixes

 - <csr-id-49c3e3d02ad0c14c4123fe1a7fea1f2a5e7a990f/> alignment when in interactive mode and -f binary

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 199 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#177](https://github.com/Byron/dua-cli/issues/177)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#177](https://github.com/Byron/dua-cli/issues/177)**
    - Alignment when in interactive mode and -f binary ([`49c3e3d`](https://github.com/Byron/dua-cli/commit/49c3e3d02ad0c14c4123fe1a7fea1f2a5e7a990f))
 * **Uncategorized**
    - Fixes alignment when in interactive mode and -f binary ([`b3bb851`](https://github.com/Byron/dua-cli/commit/b3bb85177d2fc4b299a9d82313832be96b34c3b6))
</details>

## 2.20.1 (2023-05-05)

## 2.20.0 (2023-05-05)

### New Features

 - <csr-id-13bfe4582f8cbf6f8f12e7ee8acaae710e8a87d2/> TUI now shows performance metrics while scanning and after.
   This is in preparation for the `moonwalk` upgrade.
 - <csr-id-d0e85fec1586a8937928472e361837ef21e40b14/> improve CLI help provided with the `--format` flag.
   It's now possible to see what possible values are without reading a swath
   of text. Now the default is shown as well which is more important now that
   it changes depending on the platform.
 - <csr-id-22f54dd7c0e83b55e0acc2fb1a10ab487bdeb9fb/> use metric byte format only on MacOS.
   That way, on linux the binary format is used by default which is more common
   on that platform.

### Bug Fixes

 - <csr-id-b61ec973b7437230183d6dabf361b0848519f5dc/> Improve documentation for `Marked Items` pane to make clearer how to delete items.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release.
 - 71 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#33](https://github.com/Byron/dua-cli/issues/33), [#85](https://github.com/Byron/dua-cli/issues/85)

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#33](https://github.com/Byron/dua-cli/issues/33)**
    - Improve documentation for `Marked Items` pane to make clearer how to delete items. ([`b61ec97`](https://github.com/Byron/dua-cli/commit/b61ec973b7437230183d6dabf361b0848519f5dc))
 * **[#85](https://github.com/Byron/dua-cli/issues/85)**
    - Use metric byte format only on MacOS. ([`22f54dd`](https://github.com/Byron/dua-cli/commit/22f54dd7c0e83b55e0acc2fb1a10ab487bdeb9fb))
 * **Uncategorized**
    - TUI now shows performance metrics while scanning and after. ([`13bfe45`](https://github.com/Byron/dua-cli/commit/13bfe4582f8cbf6f8f12e7ee8acaae710e8a87d2))
    - Thanks clippy ([`565581f`](https://github.com/Byron/dua-cli/commit/565581fc11faf7512c27fe9095090f482a8d32f0))
    - Simplify GUI refreshes by using a throttle ([`c921dc7`](https://github.com/Byron/dua-cli/commit/c921dc72d3008179e72df9d85f0e0c21c998e199))
    - Generalize the throttle implementation to allow usagein UI ([`e03c560`](https://github.com/Byron/dua-cli/commit/e03c560e8b54e2e231d578e1d5e9dcd206d34216))
    - Added additional clarification for deleting help files. ([`fcc8be9`](https://github.com/Byron/dua-cli/commit/fcc8be93bd8224c01216ed2136cbf7309470ca2f))
    - Improve CLI help provided with the `--format` flag. ([`d0e85fe`](https://github.com/Byron/dua-cli/commit/d0e85fec1586a8937928472e361837ef21e40b14))
    - Refactor ([`b474b81`](https://github.com/Byron/dua-cli/commit/b474b8146de6ce925098b08a1d6af62aa0c25f77))
    - Use binary format by default (except on macOS) ([`3ccf204`](https://github.com/Byron/dua-cli/commit/3ccf204a18c784a7af7b6255173b332e0083c047))
    - Merge pull request #147 from nyurik/patch-1 ([`658c676`](https://github.com/Byron/dua-cli/commit/658c676be779655165e5c5462873c8e828e710f2))
</details>

## 2.19.2 (2023-02-23)

<csr-id-fe956ca6f244613762bb48de79eac1f6fa399e1b/>

### Bug Fixes

 - <csr-id-31dacad6f723f379a2d12417d65177faccd67b76/> `-x` is applied to traversal as well.
   Previously `dua` would cross filesystems for traversal and simply not
   yield them, which somewhat defeated the purpose.
   
   Now it will avoid traversing into filesystem entries that are on a different
   filesystem, which should improve its performance visibly whenever multiple
   filesystems are involved.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 18 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - `-x` is applied to traversal as well. ([`31dacad`](https://github.com/Byron/dua-cli/commit/31dacad6f723f379a2d12417d65177faccd67b76))
    - Refactor ([`dbc9845`](https://github.com/Byron/dua-cli/commit/dbc9845c7d63d7c113f9f61b91da99ff0b249ad2))
    - Update help.rs ([`c36c5b9`](https://github.com/Byron/dua-cli/commit/c36c5b968814e77c538efd0765894491dc150e95))
    - Don't recurse on cross-device filesystems ([`fe956ca`](https://github.com/Byron/dua-cli/commit/fe956ca6f244613762bb48de79eac1f6fa399e1b))
</details>

## 2.19.1 (2023-02-05)

### Bug Fixes

 - <csr-id-fb5a39ffb67fad80be0d2090efd34d259d439e98/> redraw window while gathering metadata in interactive mode.
   This fixes a by now long-standing issue with interactive mode only updating
   when keys are pressed, but not automatically.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 52 calendar days.
 - 54 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#143](https://github.com/Byron/dua-cli/issues/143)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#143](https://github.com/Byron/dua-cli/issues/143)**
    - Redraw window while gathering metadata in interactive mode. ([`fb5a39f`](https://github.com/Byron/dua-cli/commit/fb5a39ffb67fad80be0d2090efd34d259d439e98))
 * **Uncategorized**
    - Draw window before processing events, fixes #143 ([`d957a61`](https://github.com/Byron/dua-cli/commit/d957a61ac79b990fa3cf470a9b500b6f390e3a18))
    - Create our own threadpool with minimal stack instead of using the global one. ([`7802985`](https://github.com/Byron/dua-cli/commit/78029853ba687cabd37adbbdf41b2ee480bbcbf8))
    - Uprgade to latest `jwalk` version for more hang-safety ([`9bdf26a`](https://github.com/Byron/dua-cli/commit/9bdf26a7dbb7577ea10e0eac970c081a7bfa66a6))
</details>

## 2.19.0 (2022-12-13)

### New Features

 - <csr-id-f073375938f742db3259ec284c3c0d4a56fd0077/> Remove the handbrake on MacOS which can now deliver the expected performance.
   Previously it would limit itself to only using 4 threads as it would
   use a lot of time in user space. This has changed now, and the traversal
   itself is much more efficient (even though it could definitely be more
   efficient when comparing to `pdu`).
   
   In any case, counting performance should now greatly improve on M1
   MacOS machines.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Remove the handbrake on MacOS which can now deliver the expected performance. ([`f073375`](https://github.com/Byron/dua-cli/commit/f073375938f742db3259ec284c3c0d4a56fd0077))
</details>

## 2.18.2 (2022-12-13)

## 2.18.1 (2022-12-13)

<csr-id-946806e7390799807361562b038fb12eeb2ddf11/>
<csr-id-d3fa946029ef44e5032762ff265180c23a629316/>

Update all dependencies to the latest version. This most notably changes the look of the CLI
to something without color by default thanks to the upgrade to `clap` 4.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 92 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 2 times to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade to clap 4 ([`dd8b0ef`](https://github.com/Byron/dua-cli/commit/dd8b0ef8e12dfc8d7fb8f359f504c63034d60b9f))
    - Upgrade sysinfo and make thread detection work for all Apple M series for now. ([`bbd5c67`](https://github.com/Byron/dua-cli/commit/bbd5c67342f9e5b509b0ab6e9ca2319c3c7605e2))
    - Thanks clippy ([`82dc467`](https://github.com/Byron/dua-cli/commit/82dc4670bd9b3b93ae949022ecdc58ead79cf905))
    - Replace `colored` dependency with `owo-colors`. ([`946806e`](https://github.com/Byron/dua-cli/commit/946806e7390799807361562b038fb12eeb2ddf11))
    - Refactor ([`a734efb`](https://github.com/Byron/dua-cli/commit/a734efb7e332de6a3bb4911e72463e4f6fc342e1))
    - Thanks clippy ([`44e19ee`](https://github.com/Byron/dua-cli/commit/44e19ee67924eb28b87698874d377a999cafceee))
    - Colored path printing; fix size column format ([`d3fa946`](https://github.com/Byron/dua-cli/commit/d3fa946029ef44e5032762ff265180c23a629316))
</details>

## 2.18.0 (2022-09-12)

<csr-id-6a636d542594a76ef8b2faf2ec6347e4c8cb6b38/>

### Fixes

- Remove a duplicate draw call which would have doubled the time it takes to refresh on user input.
  This might have been noticable when large amounts of files are displayed.

### New Features

 - <csr-id-28f5ac90cc1ba7d668ae8a83eb5cd899294a8301/> Automatically resize if the terminal changes in size.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 52 calendar days.
 - 69 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#28](https://github.com/Byron/dua-cli/issues/28)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#28](https://github.com/Byron/dua-cli/issues/28)**
    - Automatically resize if the terminal changes in size. ([`28f5ac9`](https://github.com/Byron/dua-cli/commit/28f5ac90cc1ba7d668ae8a83eb5cd899294a8301))
 * **Uncategorized**
    - Merge branch 'dep-upgrade' ([`20b7672`](https://github.com/Byron/dua-cli/commit/20b76721939b77dc6c9a86d3c5f4c22cc7f1cf65))
    - Switch from colored to owo-colors ([`6a636d5`](https://github.com/Byron/dua-cli/commit/6a636d542594a76ef8b2faf2ec6347e4c8cb6b38))
    - Add Apple M2 to default thread derivation ([`b5ec900`](https://github.com/Byron/dua-cli/commit/b5ec90042dec10fef8a35c27c2f7dcdb97b92293))
</details>

## 2.17.8 (2022-07-05)

## 2.17.7 (2022-06-14)

### Fixes

- Improve readability of the currently visible path in light terminal color themes [(#129)](https://github.com/Byron/dua-cli/pull/129).

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'show-path' ([`1beb7d7`](https://github.com/Byron/dua-cli/commit/1beb7d7870a308e4829caada7ba3147326ffe0d4))
    - Restyle for compatibility with 'light' color schemes ([`ed28cdb`](https://github.com/Byron/dua-cli/commit/ed28cdbe979cf1fa4a2eccfc3a851fd94f7f2695))
</details>

## 2.17.6 (2022-06-12)

A maintenance release which should make the `ctrl + o` feature open files without blocking on linux
thanks to an upgrade in the `open` crate which powers this feauture.

## 2.17.5 (2022-05-13)

## 2.17.4 (2022-05-12)

### Bug Fixes

- Show all possible information even if one input path could not be read. Previously it would fail
  entirely without printing anything useful but a relatively non-descript error message.
 - <csr-id-75b3eed98f14d918f474f73caa3cdedd5af927ad/> broken or non-existing root path will still print the valid results.
   Previously it would fail completely without printing anything.
 - <csr-id-8742232a15c2bdd608c2e2c731a786c59c7d58dc/> Open interactive mode even if one of the input paths can't be read.
   Note that there can still be improvements in indicating which path
   failed.
   Also it will happily show an empty user interface in case all input
   paths are not readable.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#124](https://github.com/Byron/dua-cli/issues/124)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#124](https://github.com/Byron/dua-cli/issues/124)**
    - Open interactive mode even if one of the input paths can't be read. ([`8742232`](https://github.com/Byron/dua-cli/commit/8742232a15c2bdd608c2e2c731a786c59c7d58dc))
    - Broken or non-existing root path will still print the valid results. ([`75b3eed`](https://github.com/Byron/dua-cli/commit/75b3eed98f14d918f474f73caa3cdedd5af927ad))
 * **Uncategorized**
    - Merge branch 'broken-link-handling' ([`157b43c`](https://github.com/Byron/dua-cli/commit/157b43c2cb203c067c66f499a9fd849e5f0e811c))
</details>

## 2.17.3 (2022-05-10)

## 2.17.2 (2022-05-06)

A maintenance release that updates all dependencies. Most notably, `trash-rs` includes a fix for
properly moving files into the trash that required parent directories to be created.

## 2.17.1 (2022-03-20)

### Improvements to aggregate progress reporting

Previously, aggregate mode progress reports were handled by an
infinitely-looping thread carrying a 64-bit atomic of the current count,
which it would print periodically.

This resulted in #99 - breaking on platforms without 64-bit atomics,
for which a feature was added to disable it.

It also implied a race condition, where the "Enumerating ..." message
could be printed after results had been gathered but before dua exited.

Additionally, part of the status message could be left on the display if
the first line of a report was too short to cover it.

This commit should resolve these:

- The 64-bit atomic counter is replaced with an 8-bit AtomicBool
- All printing is controlled from the main thread
- The first line is cleared prior to printing a report

The only notable drawback I see with this approach is that progress
reporting can sometimes be delayed, since the display is only evaluated
for update during periods the aggregation loop makes progress. The
practical difference appears relatively minor.

Since this should resolve #99, the aggregate-scan-progress feature is
removed.

Special thanks to [@Freaky](https://github.com/Freaky) for the contribution!

### BREAKING change for package maintainers

The `aggregate-scan-progress` feature was removed as it shouldn't be required anymore.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 58 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Improve aggregate progress reporting ([`7d83f96`](https://github.com/Byron/dua-cli/commit/7d83f965d620ccebeda9a7451cdbb2e40ed88c24))
    - Adjust to changes in clap ([`f9df024`](https://github.com/Byron/dua-cli/commit/f9df02420d7bd4e492c4a9130833fdf31e739909))
    - Update clap to official release ([`b029dc5`](https://github.com/Byron/dua-cli/commit/b029dc5d190b23bf3e3fc95a3947f28f868e674e))
</details>

## 2.17.0 (2022-01-21)

### New Features

 - <csr-id-e2686952b4daf4c35303689c36bebc3dfe3faf29/> interactive mode learns 'toggle [a]ll' and 'remove [a]ll'.
   In the mark pane, the 'a' key will now toggle all entries.
   This is particularly interesting for selecting entries to
   exclude by hande and then invert the selection by toggling [a]ll.
   
   In the mark pane, toggling all with the 'a' key means removing
   all entries and closing the pane.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 12 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Interactive mode learns 'toggle [a]ll' and 'remove [a]ll'. ([`e268695`](https://github.com/Byron/dua-cli/commit/e2686952b4daf4c35303689c36bebc3dfe3faf29))
    - Add documentation ([`6dbaa57`](https://github.com/Byron/dua-cli/commit/6dbaa570014f27b20ca719f5a092e768e4c8289d))
    - Add `a` key to toggle marked status of all entries ([`15d0597`](https://github.com/Byron/dua-cli/commit/15d0597a51b166e022ba2d41c377d515a878c1a2))
</details>

## 2.16.0 (2022-01-09)

### New Features

 - <csr-id-26d65145650cc3aac4ad540fdf04e95e139812e3/> Add `--ignore-dirs` option, with useful default on linux.
   
   On linux there are a few directories which shouldn't be traversed by
   default as they may cause hangs and blocking.
   
   With the new argument it's possible to specify absolute directories
   to not enter during traversal, with a default set to avoid
   problematic directories on linux right away.

### Bug Fixes

 - <csr-id-756ca542a73575df581433fdd84cee8f4bef99b5/> build on platforms without 64-bit atomics

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 13 calendar days.
 - 75 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#116](https://github.com/Byron/dua-cli/issues/116)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#116](https://github.com/Byron/dua-cli/issues/116)**
    - Add `--ignore-dirs` option, with useful default on linux ([`26d6514`](https://github.com/Byron/dua-cli/commit/26d65145650cc3aac4ad540fdf04e95e139812e3))
 * **Uncategorized**
    - Build on platforms without 64-bit atomics ([`756ca54`](https://github.com/Byron/dua-cli/commit/756ca542a73575df581433fdd84cee8f4bef99b5))
    - Upgrade clap ([`87d8c45`](https://github.com/Byron/dua-cli/commit/87d8c45b105722352f58b2020aaeaff62f3e00f6))
</details>

## 2.15.0 (2021-12-27)

Make `dua` less prone to hanging by ignoring certain special directories on linux.

### New Features

 - <csr-id-d5fe5cca53a74c4c3cf392100d6ea5c2fe712a9d/> Add `--ignore-dirs` option, with useful default on linux.
   
   On linux there are a few directories which shouldn't be traversed by
   default as they may cause hangs and blocking.
   
   With the new argument it's possible to specify absolute directories
   to not enter during traversal, with a default set to avoid
   problematic directories on linux right away.

## 2.14.11 (2021-10-26)

### Bug Fixes

 - <csr-id-f26309c91a271f1c2c32dfb55dbbb8c713f5e97d/> `cargo install` without `--locked` should work now

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#111](https://github.com/Byron/dua-cli/issues/111)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#111](https://github.com/Byron/dua-cli/issues/111)**
    - Cargo install without --locked should work now ([`f26309c`](https://github.com/Byron/dua-cli/commit/f26309c91a271f1c2c32dfb55dbbb8c713f5e97d))
 * **Uncategorized**
    - Thanks clippy ([`6cff8bc`](https://github.com/Byron/dua-cli/commit/6cff8bc4aea9ac0c93903fcf1357d29a3b9fea0b))
</details>

## 2.14.10 (2021-10-26)

## 2.14.9 (2021-10-26)

## 2.14.8 (2021-10-26)

### Changed

 - <csr-id-49193f0506946981bc056b29c3f09c94e30ac457/> auto-config support for Apple M1 Pro and Apple M1 Max

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 38 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Auto-config support for Apple M1 Pro and Apple M1 Max ([`49193f0`](https://github.com/Byron/dua-cli/commit/49193f0506946981bc056b29c3f09c94e30ac457))
</details>

## v2.14.7 (2021-09-18)

- Fix deletion which broke with Rust 1.55, for those who are compiling the tool themselves.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 27 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix deletion process on Rust 1.55 ([`f45681a`](https://github.com/Byron/dua-cli/commit/f45681aa523fa6cc9d451ef46a8ce62f2ef99bf8))
</details>

## v2.14.6 (2021-08-22)

- Support for arrow keys as well as Home & End. The help pane was updated to reflect these changes.
- More readable information on how to delete or trash files in the mark pane.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 3 calendar days.
 - 6 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'style' ([`5904630`](https://github.com/Byron/dua-cli/commit/5904630cfebd4e99bc4ee7a9c23550f85add41d4))
    - Support Home/End and fix inconsistent help text ([`29017f6`](https://github.com/Byron/dua-cli/commit/29017f6f94003f58118ad7d1fded1d47f79349eb))
    - Improve mark widget tip style ([`019e4cb`](https://github.com/Byron/dua-cli/commit/019e4cb65e6d6302e08692c446bac56fb3beee25))
    - Format correctly ([`8977c17`](https://github.com/Byron/dua-cli/commit/8977c17bcb10373c33d695dd682781fd9590e4e7))
    - Remove unnecessary line ([`d6bbb6d`](https://github.com/Byron/dua-cli/commit/d6bbb6dd91b5367f8bd1f8569d39dbb30b8f89a2))
</details>

## v2.14.5 (2021-08-16)

- Fix installation via `cargo install dua-cli`. Please note that it might break again as it still depends on the unsable `clap-3 beta 4`. Even when pinning it breakage is possible as its dependencies itself aren't pinned.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 11 calendar days.
 - 11 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix #102, bump patch level ([`3a6c654`](https://github.com/Byron/dua-cli/commit/3a6c654dc2939b5979c47d8fbd14932741f8d1d1))
    - Add aggregate-scan-progress feature to help with #99 ([`7429cb3`](https://github.com/Byron/dua-cli/commit/7429cb3d1139605abdf3efcb8a4d5cceb300be1b))
</details>

## v2.14.4 (2021-08-05)

- upgrade depencies
- upgrade to tui 0.16

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release over the course of 6 calendar days.
 - 11 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Thanks clippy ([`4598d64`](https://github.com/Byron/dua-cli/commit/4598d64a1150967e48013091e044eae851de62f9))
</details>

## v2.14.3 (2021-07-25)

- upgrade `open` crate to v2

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 11 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade open to v2 ([`98c859c`](https://github.com/Byron/dua-cli/commit/98c859c71d9ee4be4c19bc436a494f035a241bc1))
</details>

## v2.14.2 (2021-07-14)

- `Ctrl-T` to trash (instead of removal) is now an optional default feature, allowing it to be
  disabled on FreeBSD which isn't currently supported.
- Update dependencies

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 14 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'optional-trash' ([`b12b98a`](https://github.com/Byron/dua-cli/commit/b12b98a07935c839a11af08cfa9dc872b5a127e8))
    - Disable test that now starts failing on windows even though… ([`64175e0`](https://github.com/Byron/dua-cli/commit/64175e028965958d0c22f8ffe55cab2fc01f9fc8))
    - Refactor ([`6894dd8`](https://github.com/Byron/dua-cli/commit/6894dd8db51cd6fe8a70ad0c906ef351dc0a720c))
    - Make the trash feature optional ([`1fdded1`](https://github.com/Byron/dua-cli/commit/1fdded129fe766729ac332fa881c0681c9495316))
</details>

## v2.14.1 (2021-06-30)

- Pressing `ctrl+t` in the mark pane now trashes entries instead of deleting them. Not only does that make
  'deletion' reversible but it makes removal of the entry faster in many cases as well.
- updated dependencies

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade sysinfo ([`e1b8a01`](https://github.com/Byron/dua-cli/commit/e1b8a01579e211c268356ea25c56cfb9391ca717))
    - Cargo fmt ([`97a9804`](https://github.com/Byron/dua-cli/commit/97a980436ab46693804ad0a361ab0388f34c8381))
</details>

## v2.14.0 (2021-06-30)

<csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/>

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 21 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'trash' ([`64d8dc8`](https://github.com/Byron/dua-cli/commit/64d8dc8b9baf0fd2e8942b1391f783fe8a7d4586))
    - Thanks clippy ([`68bbb68`](https://github.com/Byron/dua-cli/commit/68bbb68ffd4887d2023a520e4dfc69b9d8edc736))
    - Add mark pane prompt message for ctrl + t ([`af538bc`](https://github.com/Byron/dua-cli/commit/af538bc545c3b3b7c0a3d5541a1a80b0da536e5b))
    - Deduplicate code ([`02dd1b7`](https://github.com/Byron/dua-cli/commit/02dd1b72c8fe741fb153094fdb08816f7f593c6f))
    - Implement Ctrl+t move to trash ([`00fae90`](https://github.com/Byron/dua-cli/commit/00fae90e0dffc468c75bd362fa4220bc8650fb86))
</details>

## v2.13.1 (2021-06-09)

<csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/>

- Allow usage of the feature introduced in v2.13 by writing the TUI to stderr instead of stdout.
  That way the output can be redirected.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Show TUI on stderr to enable writing files to stdout ([`a93a642`](https://github.com/Byron/dua-cli/commit/a93a642765540d4010dc2fab90737cd39abaa32d))
</details>

## v2.13.0 (2021-06-08)

- Print remaining marked paths upon exit on stdout. This may help to use `dua i` with other programs
  who want to process the marked paths on their own.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Print marked items upon exit if these are left in the marked pane ([`017cbd7`](https://github.com/Byron/dua-cli/commit/017cbd7b4c3e57e1a98fbc595159be39bc97c708))
</details>

## v2.12.2 (2021-06-07)

- Prepare for release of new Apple hardware and be more specific when auto-configuring the correct amount of threads.
  Instead an error message will be printed to inform that the given CPU brand isn't configurable yet.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 8 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Prepare new release ([`f45852a`](https://github.com/Byron/dua-cli/commit/f45852a5880fbcd9670f0de3643ea9614ec35de4))
    - Set default processor count on Apple Silicon in a way that won't be totally wrong in future ([`fe9611a`](https://github.com/Byron/dua-cli/commit/fe9611a7fd9a1592cc1a4517948b4a32fba904c9))
    - Refactor ([`c3c103e`](https://github.com/Byron/dua-cli/commit/c3c103eebd82fc729788694a9f3bfd4ded855cf8))
    - Refactor ([`115db26`](https://github.com/Byron/dua-cli/commit/115db26ab86fcb50dd14b12b64240b66bbac53f1))
</details>

## v2.12.1 (2021-05-30)

- Fixed bug that would cause `dua` to unconditionally sleep for 1 second. This sleep was intended for a spawned thread,
  but it slipped into the main thread.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix terrible bug causing an unnecessary wait in front of each invocation ([`ac604b3`](https://github.com/Byron/dua-cli/commit/ac604b35c0b80fa6b380cc395a95bf0a5d1d196d))
    - Fix tests ([`dfb40a2`](https://github.com/Byron/dua-cli/commit/dfb40a20d1e697d2f3fc3a159febf9adb3a817b2))
    - Only fetch metadata for files for a speedup ([`d381c6c`](https://github.com/Byron/dua-cli/commit/d381c6caed1fd404d7a11c1f581abdba749b7a20))
    - Mildly optimize progress performance… ([`ffdb0c2`](https://github.com/Byron/dua-cli/commit/ffdb0c270f9c07a3518e2335ee77d7788bfc7793))
</details>

## v2.12.0 (2021-05-29)

YANKED.

- Add minimal progress for when `dua` invocations take longer than 1 second

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 20 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Only display progress on if stderr is a tty ([`a0d6288`](https://github.com/Byron/dua-cli/commit/a0d628898226e272e9f29137da148991e07f3641))
    - Add simple progress to indicate something is happening on long `dua` runs ([`e68481f`](https://github.com/Byron/dua-cli/commit/e68481f3524d214b76d2895a10febc3a524c3256))
    - Thanks clippy ([`78a68b1`](https://github.com/Byron/dua-cli/commit/78a68b1a9ed5d39d250c5478041e40425a198756))
</details>

## v2.11.3 (2021-05-09)

- re-add arm builds
- dependency updates (including tui 0.15)

## v2.11.2 (2021-05-03)

- dependency updates (including tui 0.15)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 40 calendar days.
 - 70 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix help menu typo ([`98d973f`](https://github.com/Byron/dua-cli/commit/98d973fdf1cea099bfe963e9b1736ab2cac08a35))
    - Thanks clippy ([`59279d4`](https://github.com/Byron/dua-cli/commit/59279d464aac8c3985720d1d46b0a190b4443d2f))
</details>

## v2.11.1 (2021-02-22)

<csr-id-59315b7c63b7328fa70bfe5fc43fdbe9dc5f92e7/>

- The `-x/--stay-on-filesystem` flag is now respected for multiple root paths, as in `dua -x
path-FS1/ path-FS2/`, as such `dua` will stay in FS1 if the CWD is in FS1.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 7 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Respect 'stay_on_filesystem' when no input files are provided ([`33f81d6`](https://github.com/Byron/dua-cli/commit/33f81d6f56d1c324548a7b6d8a06bac168821516))
</details>

## v2.11.0 (2021-02-15)

### Features

- Add binding capital 'H' to go to the top of any pane/list
- Add binding capital 'G' to go to the bottom of any pane/list

### Fixes

- Without user input during `dua i [<multiple paths>]` the top-most entry will remain selected.
- Avoid stale frame at the end of traversal in interactive sessions when there is no user input.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 23 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Enforce drawing once after traversal is done ([`ee73690`](https://github.com/Byron/dua-cli/commit/ee7369022611745ec9c55beddf1b907f13ed3559))
    - Keep selecting the first element during iteration unless… ([`6d7b3cd`](https://github.com/Byron/dua-cli/commit/6d7b3cd062214f2cc66886d49d1a60406204abf3))
    - Thanks clippy ([`6ca9e6c`](https://github.com/Byron/dua-cli/commit/6ca9e6ca52a4d4d32036df2914ee773ab313397b))
    - Add bindings 'H' and 'G' to go to the top/bottom of any pane ([`8b606ac`](https://github.com/Byron/dua-cli/commit/8b606ac464ec5fa3979ab73fef4d29733d389760))
</details>

## v2.10.10 (2021-01-23)

<csr-id-9384cdb5b95e5260f46ccd23e7ca276304190a34/>

Fix --version flag.
It looks like the latest BETAs of clap removed setting the version implicitly.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 16 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix --version ([`1ba3c1c`](https://github.com/Byron/dua-cli/commit/1ba3c1cce9ae9419633f1e197b76c87649e9174a))
</details>

## v2.10.9 (2021-01-07)

Fix build.

Now that `jwalk` was released in v0.6 with v0.5.2 yanked, `cargo install` will use the previous
version v0.5.1 which does not fit the latest `dua` anymore.

This is now fixed and hopefully permanently so thanks to using `jwalk` v0.6.

## v2.10.8 (2021-01-04)

<csr-id-dc100c8b4a838c92f39d5a67da7eea06e7dec9af/>

Fix build.

A breaking change in jwalk can cause builds to fail. This prevents the issue from spreading at least
with dua-cli.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 19 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Dependency update ([`420f1f6`](https://github.com/Byron/dua-cli/commit/420f1f677b77acd73729df19edf2849c65d8d33b))
</details>

## v0.14.0 (2021-01-04)

## v2.10.7 (2020-12-16)

Better performance on Apple Silicon (M1).

The IO subsystem on Apple Silicon is different and won't scale nicely just by using all amount of available cores. Instead it seems best to only
use as many threads as performance cores are present on the system - otherwise the performance might actually get worse while using more power.

On all other systems, the default number of threads did not change.

**Please note that for optimial performance** one would need an arm build on MacOS, currently provided is only intel builds.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 31 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Select better default thread count on Apple Silicon (M1) ([`a1cf012`](https://github.com/Byron/dua-cli/commit/a1cf012f36269d97953baac9288b2fc5551bc6a0))
</details>

## v2.10.5 (2020-11-15)

Dependency update.

- upgrade to TUI v0.13.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Custom usage to fix #71 ([`018b00d`](https://github.com/Byron/dua-cli/commit/018b00db339f9772922007e293567231164b330b))
    - Switch from structup to clap 3 beta.2 ([`5782c4f`](https://github.com/Byron/dua-cli/commit/5782c4ff99b70ea101ed2f36711a456fd4e4e37b))
</details>

## v2.10.4 (2020-11-15)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release over the course of 13 calendar days.
 - 31 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Show 'scanning' note even without entering a directory ([`8992625`](https://github.com/Byron/dua-cli/commit/8992625fe2bfc8ceb371a86733bb3900e4caf3d9))
</details>

## v0.13.0 (2020-11-15)

## v0.0.1 (2020-10-26)

## v2.10.3 (2020-10-15)

Dependency update.
Should fix [this issue](https://github.com/Byron/dua-cli/issues/66)

## v0.12.0 (2020-09-28)

## v2.10.2 (2020-07-27)

Change light-grey color in command-line mode to Cyan to fix disappearing text.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 3 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Refactor ([`cdc5ee3`](https://github.com/Byron/dua-cli/commit/cdc5ee36d2c7c6bc6ecc9676ebaa408066a9eb5a))
    - Src, aggregate: fix colors for aggregate mode ([`4d2e839`](https://github.com/Byron/dua-cli/commit/4d2e83904fd66a3d480b5f50ad6fa2192d113a3f))
</details>

## v2.10.1 (2020-07-24)

Change light-grey color in interactive mode to Cyan to fix disappearing text.

See [this PR](https://github.com/Byron/dua-cli/pull/62) for reference.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix styling for folders (cyan=folders, not chagned - regular files) ([`2cc6916`](https://github.com/Byron/dua-cli/commit/2cc69169282a07a485992bf95969cf6f81981b08))
    - Fix clippy warnings ([`292c4d3`](https://github.com/Byron/dua-cli/commit/292c4d30722592b3e5ab1d779b5502cb0d129999))
</details>

## v2.10.0 (2020-07-22)

Minor improvements of looks; improved windows support.

- previously in interactive mode on Windows, directory sizes would appear as 0 bytes in size. This is now fixed!

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 10 calendar days.
 - 15 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Minor style improvements to handle special case ([`69a2490`](https://github.com/Byron/dua-cli/commit/69a2490844d87c09cd5cc51da49e3cd87a03c35a))
    - Avoid jump when cycling through byte visualization ([`4f91292`](https://github.com/Byron/dua-cli/commit/4f912929f213c00f6721995bfc5ee0b8879d80e9))
    - Fix mark pane ([`b4476ba`](https://github.com/Byron/dua-cli/commit/b4476bac270e2d1cdeb0f28bf7528d95b770a7e3))
    - Help is back to normal ([`8c2a174`](https://github.com/Byron/dua-cli/commit/8c2a174ed31cfc6e7095cf1cf4dbc24bf38ea975))
    - Help looks better now, but is far from 'normal' ([`29ee421`](https://github.com/Byron/dua-cli/commit/29ee421dd40666c53f659692a9a55cf8874cee1a))
    - Switch to crosstermion 0.3 for tui 0.10 support ([`fd8c441`](https://github.com/Byron/dua-cli/commit/fd8c441af3739027b7959a21b530ddb4da455f73))
    - Merge remote-tracking branch 'origin/master' ([`4812206`](https://github.com/Byron/dua-cli/commit/4812206eab68ea5588d93f9ea0589f9e772ee5ad))
    - Upgrade to tui 0.10 step one… ([`839b932`](https://github.com/Byron/dua-cli/commit/839b9323d93b9f562f6414cd66504b6d686c0224))
    - Fix path construction of 'sample_02_tree' for test ([`5a36cd1`](https://github.com/Byron/dua-cli/commit/5a36cd18a31ca1fbdc62d4e594933a6327fe4e7d))
    - Fix platform size difference of 'sample_01_tree' for test ([`62c5833`](https://github.com/Byron/dua-cli/commit/62c58330b41cb19adde1c7d2b08a5db251be3580))
    - Re-enable test, disabled accidentally ([`48cbe09`](https://github.com/Byron/dua-cli/commit/48cbe0919da1dd6aa8c933b5d156e7f0ce5997a8))
    - Update to colored 2.0 ([`72e776d`](https://github.com/Byron/dua-cli/commit/72e776d9a3668a81a9502e9560c06a2e500a37c8))
    - Fix test on windows - it's breaking now since #53 is fixed ([`1207bdd`](https://github.com/Byron/dua-cli/commit/1207bdd582c75895354b639fb81006d97076da83))
    - Don't pay extra on linux for helping with #53 ([`d18191d`](https://github.com/Byron/dua-cli/commit/d18191d8b19471eabc34526070bcc440edd72626))
    - Use full path for obtaining the 'real size on disk' ([`22a13fb`](https://github.com/Byron/dua-cli/commit/22a13fbea06199151d5cdf2f3a0533984111e0b3))
    - Replace flume with just std::sync::mpsc ([`ba78ae4`](https://github.com/Byron/dua-cli/commit/ba78ae433d1ea905bf1efd751cec34901e509caa))
</details>

## v0.10.1 (2020-07-22)

## v0.10.0 (2020-07-22)

## v0.4.1 (2020-07-10)

## v2.9.1 (2020-07-07)

Globs for Windows; fixed handling of colors.

- On widnows, `dua` will now expand glob patterns by itself as this capability is not implemented by shells `dua` can now run in.
- A bug was discovered that could cause `dua a` invocation to now show paths behind their size in an incorrect attempt to not print with color.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'rivy-fix.win' ([`edd0d74`](https://github.com/Byron/dua-cli/commit/edd0d74a12096f83c4b75ffd021c31dcbc269a46))
    - Fix color handling (causing the text to disappear); fix tty detection ([`82d005b`](https://github.com/Byron/dua-cli/commit/82d005b9e3ed9ce8d4441c607ec160f2f0a48b1c))
    - Add windows wildcard argument support (using `wild`) ([`2c73b4d`](https://github.com/Byron/dua-cli/commit/2c73b4d59603c12d31ded1a2f2ca9ef97a5ff0b3))
    - Fix windows compiler warnings (unused_variables) ([`5a11216`](https://github.com/Byron/dua-cli/commit/5a11216b53af2644100fcfebe44b0b6eea2dbb78))
</details>

## v2.9.0 (2020-07-06)

Full windows support!

- On Windows, we will now build using `crossterm`, which was greatly facilitated by `crosstermion`.
- On Unix systems, the backend is still `termion`.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Skip one test on windows ([`fece423`](https://github.com/Byron/dua-cli/commit/fece4231cd24409b0772a820cee18c2922d45e5b))
    - Make interactive mode optional, allow selection of backend for windows, unix ([`464829e`](https://github.com/Byron/dua-cli/commit/464829e11f5d6d63019ec167e2e1b1b7c0061f0a))
    - Completely rid ourselves of Termion to make backend selection possible ([`0e760d7`](https://github.com/Byron/dua-cli/commit/0e760d733108a7e3a2153b4cee03f33ef13e5cd4))
    - Replace termion::color with colored ([`40e9eb1`](https://github.com/Byron/dua-cli/commit/40e9eb1d0e548dac3ec896d293291d1e439ba976))
    - Termcolor spends 1200 lines on handlings buffers, and it's not liking plain io::Write ([`e867e58`](https://github.com/Byron/dua-cli/commit/e867e58ebd2febc66342f0337f08b75574b24e02))
    - For a moment I thought 'colored' could be used, but… ([`86f16c3`](https://github.com/Byron/dua-cli/commit/86f16c3042d9f8ba400512c8f2916c3a40e2d1f8))
    - Always use crossterm for now just to test if it works and… ([`3e0d4b0`](https://github.com/Byron/dua-cli/commit/3e0d4b022ff8d6ce5115894f3b6ad68f01ff370f))
    - Use crosstermion to create a terminal with the corresponding backend ([`98f850a`](https://github.com/Byron/dua-cli/commit/98f850a1ccd30618620a7d78999899c24463238a))
    - Allow case-insensitivity with byte format variants ([`4b59c36`](https://github.com/Byron/dua-cli/commit/4b59c36ca8c53e63dd74fc0b3179a4ed9de2f60d))
    - Convert input handling to crosstermion ([`388a134`](https://github.com/Byron/dua-cli/commit/388a1347580df120cead11f98516ceb911373316))
    - Show possible variants of byte formats ([`fddc8cb`](https://github.com/Byron/dua-cli/commit/fddc8cbcadb50a6ad2bf06e883fe751f3bca55b3))
</details>

## v2.8.2 (2020-07-02)

- Switch back to `clap` from `argh` to support non-UTF-8 encoded paths to be passed to dua

I hope that `argh` or an alternative will one day consider supporting os-strings, as it would in theory be an issue
for anyone who passes paths to their command-line tool.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Make aliases visible in generated docs ([`531fbf1`](https://github.com/Byron/dua-cli/commit/531fbf1d5b4107cc54a426559e552d818e1d5735))
    - Bring structopt back, argh doesn't support OsStrings ([`e32778b`](https://github.com/Byron/dua-cli/commit/e32778b00dd38bc2053d325453ec19f498b68a29))
</details>

## v2.8.1 (2020-07-02)

- Switch from deprecated `failure` to `anyhow` to reduce compile times a little and binary size by 130kb.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Use 'anyhow' instead of 'failure' to simplify code and reduce bloat ([`af7a09c`](https://github.com/Byron/dua-cli/commit/af7a09c53faf9ebeeb8c0a15278b510738d1f34f))
</details>

## v2.8.0 (2020-07-02)

- Switched from `clap` to `argh` for a 300kb reduction in binary size and 1 minute smaller compile times.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - All tests work with argh (which really needs aliases) ([`03e9a2a`](https://github.com/Byron/dua-cli/commit/03e9a2ac143c269d2c44a6bd13a0da10ede8bf38))
    - First version of options struct based on Argh ([`d787a9c`](https://github.com/Byron/dua-cli/commit/d787a9c5b8ccadae678c985b05ecc328d62df8f3))
</details>

## v2.7.0 (2020-07-02)

- [Support for extremely large][issue-58], zeta byte scale, files or filesystem traversals.
- [Fix possibly incorrect handling of hard links][pr-57] in traversals spanning multiple devices.

Both changes were enabled by [@Freaky](https://github.com/Freaky) whom I hereby thank wholeheartedly :).

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 32 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Use u128 for byte sizes ([`1d8ba52`](https://github.com/Byron/dua-cli/commit/1d8ba524ac83a0c3b5e4146cf937ed75650f1e97))
    - Fix inode filtering with multiple devices ([`c37ee44`](https://github.com/Byron/dua-cli/commit/c37ee449f32ed3af0fc222f669ae3f40859d8a39))
</details>

## v2.6.1 (2020-05-31)

- quit without delay from interactive mode after `dua` was opened on huge directories trees.
  See [this commit](https://github.com/Byron/dua-cli/commit/91aade36c71e4e14167030b6ec8c3c13dcdc1b2b) for details.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 11 calendar days.
 - 27 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Avoid deallocation a potentially big hashmap ([`91aade3`](https://github.com/Byron/dua-cli/commit/91aade36c71e4e14167030b6ec8c3c13dcdc1b2b))
    - Add windows-by-handle feature to lib.rs, where it probably has to be ([`cc1930a`](https://github.com/Byron/dua-cli/commit/cc1930ab6c387628cd1f2ba3499d64b7a523ad5f))
    - Fix crossdev to support windows (as originally intended) ([`3884ea6`](https://github.com/Byron/dua-cli/commit/3884ea66d74a0a04beb24e7c12144ac8245d4b95))
</details>

## v2.6.0 (2020-05-04)

- Use `x` to only mark entries for deletion, instead of toggling them.
- Add `-x` | `--stay-on-filesystem` flag to force staying on the file system the root is on, similar to `-x` in the venerable `du` tool.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 21 calendar days.
 - 29 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade to tui 0.9 ([`42c541a`](https://github.com/Byron/dua-cli/commit/42c541ac1977cef5169981c5996820214da9c937))
    - Add '-x' flag to not cross filesystems ([`9156cf7`](https://github.com/Byron/dua-cli/commit/9156cf7cac8f91a496f7383940f3ce6140ffe54c))
    - Fix cargo fmt ([`a5988d0`](https://github.com/Byron/dua-cli/commit/a5988d091b437315a91accd21f6f1b61d21e2e9a))
    - Add 'x' key to mark for deletion, without toggling ([`5cedded`](https://github.com/Byron/dua-cli/commit/5cedded25d10800805d6717381bf2981e270e23d))
    - Mild refactor ([`5c1a04b`](https://github.com/Byron/dua-cli/commit/5c1a04bb108eefdb6e10294fef0681cf92ecbaad))
    - Fix clippy lints ([`83804ad`](https://github.com/Byron/dua-cli/commit/83804adf605c2d1264b0fcafcdbf5f77023570ab))
</details>

## v2.5.0 (2020-04-05)

Much more nuanced percentage bars for a more precise visualization of space consumption.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 5 calendar days.
 - 6 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix compile errors after porting commit ([`26b9569`](https://github.com/Byron/dua-cli/commit/26b9569472ffb300d7019dbed5524fdbf688c6b8))
    - Add eighth sections to bar ([`82333ac`](https://github.com/Byron/dua-cli/commit/82333ac619e95a0635c20e9bc16b364b5f520e2d))
    - Clippy ([`70b043a`](https://github.com/Byron/dua-cli/commit/70b043abfd4a5765b4966cff65a7b67c518528ef))
</details>

## v0.3.0 (2020-04-03)

## v2.4.1 (2020-03-30)

Bugfix: Update currently visible entries when scanning.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update currently visible entries whenever we get the chance during scanning ([`8b3a32f`](https://github.com/Byron/dua-cli/commit/8b3a32f9d99a26ac62e150ae6a2cb5fa835a8055))
</details>

## v2.4.0 (2020-03-29)

Full interaction during scanning phase; add inline-help for better UX.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 24 commits contributed to the release.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Don't try to shutdown keyinput thread to not lose input events ([`80979a1`](https://github.com/Byron/dua-cli/commit/80979a179f924af87a33fc81ccca055ce6df5636))
    - First step towards support aync/channel based input events ([`e811eff`](https://github.com/Byron/dua-cli/commit/e811effe6424cd691260b07d1187d7c2d34ad4f1))
    - Toggle help for entries and mark pane ([`7689016`](https://github.com/Byron/dua-cli/commit/7689016c537d054a519e4e61c577e30645537213))
    - Navigation help for 'help' pane :D ([`d5ed498`](https://github.com/Byron/dua-cli/commit/d5ed498b592ff2b7f725163cae0c8426930c005c))
    - Auto-help which follows through the panes ([`ac04d9e`](https://github.com/Byron/dua-cli/commit/ac04d9ed9992090cfaf0002c2da954fefd542241))
    - Crossbeam channel is actually not needed in this case ([`a3cf6d6`](https://github.com/Byron/dua-cli/commit/a3cf6d6f3ea68d4cc91a433b4e3701e698f27009))
    - Show 'scanning' message even without key presses. ([`1f1c0ce`](https://github.com/Byron/dua-cli/commit/1f1c0ce5171ec691152954d3169a266e760ea873))
    - Allow initial scan to be interrupted properly… ([`277824b`](https://github.com/Byron/dua-cli/commit/277824b2aeedfa1f82fa2675f17e2498230b9fe7))
    - Allow deletion of files while scanning, it should yield IOerrors only; improve 'scanning' message ([`8c3294e`](https://github.com/Byron/dua-cli/commit/8c3294e67c4a140be335816720d6c0e5d021319b))
    - Fix crashbug - division by zero… ([`5f2bc2d`](https://github.com/Byron/dua-cli/commit/5f2bc2d38205cc66b7bb1805b5a1544e8ccfaae2))
    - Now it's way more intuitive, and you can basically do everything… ([`164d885`](https://github.com/Byron/dua-cli/commit/164d8859ea0a1386dbd75a0a27dd0340e6605857))
    - Better state handling when 'peeking' during traversal… ([`d7d9a8b`](https://github.com/Byron/dua-cli/commit/d7d9a8bdd55ce6fccdc51d238e55e769c314205c))
    - Properly shutdown dua with quick-exit - solves all problems ([`437eb41`](https://github.com/Byron/dua-cli/commit/437eb41def66eedf4614902e42eb1d265967093c))
    - Surprisingly complicated to get back to normal TTY without dropping the terminal… ([`13e5695`](https://github.com/Byron/dua-cli/commit/13e5695ea499d84f508748d120d282f55cb288f5))
    - Now there could possibly be abortable and navigatable GUI while scanning… ([`0e25706`](https://github.com/Byron/dua-cli/commit/0e25706db7e25d53678b23548eddf5809a789ab4))
    - Assure we keep display state changes ([`b556405`](https://github.com/Byron/dua-cli/commit/b5564057fd999a87a7e0f9470964d05595f12556))
    - Remove now unused method ([`1ceb264`](https://github.com/Byron/dua-cli/commit/1ceb264ee9393b6adec68781100ee962ae8e3656))
    - Phase one of refactoring nearly complete ([`758ea32`](https://github.com/Byron/dua-cli/commit/758ea32b90547c9f9c8f3135f3e7fa422111e44a))
    - Also exit quickly when ctrl+c is pressed ([`00e7006`](https://github.com/Byron/dua-cli/commit/00e70066ea495af9464b9d12cfd8ef15a40c6584))
    - On the way to separating traversal from application state ([`ede6224`](https://github.com/Byron/dua-cli/commit/ede622480acb4066ea864bae200ea89de46dbcdd))
    - Revert "Asynchronous processing of keyboard events…" ([`81bd12a`](https://github.com/Byron/dua-cli/commit/81bd12a176666ca5dacdb651f2e7f2b017c41ff2))
    - Another step towards isolating the event loop from needing to own the traversal tree… ([`733fac3`](https://github.com/Byron/dua-cli/commit/733fac38e2095fdc819b584958092381b9e2bc46))
    - Asynchronous processing of keyboard events… ([`7f32fb9`](https://github.com/Byron/dua-cli/commit/7f32fb9a70dd9b7078ae4db8e465d6762336048a))
    - Cleanup 'quick-hack' done in 2.3.9 - much better now ([`9824585`](https://github.com/Byron/dua-cli/commit/9824585960f09729c5547d60edaea5d97fdb595f))
</details>

## v0.2.2 (2020-03-29)

## v2.3.9 (2020-03-27)

Do not follow symlinks unless it's the only root path to follow.

This brutally fixes an issue where symbolics links are honored when they are placed in the current working directory, as internally `dua` will
treat each cwd directory entry as individual root path.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Truly don't follow symlinks unless they are the only top-level path. ([`768cbce`](https://github.com/Byron/dua-cli/commit/768cbce3963be7d6ece448d56289223810d678ac))
</details>

## v2.3.8 (2020-03-26)

`dua interactive` (`dua i`) is now about twice as fast due to using all logical cores, not just physical ones.
This is also the first release with github releases: https://github.com/Byron/dua-cli/releases/tag/v2.3.8

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Considerably speed up dua interactive by allowing to use all (logical) cores ([`085ae37`](https://github.com/Byron/dua-cli/commit/085ae37d70bbd4328e046a47bc41c13e669eb562))
</details>

## v2.3.7 (2020-03-24)

<csr-id-45d1ef31181cd9b430d855a4fe23550ea97e685e/>

Upgrade to filesize 0.2.0 from 0.1.0; update dependency versions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 8 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update to filesize v0.2 ([`cf902db`](https://github.com/Byron/dua-cli/commit/cf902dbc2cc7b80b2657cf2429db708cc71b6253))
</details>

## v2.3.6 (2020-03-16)

Upgrade to jwalk 0.5 bringing better threading control and no symlink following during traversal

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Now we are truly single-threaded when threads = 1 ([`b7ed2bb`](https://github.com/Byron/dua-cli/commit/b7ed2bbc957c416e8af08983bba46a4fe2a9553c))
    - Add marker for future improvement : parallel deletion ([`394e261`](https://github.com/Byron/dua-cli/commit/394e2615d5fb2cbde9ddb076f1e4867a4161e05a))
    - Jwalk 0.5 has landed - now we don't follow symlinks during traversal! ([`0d6116e`](https://github.com/Byron/dua-cli/commit/0d6116eea1e741bc8bc1fc6d04536c8242c5aa42))
</details>

## v2.3.5 (2020-03-15)

Fast exit from interactive mode for a responsive exit; dependency updates (except jwalk)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Revert "Upgrade to jwalk 0.5; stop following symlinks during traversal" ([`d2fda42`](https://github.com/Byron/dua-cli/commit/d2fda42dca410a9319f3f08b24545cbd8b8f1f59))
</details>

## v2.3.4 (2020-03-15)

YANKED - jwalk 0.5.0 wasn't used correctly which led to a performance regression

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade to jwalk 0.5; stop following symlinks during traversal ([`4990fa4`](https://github.com/Byron/dua-cli/commit/4990fa4202f2b687ee2476efe0a406fdfe23fd96))
    - Adapt journey tests to changed signature ([`b26f8ff`](https://github.com/Byron/dua-cli/commit/b26f8ff07730c6d0ba21cd2db398539a1252bf7a))
</details>

## v2.3.3 (2020-03-14)

YANKED - journey tests failed to changed method signature.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 18 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Exit the program directly to avoid latency ([`175de56`](https://github.com/Byron/dua-cli/commit/175de56ebe0aff01f7e67de9862d98ba0970feea))
</details>

## v2.3.2 (2020-02-25)

Incude the license file in crate.

## v2.3.1 (2020-02-23)

Include .md files in Crate, update dependencies.

## v2.3.0 (2020-02-22)

Show size on disk by default; Dependency Update.

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/37), hard links are now not counted anymore.
The `-l` flag will count hard links as it did before.

And of course, this has no noticable performance impact.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Rename 'count-links' to more descriptive 'count-hard-links' ([`db514fe`](https://github.com/Byron/dua-cli/commit/db514fe58c234ad312156814ba6f5ee7b7af0b60))
    - Merge branch 'Freaky-hardlink-tracking' ([`a6a4cf3`](https://github.com/Byron/dua-cli/commit/a6a4cf3705ba764ca0862fd3faaf0f7df31ac28d))
    - Cargo fmt ([`ba7b071`](https://github.com/Byron/dua-cli/commit/ba7b071af53444cf33ed6a11aae02b34bc26c82b))
    - Add hardlink tracking, and an option to disable it ([`5b52294`](https://github.com/Byron/dua-cli/commit/5b522946adb5bb71dd51068eee5f1136e6403b31))
</details>

## v2.2.0 (2020-02-22)

Show size on disk by default; Dependency Update.

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/35), the old apparent size can be displayed with the
`-A` flag, and the much more useful 'size on disk' is now shown by default.

To my pleasant surprise, this does not seem to affect performance at all - everything stays speedy.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 21 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge branch 'Freaky-apparent-size' ([`4db48ce`](https://github.com/Byron/dua-cli/commit/4db48ce218f12e11bbf6727fab6fb58c142b1a33))
    - Add support for real/apparent size ([`d86e1e0`](https://github.com/Byron/dua-cli/commit/d86e1e0f66ac8bd031233a6a54e2a1694acf1142))
</details>

## v2.1.13 (2020-02-01)

Dependency Update; Github Releases.
Binaries for Linux and MacOS are now available on GitHub Releases.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 87 calendar days.
 - 101 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update tui to 0.8 ([`d871bc0`](https://github.com/Byron/dua-cli/commit/d871bc044028edf6e1cdb4cdcb1c59176648c129))
    - Update all dependencies to latest version ([`543f7f3`](https://github.com/Byron/dua-cli/commit/543f7f3948c26250a8fc6ebf79a49f3ddfa3cb63))
</details>

## v2.1.12 (2019-10-23)

More obvious highlighting of active panel.

Depending on the terminal used, it might not have been obvious which panel was active. This might be
confusing to new and current users.
Now the color of the widget frame is changed to light gray, instead of remaining gray.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 89 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Make sure borders are drawn more priminently on focus ([`70c8d44`](https://github.com/Byron/dua-cli/commit/70c8d44b8ac42170989aa2e892cf44f79b9ab4c2))
</details>

## v2.1.11 (2019-07-26)

Finally fix symlink handling.

`dua` will not follow symbolic links when deleting directories. Thank a ton, @vks!

_Technical Notes_: Handling symbolic links properly is impossible without usage of `symlink_metadata()`.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Don't follow symlinks when calculating size interactively ([`6b235de`](https://github.com/Byron/dua-cli/commit/6b235de6f43af0f7573275c2b205741f326fd4cf))
    - Don't follow symlinks when deleting files recursively ([`e01f157`](https://github.com/Byron/dua-cli/commit/e01f157d708eb1cf5cdef0daff843eda98c5db76))
</details>

## v2.1.10 (2019-07-25)

Compatibility with light terminals.

- the TUI is now usable on light terminals, and highlighting is more consistent. Thank you, @vks!
- Fixes misaligned columns when displaying '100.00%' alongside other rows by displaying `100.0%` instead. Thanks, @vks, for pointing it out.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 2 calendar days.
 - 4 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - A single decimal slot for percentages; Fixes #26 ([`44aa899`](https://github.com/Byron/dua-cli/commit/44aa8997e3b18214f7177f7c6cc36a25daafbf24))
    - Run rustfmt; use debug_assert; rename function ([`fa7daf1`](https://github.com/Byron/dua-cli/commit/fa7daf1be9b67d70c3cde64cecdd4a76d2e8082b))
    - Use same colors in mark pane as in entries pane ([`3baf7f3`](https://github.com/Byron/dua-cli/commit/3baf7f31b91c71ba0acb2be886a47ccbd2b295fb))
    - Fix color scheme for light terminals ([`977e69f`](https://github.com/Byron/dua-cli/commit/977e69f9aafc54f9b2ed9ddb2eee5164e30b213c))
    - Forbid unsafe everywhere ([`f4028ba`](https://github.com/Byron/dua-cli/commit/f4028baf655e2994459e55d62435de4456fee80f))
</details>

## v2.1.9 (2019-07-21)

Improved handling of broken symlinks.

- during symlink deletion, now broken symlinks will be deleted as expected.
- always return to the previous terminal screen so the TUI doesn't stick to the current one.
- display broken symlinks on the first level of iteration.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 6 calendar days.
 - 7 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Show broken symlinks on the first level of iteration ([`eb015d3`](https://github.com/Byron/dua-cli/commit/eb015d38cbe01ff6b04855ad94936cd8f59be4bc))
    - Handle broken symlinks, they can now be deleted ([`978ddba`](https://github.com/Byron/dua-cli/commit/978ddbae31a3769162cfb0fb1b6c95d96701d774))
    - Assure we flush stdout to switch back to the previous screen ([`8cdc2ea`](https://github.com/Byron/dua-cli/commit/8cdc2ea4decf7eceba3e01d67b64c41ab9ddcb26))
    - Allow for pageup/down to work in selector pane (interactive mode) ([`cb2bbdf`](https://github.com/Byron/dua-cli/commit/cb2bbdfe616b38311ebe26e78999c69a4637a5dd))
</details>

## v2.1.8 (2019-07-14)

Don't follow symbolic links when deleting directories.

[A critical bug was discovered](https://github.com/Byron/dua-cli/issues/24) which would lead to deletion
of unwanted `directories` as `dua` would follow symbolic links during traversal during deletion.

Please note that symbolic links to files would be treated correctly, only removing the symbolic link.

This is now fixed.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 11 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Do not follow symbolic links when iterating directories! ([`560a76d`](https://github.com/Byron/dua-cli/commit/560a76d43fa44c4ebf9bdc51087647bb800bbe68))
</details>

## v2.1.7 (2019-07-03)

Use latest version of open-rs.

That way, pressing `shift + O` to open the currently selected file won't possibly spam the terminal
with messages caused by the program used to find the system program to open the file.

Fixes [#14](https://github.com/Byron/dua-cli/issues/14)

## v2.1.6 (2019-07-03)

## v2.1.5 (2019-07-03)

- re-release with Cargo.lock

## v2.1.4 (2019-07-02)

## v2.1.3 (2019-06-16)

## v2.1.2 (2019-06-16)

Bug fixes and improvements.

- Performance fix when showing folders with large amounts of files
- Display of amount of entries per directory

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Performance improvements ([`d9dcbd0`](https://github.com/Byron/dua-cli/commit/d9dcbd0f89c1267f272f3cd7e9f9dd69d0ae145b))
</details>

## v2.1.1 (2019-06-16)

Bug fixes and improvements.

- Better information about deletion progress
- removal of windows support

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Better progress display when deleting multiple items ([`d586703`](https://github.com/Byron/dua-cli/commit/d5867038aa8d1d216c146fe8d0a919352dce4855))
</details>

## v2.1.0 (2019-06-16)

Bug fixes and improvements.

- windows support (never actually worked), usage of crossterm is difficult thanks to completely
  different input handling.
- additional key-bindings
- auto-restore previous selection in each visited directory

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Auto-restore previously selected entries; quality of life! ([`52f40ca`](https://github.com/Byron/dua-cli/commit/52f40caf557c4dfdae169b39984dd6fda1f77474))
    - Add 'h' and 'l' as alternative keybindings ([`251ea53`](https://github.com/Byron/dua-cli/commit/251ea53bbd5072a7e7315c610cbb59540f93c7a9))
</details>

## v2.0.1 (2019-06-16)

Bug fixes and improvements.

- fix typo in title
- better display of IO-Errors in aggregate mode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Error formatting suggestions ([`fba47e6`](https://github.com/Byron/dua-cli/commit/fba47e68757341b76b168ebf4d8b631a826712fc))
    - Add a missing "n" to the header ([`49bc227`](https://github.com/Byron/dua-cli/commit/49bc227d9b5adfcf27c78eca763a28ce51f26211))
</details>

## v2.0.0 (2019-06-15)

<csr-id-c67abaec3c573dbfaf31be22693220a49a67b262/>
<csr-id-a128eb4a6e675f148a203ac66de075ee0c0def1c/>
<csr-id-ef8cf5636f782024372f044af80f06ed030168b0/>
<csr-id-dacb897405c06f9468faa860e27f47d1d0e548bb/>
<csr-id-51ce1ed159d59c6e221af4df9a3f7da41b1820cb/>
<csr-id-6cbd4866b18de91d3702a55c45650615d67f5f30/>
<csr-id-7ad2130bada27098e2d24f06650873a53b159f87/>
<csr-id-49edb7654ce3380bcde28630645af3740cf1a07a/>
<csr-id-984bf4fcce05cd5d495511123c2c3b6906b96f6d/>
<csr-id-b4a2e0ee8f267ee50f92433e826fa9e42ff618db/>
<csr-id-b4669c0214a1bc858cf437a65583af7e4b9ec277/>
<csr-id-fcde45752a9b86ed606b78f522f6b6dd0de25457/>
<csr-id-01dd8e284224e42b59f317cd922d388f23def829/>
<csr-id-d42573e63a120c8c5a253b7be52f9c68fb72274b/>
<csr-id-c0aa567e81b54913df464c9b500fe7a20ada0ea5/>
<csr-id-f9a9cdf9f827a5e08b1bcc6035f908fdb971c9fd/>

Interactive visualization of directory sizes with an option to queue their deletion.
A sub-command bringing up a terminal user interface to allow drilling into directories, and clearing them out, all using the keyboard exclusively.

### Other

- Single Unit Mode, see [reddit](https://www.reddit.com/r/rust/comments/bvjtan/introducing_dua_a_parallel_du_for_humans/epsroxg/)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 211 commits contributed to the release.
 - 14 days passed between releases.
 - 16 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Handle symlinks in a rather brutal way. ([`209eecf`](https://github.com/Byron/dua-cli/commit/209eecf042761eba35be809ca22bc98af472acad))
    - Fix journey-tests ([`854dc46`](https://github.com/Byron/dua-cli/commit/854dc46e1d99ce5c089369820351b9354707a300))
    - Pane is now displayed during deletion; keeps last item selected ([`86e593f`](https://github.com/Byron/dua-cli/commit/86e593f0baee79a973845e4c7dae1339d3e838df))
    - This might be the first working version of deletion ([`08dfbb6`](https://github.com/Byron/dua-cli/commit/08dfbb633fe25cc922b898aaf367f26a08730d91))
    - Update num entries and bytes total ([`48813ae`](https://github.com/Byron/dua-cli/commit/48813ae0a1c9316b4a7ad1669de2c44389026769))
    - Usage of StableGraph fixes logic thus far ([`a3627c8`](https://github.com/Byron/dua-cli/commit/a3627c8d04b2a755a1e466745c84591ae8e9033b))
    - Better separation of concerns when iterating marked items ([`0fb99e0`](https://github.com/Byron/dua-cli/commit/0fb99e00453da6d63cc01af64fdab8419314763b))
    - First half-baked version of deletion within traversal tree ([`f8485c8`](https://github.com/Byron/dua-cli/commit/f8485c8d48fb231b113a6511ee4048712ccc27fc))
    - Refactor ([`1ce57a2`](https://github.com/Byron/dua-cli/commit/1ce57a29c45ee9896bfc529a13875dbc3859812f))
    - Refactor ([`afdbc1d`](https://github.com/Byron/dua-cli/commit/afdbc1dadcf6c1f1e6384f65b2cac5325a5bcf17))
    - First rough version of the required pieces in MarkPane ([`f1bc4cd`](https://github.com/Byron/dua-cli/commit/f1bc4cd689b7db594ceef89aa31c48b4166d21a2))
    - First sketch of the delete-draw-loop ([`60ba3e7`](https://github.com/Byron/dua-cli/commit/60ba3e7f5216030e7dd4a12355de6ac78999d8e1))
    - First test to fully verify deletion ([`c67abae`](https://github.com/Byron/dua-cli/commit/c67abaec3c573dbfaf31be22693220a49a67b262))
    - Move parts of the tests into their own files ([`a128eb4`](https://github.com/Byron/dua-cli/commit/a128eb4a6e675f148a203ac66de075ee0c0def1c))
    - Somewhere over China: preparation for splitting tests into modules ([`82b0ced`](https://github.com/Byron/dua-cli/commit/82b0ced5c18ae8dbe3730434e2447a013bb35480))
    - Somewhere over China: refactor deletion - now with error handling ([`406435b`](https://github.com/Byron/dua-cli/commit/406435beff334d8f0ad62560176774ede2771ecd))
    - Somewhere over China: Let's not be quite so ignorant about errors during deletion ([`eb4f978`](https://github.com/Byron/dua-cli/commit/eb4f9780d69824b9ca389f42b2ec65077640cd54))
    - Recursive deletion - tests can begin ([`ef8cf56`](https://github.com/Byron/dua-cli/commit/ef8cf5636f782024372f044af80f06ed030168b0))
    - Simple recursive copy - deletion would like depth-first though ;) ([`dacb897`](https://github.com/Byron/dua-cli/commit/dacb897405c06f9468faa860e27f47d1d0e548bb))
    - Basic for test with writable directory ([`51ce1ed`](https://github.com/Byron/dua-cli/commit/51ce1ed159d59c6e221af4df9a3f7da41b1820cb))
    - Make marker selection feel right ([`6cbd486`](https://github.com/Byron/dua-cli/commit/6cbd4866b18de91d3702a55c45650615d67f5f30))
    - Nicer colors for warn window in selection ([`7ad2130`](https://github.com/Byron/dua-cli/commit/7ad2130bada27098e2d24f06650873a53b159f87))
    - Warning window follows user selection ([`49edb76`](https://github.com/Byron/dua-cli/commit/49edb7654ce3380bcde28630645af3740cf1a07a))
    - Fix handling of deleting the first index in the mark list ([`984bf4f`](https://github.com/Byron/dua-cli/commit/984bf4fcce05cd5d495511123c2c3b6906b96f6d))
    - More prominent selection in mark pane ([`b4a2e0e`](https://github.com/Byron/dua-cli/commit/b4a2e0ee8f267ee50f92433e826fa9e42ff618db))
    - Rustic way of handling the mark panes disappearance ([`b4669c0`](https://github.com/Byron/dua-cli/commit/b4669c0214a1bc858cf437a65583af7e4b9ec277))
    - Don't show warning if nothing is marked anymore ([`fcde457`](https://github.com/Byron/dua-cli/commit/fcde45752a9b86ed606b78f522f6b6dd0de25457))
    - Actually hook up spacebar in mark pane ([`01dd8e2`](https://github.com/Byron/dua-cli/commit/01dd8e284224e42b59f317cd922d388f23def829))
    - Make help window pretty again ([`d42573e`](https://github.com/Byron/dua-cli/commit/d42573e63a120c8c5a253b7be52f9c68fb72274b))
    - Better handling of what is selected after removing a marked entry ([`c0aa567`](https://github.com/Byron/dua-cli/commit/c0aa567e81b54913df464c9b500fe7a20ada0ea5))
    - Don't try to go down as marked items are removed ([`f9a9cdf`](https://github.com/Byron/dua-cli/commit/f9a9cdf9f827a5e08b1bcc6035f908fdb971c9fd))
    - Fixed Up and Down key inputs and added Left and Right for Ascent and Descent navigation ([`eae992f`](https://github.com/Byron/dua-cli/commit/eae992fbf0b0f0adaf8feffcb0e4903deabc562e))
    - First version of removing marked items from the list ([`3b71763`](https://github.com/Byron/dua-cli/commit/3b717634364647139388dffd0d68ce6c9729eee9))
    - Only show hotkey for deletion when focus is on the mark pane ([`05ed8c4`](https://github.com/Byron/dua-cli/commit/05ed8c494a1201daa4daa1506455a52f8b2b5b8e))
    - First version of help line which tells what to do to delete things ([`f34ceeb`](https://github.com/Byron/dua-cli/commit/f34ceeb91f41298278f4be62a053308946d41ea7))
    - Happier clippy ([`f83942b`](https://github.com/Byron/dua-cli/commit/f83942b40cd545ee7b6b18e091c273d27a8610a8))
    - Grapheme handling when truncating long filenames ([`0994466`](https://github.com/Byron/dua-cli/commit/0994466c45e4a46769c6998d87cf532e80108af3))
    - First prettier version of mark pane ([`28d84fc`](https://github.com/Byron/dua-cli/commit/28d84fc18f3efc7cfd4aa1728656998e652e934b))
    - Proper scrolling in mark pane ([`6bd6556`](https://github.com/Byron/dua-cli/commit/6bd6556449daae40fdabedf64866b641785787f5))
    - Merge pull request #8 from tsathishkumar/master ([`047e424`](https://github.com/Byron/dua-cli/commit/047e424d4fee8061b55a3253b8829ad1ffb84f0c))
    - Happy clippy ([`3fc9beb`](https://github.com/Byron/dua-cli/commit/3fc9beb205a2ad5f1da00472a6bc1a94cc64e769))
    - Assure we don't keep threads around unnecessarily in interactive mode ([`95685f1`](https://github.com/Byron/dua-cli/commit/95685f1387b74e2bbd7c1e67d383cd5861aa3451))
    - Refactor ([`24e1e2c`](https://github.com/Byron/dua-cli/commit/24e1e2cc3345e6891ec12c821b425ebc91f41d8d))
    - Move EntryMarkMap into Mark widget ([`141efd0`](https://github.com/Byron/dua-cli/commit/141efd025dabd0f94f7b195400900ccb2db9049a))
    - Moved marked information from footer to title of mark pane ([`6cb2d92`](https://github.com/Byron/dua-cli/commit/6cb2d92aa41e179242bb926b965862d90f06df82))
    - Maintain sorting even though we have a map - each render must allocate now ([`8d21dbb`](https://github.com/Byron/dua-cli/commit/8d21dbb3a44aeaf3989c25d9555559b34632f8c7))
    - See how it is when sorting by alphabet ([`5cff69c`](https://github.com/Byron/dua-cli/commit/5cff69c47a5b92017e6b1c55a35fd97f08ab3181))
    - Tests to verify focus handling works ([`65321d7`](https://github.com/Byron/dua-cli/commit/65321d786aa105f3f99ea43144f9f4b5a4ee4574))
    - Fix tests - if there is no item, there is no pane ([`80f7a06`](https://github.com/Byron/dua-cli/commit/80f7a0629954d05c3397f80cd0f9a74ae0a3f002))
    - Implement actual marker selection ([`6ba885e`](https://github.com/Byron/dua-cli/commit/6ba885e247b4d9d886b6867483c90b8dc0e5e7ae))
    - Know about focus in marker pane ([`2dafff4`](https://github.com/Byron/dua-cli/commit/2dafff434f9e772d779ec71a2fd8de1e5d2780db))
    - Simplify mark selection by making it based on position in list ([`beed74a`](https://github.com/Byron/dua-cli/commit/beed74aec250823aa01f33925f2a877414c5526c))
    - Refactor ([`d319f0b`](https://github.com/Byron/dua-cli/commit/d319f0b3b293167b4dfef79fed25b305cd1309e1))
    - Fix header highlight logic, quite literally ([`0a266d3`](https://github.com/Byron/dua-cli/commit/0a266d362a11ffd420806cc49ac6884815b0b915))
    - Move ownership of marked entries to the MarkPane ([`9ffacd0`](https://github.com/Byron/dua-cli/commit/9ffacd03e256b45ecd40744e5507f37c30ae9b5e))
    - Some experimentation with selection handling in the new pane ([`4c354f4`](https://github.com/Byron/dua-cli/commit/4c354f475bfe841f3797be0a3341212aeeaa60c8))
    - A step towards more self-contained components ([`29c0cf3`](https://github.com/Byron/dua-cli/commit/29c0cf3c5a584764e060dd9f34592edbc8098562))
    - Reactor help: move event handling closer to where it belongs ([`04f5324`](https://github.com/Byron/dua-cli/commit/04f5324b17efe4c7b62a0afc7d2b34304a9a4407))
    - Refactor ([`4cde0f6`](https://github.com/Byron/dua-cli/commit/4cde0f6892f29a16694155ec25d94f4ce3c3d0c9))
    - The first display of paths to be deleted! ([`b79b1ae`](https://github.com/Byron/dua-cli/commit/b79b1aee4ebe97034da0804f5d1dae2bfedd1210))
    - Color header based on mark and pane focus state, for dramatic effect! ([`f54a5aa`](https://github.com/Byron/dua-cli/commit/f54a5aa7aef7f5a29131db485154607bedc4da23))
    - The first incarnation of the mark window ([`98aa1df`](https://github.com/Byron/dua-cli/commit/98aa1df3e99be5543dbc7ade969de3373cc132ea))
    - Fix issue with seeing nothing when trying to enter a file ([`96121b5`](https://github.com/Byron/dua-cli/commit/96121b55802e2ba038129cafafc48910e29a8a8f))
    - Fix endless loop and infinite memory consumption due to... NAN!! ([`0718d2a`](https://github.com/Byron/dua-cli/commit/0718d2a2a1f8ac16f0bbd30b520a3804e09eab41))
    - Let's not get ahead of ourselves ;) ([`399391a`](https://github.com/Byron/dua-cli/commit/399391a3d72ca099b30f7bc2c0468ce845c71798))
    - Get rid of black percentage bars :D! ([`1f9cb8e`](https://github.com/Byron/dua-cli/commit/1f9cb8e8ad4f0908bf1ab068765ac9898b402328))
    - Better help ([`3c76c0f`](https://github.com/Byron/dua-cli/commit/3c76c0f408a0bfe4eea271c5a77c4911c39c8eee))
    - Inform about marked entries in the footer ([`dd898c6`](https://github.com/Byron/dua-cli/commit/dd898c6a3e045782970b8496e888adf661e382c2))
    - Coloring for marked entries ([`22902a5`](https://github.com/Byron/dua-cli/commit/22902a5889ab36303aed53c0d2fe57a3be919474))
    - Preparing for displaying the marked state in entries list ([`2f3f214`](https://github.com/Byron/dua-cli/commit/2f3f214e03de477ad05aa12a1ac2ba0775a36c14))
    - Remove Widget trait from the Header ([`53add13`](https://github.com/Byron/dua-cli/commit/53add13094a39751158f8cae27988bcbee47d08d))
    - Refactor ([`7bef597`](https://github.com/Byron/dua-cli/commit/7bef5974e86de825dcb0b3507df16a80b6986d88))
    - More hotkeys ([`eec9803`](https://github.com/Byron/dua-cli/commit/eec980374f7ada8c002d7f8d1663307552f801ab))
    - Fix sorting; add some alternate keys ([`f2e4504`](https://github.com/Byron/dua-cli/commit/f2e45047015ec2c08777513a366db92af0ae3586))
    - Clear screen at initialization ([`37ce7fe`](https://github.com/Byron/dua-cli/commit/37ce7fe923ad76e9c6b24a462b3cb258eef88607))
    - Refactor ([`c33ae7c`](https://github.com/Byron/dua-cli/commit/c33ae7c7d9f538490346a8532e27c3dd6c4aa21d))
    - Assure we see something while scanning - entries are now manually provided ([`2c1cb19`](https://github.com/Byron/dua-cli/commit/2c1cb19aeb89d25977bd9fa76b8572d7e7d942a7))
    - The block is now not needed anymore - we can just own simple props ([`42fb0cc`](https://github.com/Byron/dua-cli/commit/42fb0cccb10ce1084267b63b07a5a0a8bf84de99))
    - Finally, everything was properly ported to tui-react ([`7549e82`](https://github.com/Byron/dua-cli/commit/7549e82fa1afc3fd87af6e42c13757a1c11994ea))
    - Entries is now ReactEntries :) ([`ae679ed`](https://github.com/Byron/dua-cli/commit/ae679ed0daed2f2faf1bd8b4db922bdf450f738a))
    - Add tui-react as library - it's proven (enough)... ([`3aa9b01`](https://github.com/Byron/dua-cli/commit/3aa9b0168425706b6bdfa4eb2b9335da24bc15fd))
    - Make clear the Component is very a TopLevelComponent, very special! ([`80ae2ac`](https://github.com/Byron/dua-cli/commit/80ae2ac79c1525886c613452c835099eeae97c4d))
    - FINALLY! It works, and is on the way to using tui-react ([`c5fd940`](https://github.com/Byron/dua-cli/commit/c5fd9402a19ea427375751c7dfe61153897a273f))
    - What about simply not implementing the trait :D? Concrete types for the win! ([`180ebb7`](https://github.com/Byron/dua-cli/commit/180ebb77b28ad4ecb4bebc44173f8b3b9338dc41))
    - Removed propsmut in the hope it will work then, but not quite (yet?) ([`f8b3a0b`](https://github.com/Byron/dua-cli/commit/f8b3a0b38aaffbf8f2d78cd9147545f3d905b63b))
    - Revert "An attempt to make it better by removing BorrowMut... to no avail, but different error" ([`8059e8b`](https://github.com/Byron/dua-cli/commit/8059e8b8d292fc9ab1ec54a957c0531b7106711f))
    - An attempt to make it better by removing BorrowMut... to no avail, but different error ([`b9c485a`](https://github.com/Byron/dua-cli/commit/b9c485a6e4fe629014ac1ddcc56bd2a78f7b7c66))
    - The first attempt to actually use the ReactList - it's just insane... ([`4e1a326`](https://github.com/Byron/dua-cli/commit/4e1a32631874f49a048ba42b0deb5c6277118934))
    - Extract react to directory ([`9cb8f4f`](https://github.com/Byron/dua-cli/commit/9cb8f4f40a2f8fc6e3f927f81459a4baafb25c31))
    - An elegant solution to the Block rendering problem - it's not a component after all... ([`c799ac9`](https://github.com/Byron/dua-cli/commit/c799ac925fc79b218bf0ff7c6f37e81980e755c6))
    - List compiles, but block still makes trouble ([`39938fb`](https://github.com/Byron/dua-cli/commit/39938fb193aeca619d9d37bb78b977f64182be05))
    - Add react block for use in react-style components ([`b6004e2`](https://github.com/Byron/dua-cli/commit/b6004e24a96bfbfad2743418d2e2bf7647c78120))
    - Support for mutable props - useful for iterators for example ([`b2f5187`](https://github.com/Byron/dua-cli/commit/b2f518764a28800ac911904f7b1e59daa08e6948))
    - Add ReactFooter ([`9a5ffd2`](https://github.com/Byron/dua-cli/commit/9a5ffd238470b511c4818e917f55ba4dafaf212c))
    - Help pane is now a component :) ([`c243521`](https://github.com/Byron/dua-cli/commit/c243521ea7466e9584ff0455f409b2a4160c4fb4))
    - First moderately working step towards react-tui mode ([`3f3fe77`](https://github.com/Byron/dua-cli/commit/3f3fe77d1679f867928d70d8e844f0041d26bf35))
    - Now it work, borrowmut was the problem ([`705f4b8`](https://github.com/Byron/dua-cli/commit/705f4b842175de7375058fff54455ba3204dffe0))
    - First attempt to demo it... fail due to type inference issues? ([`717abd7`](https://github.com/Byron/dua-cli/commit/717abd71158166847c43bc60a2208345186994c4))
    - First sketch of component ([`eebef81`](https://github.com/Byron/dua-cli/commit/eebef816f307d941e428a27e8871830b73c1cdae))
    - Cleanup terminal ([`cb12e94`](https://github.com/Byron/dua-cli/commit/cb12e94cb9c2cad8007e1230f21f2e1380858835))
    - Basis for react-like terminal implementation - that way we can have state ([`b3ebbfc`](https://github.com/Byron/dua-cli/commit/b3ebbfc1e76292a401e20595928815f83ab83373))
    - Use entries from the state contained in the parent app ([`03d2ee3`](https://github.com/Byron/dua-cli/commit/03d2ee3e65abb7522dfe8a7802cebfb9b93cb44e))
    - EntryDataBundle with all data we need: next - don't query during draw ([`8f3daee`](https://github.com/Byron/dua-cli/commit/8f3daee851d305d61d6efd39ce8c562f06a744a4))
    - Step 1: we store entries as we enter/exit nodes ([`7483ddb`](https://github.com/Byron/dua-cli/commit/7483ddb97d754dea3415a4906082bcf0f85eb818))
    - Sorted entries now fetches the Path as well, prep for entries refactoring ([`4a1220e`](https://github.com/Byron/dua-cli/commit/4a1220eabf30db015463312000be7a2574c6e582))
    - Show missing files in red. Also reveals: we need to refactor entries... ([`cade6b1`](https://github.com/Byron/dua-cli/commit/cade6b1dab7d17f3f277ed288d9498a9b435f65a))
    - Make app.rs into module directory, incl. further splits ([`e9a8614`](https://github.com/Byron/dua-cli/commit/e9a8614152b6f719cc748c377ffe863b19a50b7e))
    - Move sorted_entries closer to where it is used ([`50438ef`](https://github.com/Byron/dua-cli/commit/50438ef584d5f2ade0a0501ebca151c99893580f))
    - Move application tests closer to... the application. Nice! ([`b0a02d3`](https://github.com/Byron/dua-cli/commit/b0a02d30f97d15e0c6fc19e5f4f7b8c56500ff7a))
    - Moved 'interactive' portion of code into binary - break unit tests for now ([`80f01db`](https://github.com/Byron/dua-cli/commit/80f01dbfcce5c5c6d482a47d9f04fd5a0f8e75c0))
    - Typo :D ([`240cc7a`](https://github.com/Byron/dua-cli/commit/240cc7a2de6116c999b048445587d99d8a656e84))
    - Use most verbose visualization by default after scanning ([`39ad2a8`](https://github.com/Byron/dua-cli/commit/39ad2a80997c62f2c02fcd8cede591c0e5d303c4))
    - Smoother visualization cycle ([`fcdc355`](https://github.com/Byron/dua-cli/commit/fcdc355fd8ebb187d144f6e3160fc74e21a0df41))
    - Add Percentage and Bar at the same time!!! :D ([`5bde50f`](https://github.com/Byron/dua-cli/commit/5bde50f3f034aa833a8ea916542213ad0d1f6b1e))
    - Add long bar visualization ([`59ad2e6`](https://github.com/Byron/dua-cli/commit/59ad2e66a269703aa7dc76ecd0398df1105f286d))
    - Let byte visualization control its own width ([`a765f23`](https://github.com/Byron/dua-cli/commit/a765f232c3ad76ba5f688353aa37f02c46b42ec8))
    - Cycle through graph and bar options ([`b0ea97f`](https://github.com/Byron/dua-cli/commit/b0ea97f6afa62019792bf0fcd73368ae4b9fbd85))
    - First Bar implementation ([`5551c01`](https://github.com/Byron/dua-cli/commit/5551c0107fbe8a4a0ca9226e37d488b1f3c62dc7))
    - Support for changing the percentage display ([`097bce8`](https://github.com/Byron/dua-cli/commit/097bce870f4294e83f2062c4f80304004e8556a0))
    - Add support for static byte units ([`a1ecbf0`](https://github.com/Byron/dua-cli/commit/a1ecbf0a1a68ca7bb9f4e372e89b66ac3a945264))
    - Add a decent header line ([`9d430a2`](https://github.com/Byron/dua-cli/commit/9d430a23d950edabfbeca55ba4805c48dfde99a3))
    - Reformat ([`c8914ab`](https://github.com/Byron/dua-cli/commit/c8914abc499682fc60fa1e88fdaabc1140d0be7f))
    - Wow, help scrolling is finally working! ([`09373b2`](https://github.com/Byron/dua-cli/commit/09373b26b8f6da9a3a2407a54b0735d41a960278))
    - Tried to keep count of lines, but failed... it's hard to avoid allocations ([`31a90d7`](https://github.com/Byron/dua-cli/commit/31a90d7748678448d41b025d65981097fec26af3))
    - Scrolling for the help window ([`7219392`](https://github.com/Byron/dua-cli/commit/72193928f6ef957def962d304de465510fb09b93))
    - Implement tab key ([`1d1c351`](https://github.com/Byron/dua-cli/commit/1d1c3516432500fcf77f41146ad0119a2d97014f))
    - Refactor ([`9fcc4fe`](https://github.com/Byron/dua-cli/commit/9fcc4fee324bb28ccdb900113a1ee42177bdeb45))
    - The reamining hotkeys explained ([`5ece6f7`](https://github.com/Byron/dua-cli/commit/5ece6f74eaa5cbfbc5205f4f7ad486e6ad6c410f))
    - Ready for the next paragraph ([`2b2bd4e`](https://github.com/Byron/dua-cli/commit/2b2bd4ea9a848d5e79ad5cc630fd86b1df2c93fd))
    - Don't quit hard when hitting 'q' ([`5d30eb6`](https://github.com/Byron/dua-cli/commit/5d30eb65f91bc5a6ef501cb7c4e2d242762a02ea))
    - Help comes to live, slowly ([`286bfd4`](https://github.com/Byron/dua-cli/commit/286bfd4cb2e3416fda987ff8ea9a6b70397b9970))
    - Divert input events depending on the focus ([`e522160`](https://github.com/Byron/dua-cli/commit/e522160a66a770d88371922b479fc1f3837022b7))
    - Nicer focus tracking ([`622b163`](https://github.com/Byron/dua-cli/commit/622b1630087135c60414b7947a37b8a145e7031f))
    - First simple focus tracking ([`c19df21`](https://github.com/Byron/dua-cli/commit/c19df218c6addbbcbae9feccdfed4a75693be260))
    - First sketch on how help window could work ([`13dd5b2`](https://github.com/Byron/dua-cli/commit/13dd5b289c73aab5caa1d06e5580635e88ef81ad))
    - Mild refactoring ([`17fe6f8`](https://github.com/Byron/dua-cli/commit/17fe6f8bccd81a7c2e2f6f8b72a2576589089725))
    - Pretty colors in interactive mode ([`b7de02e`](https://github.com/Byron/dua-cli/commit/b7de02e35cd18fc596541a6561fcd617013ec8ce))
    - Save an allocation ([`017be14`](https://github.com/Byron/dua-cli/commit/017be1445de9dad942aba164b15b41d24d0866f8))
    - First compiling version of paragraph list + entries ([`ce9df24`](https://github.com/Byron/dua-cli/commit/ce9df2498ae07a49f65b63c73838d3fc8b1e9ae6))
    - Rename 'human*' formats to their non-prefixed counterpart ([`d13adea`](https://github.com/Byron/dua-cli/commit/d13adea1958081e430703be84829b3c03c5f3e26))
    - Properly fix byte column width handling ([`a5c8e37`](https://github.com/Byron/dua-cli/commit/a5c8e37b970169913ab72ea691b89aeeeffad403))
    - Refactor ([`7d451f9`](https://github.com/Byron/dua-cli/commit/7d451f968908549babd06e7858d7a5263b1737a3))
    - Implement list with paragraphs ([`593b10f`](https://github.com/Byron/dua-cli/commit/593b10f2dba54e78093e51ebf8621e5bb88a8401))
    - First sketch of 'better' list - support for paragraphs for each item ([`a5a7c06`](https://github.com/Byron/dua-cli/commit/a5a7c0606f33e125f375110ee06db828295b02e7))
    - Continuous lines for entry items ([`0121a64`](https://github.com/Byron/dua-cli/commit/0121a648c4445f3cd807f53c6ba914cd8507e40d))
    - Fix byte formatting ([`2022a51`](https://github.com/Byron/dua-cli/commit/2022a51ce4960923fc5376d8d9b10185319c8c34))
    - Prettier footer - one-line paragraphs for the win ([`9abc39b`](https://github.com/Byron/dua-cli/commit/9abc39ba9435ff994c0262417af9bd46abb76774))
    - Better message handling ([`1dec5d4`](https://github.com/Byron/dua-cli/commit/1dec5d49faf04e60047b3823ca7b23b8b4b9499a))
    - Move list scrolling code into list state ([`e3b0a25`](https://github.com/Byron/dua-cli/commit/e3b0a2585a110fecbfedb007e01b057deee3daaf))
    - Proper entries list scrolling ([`3a10bfe`](https://github.com/Byron/dua-cli/commit/3a10bfef5b3611beb1ef778eb6fa46d7f7a62009))
    - Now widgets can just update their drawstate at will ([`9247af6`](https://github.com/Byron/dua-cli/commit/9247af6d91bdd7bef2d9a49b27d09c0b7f77a8da))
    - Since performance doesn't matter here, always update widget state ([`1d27826`](https://github.com/Byron/dua-cli/commit/1d27826999f4a60d17c0d2b9a76b604edd2aa343))
    - A version with manual update and mutable widget state (even during draw) ([`156c842`](https://github.com/Byron/dua-cli/commit/156c84264e0d1a967e7c5039596e88282c38dbf0))
    - Using utility types would work, but shows it's too enforcing ([`6f81e63`](https://github.com/Byron/dua-cli/commit/6f81e63c78999b03dfecaef73f6b2ce6f397c88a))
    - Non-mutable widget state ([`971e235`](https://github.com/Byron/dua-cli/commit/971e235153f57dd87c763e8c0a07a3f79ad7375c))
    - Sketch to see how mutable widget state would look like ([`7ce062f`](https://github.com/Byron/dua-cli/commit/7ce062f010508bac368f389f4cadd2f6cc44df62))
    - Refactor ([`f6f6a7d`](https://github.com/Byron/dua-cli/commit/f6f6a7d4d7c8886236ddca4bfa3a7d7a7d4a3d9c))
    - It shows that making the stateless GUI work with list scrolling... needs state ([`92c636c`](https://github.com/Byron/dua-cli/commit/92c636c0f0cd38c10f2f76b16c6d70c159909e1b))
    - Separate modules files for widgets ([`74dc7e0`](https://github.com/Byron/dua-cli/commit/74dc7e07813503c7c1c3d5ff0c6cd4b3f2d9ad01))
    - First step towards modularizing widgets ([`fa9f68a`](https://github.com/Byron/dua-cli/commit/fa9f68aca5bdc9dd5555a0acd8f9249044cbec6a))
    - Be sure to hide the cursor explicitly ([`2937b5d`](https://github.com/Byron/dua-cli/commit/2937b5d558c7c7aff00e8b08064ace3c4b77fc37))
    - Page up and down in navigation ([`a2b4c9c`](https://github.com/Byron/dua-cli/commit/a2b4c9cc42f92af949ad6002aa85d87684e7437c))
    - Removed support to change amount of storable nodes ([`2aad00a`](https://github.com/Byron/dua-cli/commit/2aad00a568b31120144a16e80965be0495cf036f))
    - Add support for messages in the footer ([`b255e63`](https://github.com/Byron/dua-cli/commit/b255e63193cbb5e8e09c169334df2b2c35e2a5e7))
    - The first version of list scrolling... works but funnily :D ([`6e21175`](https://github.com/Byron/dua-cli/commit/6e211754964fd9f1257be7fdeecc74b58543b120))
    - Refactor ([`85726c7`](https://github.com/Byron/dua-cli/commit/85726c71cdc0f1f83db626accfe7b0991b6c6dcd))
    - Refactor ([`5da79a5`](https://github.com/Byron/dua-cli/commit/5da79a52ccd25ae068b8f0c2ab4070d4529319c3))
    - Add 'O' to open a folder or file using the default program ([`4f4ea1e`](https://github.com/Byron/dua-cli/commit/4f4ea1e9b3813062ebe87032339bd4bcd87ee3b4))
    - Improve title display, deal with relative paths ([`5b4d44c`](https://github.com/Byron/dua-cli/commit/5b4d44c0121db981a61a838db18a5e6ccf4660bf))
    - Better title for entries, based on the paths your are in ([`74870ba`](https://github.com/Byron/dua-cli/commit/74870bae69ed9bfe34e75ef82e3d76bc6f98e160))
    - Move 'traverse' module out of 'interactive' - it's unrelated ([`fb57ebd`](https://github.com/Byron/dua-cli/commit/fb57ebd0423775c4c9b757a2fad588f8baa5beec))
    - Add 'u' key to go up one level ([`84b6f8c`](https://github.com/Byron/dua-cli/commit/84b6f8ce829e7a57604b4e983c91bc52a7299ac4))
    - Show directories very similar to ncdu ([`74e5116`](https://github.com/Byron/dua-cli/commit/74e511631a7f05143e487584a4325fe65c774ba5))
    - Add 'o' navigation ([`25ceae2`](https://github.com/Byron/dua-cli/commit/25ceae2779e3e20b4ff4ac3d6149410e5f851775))
    - Add 'k' navigation key ([`748dfc3`](https://github.com/Byron/dua-cli/commit/748dfc353a7d8c7bbb6bbfb097bacec18b80e32a))
    - Add 'j' key functionality for basic navigation ([`a76ad50`](https://github.com/Byron/dua-cli/commit/a76ad5009ac9177e1efb37130d1dcedb5df1e5de))
    - Compute percentage (at all), non-graphical for now ([`df0fe62`](https://github.com/Byron/dua-cli/commit/df0fe6279065ba060803e236a73336bdcf8fe4dd))
    - Preliminary styling for selected entries ([`90f94f7`](https://github.com/Byron/dua-cli/commit/90f94f79ac54689c4af47ad31e1080da725cd7ed))
    - Unify sorting to start dealing with selections ([`0b3e158`](https://github.com/Byron/dua-cli/commit/0b3e158085d68ba43dc3ac034ce4f0b5df9d61e8))
    - Test for handling the root correctly in interactive mode ([`59a3001`](https://github.com/Byron/dua-cli/commit/59a3001012d5ff40d050a1abfc370aaa248d8f66))
    - The first test for user input, yeah! ([`05c8ec1`](https://github.com/Byron/dua-cli/commit/05c8ec1a6e2ce9af3f55d75cb761cf3b66244bb8))
    - Prepare for mutable state in application, even more :D ([`11147d8`](https://github.com/Byron/dua-cli/commit/11147d8e344435b95adaca68e5125836c0bf2ed9))
    - Prepare for handling mutable application state ([`e48898b`](https://github.com/Byron/dua-cli/commit/e48898ba98312be9e77b2d5cc8a64a127ac59688))
    - Sorting by size, descending, for entries ([`e8cb9dc`](https://github.com/Byron/dua-cli/commit/e8cb9dcda01d5dc073dfb8093f66bd13d5699105))
    - Don't display '0' for total bytes while traversing ([`9720931`](https://github.com/Byron/dua-cli/commit/9720931800fd8e189c99cbf0cb01a31f23663744))
    - Assure root size is properly computed ([`dcf3a26`](https://github.com/Byron/dua-cli/commit/dcf3a2651b79493964feb16d8a2148e851a7b4ca))
    - Refactor ([`1f482aa`](https://github.com/Byron/dua-cli/commit/1f482aab49a9094234d422b3599858e909c3f164))
    - Separate Footer widget; refresh display before event loop ([`4112a9b`](https://github.com/Byron/dua-cli/commit/4112a9b971f36c69df8f8a07fdc2735edd862a45))
    - Bytes formatting for interactive + footer ([`7eb8574`](https://github.com/Byron/dua-cli/commit/7eb857467c6d2603129edbaea636ef0d118fa064))
    - Explicitly declare an init-window ([`b919c50`](https://github.com/Byron/dua-cli/commit/b919c501a249dcf626e390d496faf6d31a9e71ac))
    - Minimal event handling to allow viewing the TUI ([`7f4fb35`](https://github.com/Byron/dua-cli/commit/7f4fb350903fe32f513ddc39ff62de2c1d663e0f))
    - Pull out draw code into closure ([`4ec1d37`](https://github.com/Byron/dua-cli/commit/4ec1d37e01337ca22060e44dda36d71ffdc21146))
    - Prepare decoupling ([`598a6b0`](https://github.com/Byron/dua-cli/commit/598a6b0ec9582cdec27285d25ab09d0efa0b7db0))
    - Refactor ([`6cf44a1`](https://github.com/Byron/dua-cli/commit/6cf44a1658f4f34ffa295b49fbb4cc6cb7c75b9f))
    - Move modules into their own files ([`2ce606f`](https://github.com/Byron/dua-cli/commit/2ce606f607fa967f94d49c5413c4b347e628e88e))
    - First sketch of drawing code - it's quite neat and straightforward ([`24097bd`](https://github.com/Byron/dua-cli/commit/24097bd19ee53ca7a4a635e6ea63c3e3c63bdc2b))
    - Infrastructure for screen updates while gathering data ([`7c2628e`](https://github.com/Byron/dua-cli/commit/7c2628eedaa0d8b1bbe4dc9fbb3fbdc72de34c13))
    - Refactor - better, and it shows it's clearly two distinct things ([`2707445`](https://github.com/Byron/dua-cli/commit/2707445ec0fcfa42b4cb9e63114081bd43198742))
    - Refactor - still ain't pretty, but it's good enough for now ([`d4918ba`](https://github.com/Byron/dua-cli/commit/d4918ba23cd0a73a7d5c5ec419777261b5a30228))
    - Very hacky passing tests... let's refactor that! ([`59b2930`](https://github.com/Byron/dua-cli/commit/59b2930fb719954d793efa7bc586d61098d6ee21))
    - Add another failing test ([`00952c6`](https://github.com/Byron/dua-cli/commit/00952c6aa7b585cd27712ab75fd854d8cec11fc4))
    - And now it seems to work... not trusting it just yet ([`16833be`](https://github.com/Byron/dua-cli/commit/16833be086fe7c15b10e902ae309533dba5382d9))
    - Now computation actually works - next up is handling of the root ([`e03dd10`](https://github.com/Byron/dua-cli/commit/e03dd10b5f9f5593d6791968e40e8454ca7ea102))
    - Probably a bit closer to a correct implementation. ([`f0e53be`](https://github.com/Byron/dua-cli/commit/f0e53be0fe93c53269399b3c7c843266dcae5b88))
    - Add test showing sizes don't work, and graph traversal neither :D ([`dec4afc`](https://github.com/Byron/dua-cli/commit/dec4afc358aa30521d564068b219eca129245782))
    - One step closer to the actual tree-building implementation ([`7c3743d`](https://github.com/Byron/dua-cli/commit/7c3743d601cce407024e65570d108867a6196893))
    - First failing attempt to build a graph on demand ([`0774ecc`](https://github.com/Byron/dua-cli/commit/0774eccb72abfd800880cbc8490cb9899f1ab140))
    - First failing test - even though just a guess :D ([`68569c6`](https://github.com/Byron/dua-cli/commit/68569c69f5fdeedddd45635e8eb6d0c255de53f4))
    - First infrastructure for unit-level tests ([`1c53865`](https://github.com/Byron/dua-cli/commit/1c538654fba3caf7f7d601d6bf8a4af24faf19c8))
    - Basic frame to support new interactive mode ([`6d82a72`](https://github.com/Byron/dua-cli/commit/6d82a724b0452e417e20cbe8a02e3bed647e9674))
    - Highlight files with a different color ([`495ccbd`](https://github.com/Byron/dua-cli/commit/495ccbda25cb27dc912c07fbdb29651b83f32c68))
</details>

## v1.1.0 (2019-06-01)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Simplified handling of 'no paths given' case ([`ae0182f`](https://github.com/Byron/dua-cli/commit/ae0182f09c0e2c3c77298cb431421cbdc64c0fa8))
</details>

## 1.0.0 (2019-06-01)

Simple CLI to list top-level directories similar to sn-sort, but faster and more tailored to getting an idea of where most space is used.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 20 commits contributed to the release over the course of 3 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Now with colored help ([`3798be8`](https://github.com/Byron/dua-cli/commit/3798be8a31902a74f4c0280d0d1def8d6bb74d10))
    - Add minimal library documentation ([`310cd6a`](https://github.com/Byron/dua-cli/commit/310cd6af912cda7333496d5d5d80a68d6ea9b155))
    - Support for colors. Using green, which might be invisible to some! ([`9d09499`](https://github.com/Byron/dua-cli/commit/9d0949933cb46d2e73c047b5f06201dbd75bca1d))
    - Add simple statistics, just for fun! ([`498bcd0`](https://github.com/Byron/dua-cli/commit/498bcd0da4dc44d04634f2cabc245f4c46d2c46a))
    - Sort by size in bytes by default; can be turned off for immediate feedback ([`f8c3ba2`](https://github.com/Byron/dua-cli/commit/f8c3ba29134af08ea7b70b4fe3951307c6be6e3a))
    - Nicer formatting of numbers ([`e7da784`](https://github.com/Byron/dua-cli/commit/e7da7843ad7894a3560b4d70076a74798404da94))
    - Make explicit that Sorting is disabled during aggregation; more spacing ([`9ba5a34`](https://github.com/Byron/dua-cli/commit/9ba5a348c67a898abb0ae648e686da48649a33df))
    - Pull out all modules into files ([`8b2ef49`](https://github.com/Byron/dua-cli/commit/8b2ef49bf9f37d0e126fa68115175fe2cf82aaf5))
    - Add --no-total option ([`961b743`](https://github.com/Byron/dua-cli/commit/961b743773da2a5112bd4ab70554c50b03ded3ad))
    - Better error reporting ([`c1cbcf3`](https://github.com/Byron/dua-cli/commit/c1cbcf355755fbd1ca6124cdba3b8e361a7bebf2))
    - Support for paths specification without subcommand ([`c50332c`](https://github.com/Byron/dua-cli/commit/c50332cead2688e40de192e1b47e50a662763a78))
    - Compute the total if there are more than one paths ([`04ce0c9`](https://github.com/Byron/dua-cli/commit/04ce0c9312fb5e290d6fbaed8e9427bec3f3e1c6))
    - Support for various byte formats ([`7dc718b`](https://github.com/Byron/dua-cli/commit/7dc718bd03f7f669638d87b4c5fee67700f045ca))
    - Add byte formatting ([`6db07e2`](https://github.com/Byron/dua-cli/commit/6db07e2e69f7f674191311719054a245e8c8b886))
    - By not counting directories, we get the correct amount of bytes ([`a19e3d7`](https://github.com/Byron/dua-cli/commit/a19e3d76fe559f59be467b4967156509e6f26715))
    - Let's just say we compute the aggregate correctly ([`61ca52a`](https://github.com/Byron/dua-cli/commit/61ca52a2a8b23daffc3eea1fe8d71078e757a0d3))
    - An attempt to abstract link size, but it's not required actually :D ([`04f50bd`](https://github.com/Byron/dua-cli/commit/04f50bdcdbe995e7d9952788eb4cc4f736299c39))
    - First basic implementation of aggregation; symlinks are not handled yet ([`638be3c`](https://github.com/Byron/dua-cli/commit/638be3c8e7362b809c2c6558d630aa355349b1e8))
    - The first failing test ([`449f964`](https://github.com/Byron/dua-cli/commit/449f964850feb89d8a179bbc8a45cea6580577eb))
    - First instantiation of template ([`e9a4472`](https://github.com/Byron/dua-cli/commit/e9a447250ba9ffd10f94f6f7d970c6da141c185d))
</details>

## v2.10.6

Fix `dua -h` usage string.

## 1.2.0

The first usable, read-only interactive terminal user interface.
That's that. We also use `tui-react`, something that makes it much more pleasant to handle the
application and GUI state.

