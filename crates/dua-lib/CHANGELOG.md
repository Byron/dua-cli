# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### New Features

 - <csr-id-c2c6f67d5bfc535e7d67ce695cf96697c74f1545/> stream roots and support cancellable walks
   Accept newly discovered roots in a running fixed-size worker pool. Keep
   per-root predicates and completion accounting, and prioritize submitted
   roots so existing directory trees cannot starve them.
   
   - Add `RootSender`, `stream_roots()`, and `Walk::next_cancellable()`.
   - Disconnect bounded output before joining workers during teardown.
   - Preserve native enumeration and bounded parallel metadata processing.
   - Cover independent completion, submission priority, input closure,
     full-channel shutdown, and cancellation.

### Performance

 - <csr-id-249bdf3beab910d0d233815a7717676f1a0f1739/> parallelize I/O-bound macOS directory scans
   This was an issue I encountered on APFS, a directory created by rustc with
   more than 1m files in it. In that case, bulk reading is slow (while efficient),
   and it turned out to be better to detect this and switch over to stat-based traversal.
   
   While looking at the filesystem probing code more closely (and how it's not dependent
   on CPU performance by differntiating wall time from kernel time), I basically
   rubber-stamped all the other code. Too much to look at, too foreign by now.
   But it did look cleaned up, so 👍.
   
   <!-- agent -->
   The reported target-directory scan left one worker waiting inside
   `getattrlistbulk` at about 15% of a CPU core. Native directory collection also
   held an entire parent directory before publishing entries and child jobs.
   
   Probe at most two initial bulk buffers before publishing them. Compare
   calling-thread user and kernel CPU time with elapsed time, and select the
   existing parallel stat queue when both refills spend more time waiting than
   executing. Reopen through ordinary enumeration only after that decision,
   discarding the unpublished probe to avoid mixed cursors or duplicate entries.
   Keep bulk reads when the probe is CPU-bound. The initial probe deliberately
   does not adapt to later cache changes.
   
   Share streaming native traversal between ordering modes, preserve parent
   ordering and APFS metadata, and bound queued metadata jobs by processing
   new batches inline when the queue fills.
   
   Regression coverage checks scale-independent timing decisions, bounded
   probing and metadata backlogs, streamed child jobs, parent ordering, and
   stat/native parity for clones, resource forks, hard links and symlinks.
   The timing test failed against the provisional fixed latency policy; the
   queue-bound test failed without backpressure.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 5 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge pull request #399 from Byron/dua-clean ([`ed276c2`](https://github.com/Byron/dua-cli/commit/ed276c230b838b94ea5af7daff10ae5996f7b455))
    - Stream roots and support cancellable walks ([`c2c6f67`](https://github.com/Byron/dua-cli/commit/c2c6f67d5bfc535e7d67ce695cf96697c74f1545))
    - Merge pull request #401 from Byron/faster-bulk-traversal ([`46b8556`](https://github.com/Byron/dua-cli/commit/46b8556d1c17770ebfa443144376441609fc9b71))
    - Parallelize I/O-bound macOS directory scans ([`249bdf3`](https://github.com/Byron/dua-cli/commit/249bdf3beab910d0d233815a7717676f1a0f1739))
</details>

## 4.0.0 (2026-09-07)

### Performance (BREAKING)

 - <csr-id-f643c7493c6ce48658983543d954fe4912f8e7fc/> avoid unnecessary metadata during deletion
   <!-- agent -->
   Deletion needs entry types, but every walk collected full sizes, timestamps,
   allocation information, and file identities. Add `Options::skip_metadata` and
   use directory-entry types without extra metadata lookups when available.
   macOS and Windows use standard enumeration for this mode; Windows roots
   query only attribute/tag information. Keep the native metadata readers for
   ordinary scans and preserve Windows verbatim-path handling.
   
   Drop per-entry deletion byte accounting. Successful removals use the existing
   scanned total; partial failures retain entry/error counts and display unknown
   bytes instead of collecting metadata solely for that notification.
   
   Local macOS/APFS release medians showed 0-11.5% lower deletion time, with
   wide-directory traversal up to 40% shorter; gains vary with tree shape.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 7 calendar days.
 - 8 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v4.0.0, safety bump dua-cli v3.0.0 ([`0e99238`](https://github.com/Byron/dua-cli/commit/0e992386d15275d08b4e1ef40196edb2a30dd799))
    - Merge pull request #397 from Byron/speed-up-deletions ([`97211e6`](https://github.com/Byron/dua-cli/commit/97211e69ec4e252760143b525ca4688a1309b544))
    - Avoid unnecessary metadata during deletion ([`f643c74`](https://github.com/Byron/dua-cli/commit/f643c7493c6ce48658983543d954fe4912f8e7fc))
    - Merge pull request #385 from Byron/io-format ([`3343158`](https://github.com/Byron/dua-cli/commit/3343158b234f9b45e2a0de4fe0ebe9dc56abbb28))
</details>

## 3.3.1 (2026-08-30)

### Performance

 - <csr-id-95136995ea39b35e6575f9ae1c5df9618ff5c0a9/> compact traversal storage and snapshot replay
   Admittedly, this one I waved through, but looked at the new storage for
   a more tightly packed tree closely.
   The parts with DirectoryId I just skipped over, as they are the most invasive
   overall and touch a log of places.
   Fine with me, everything seems to work, and this is beyond the time I can spend
   on reviewing, while the value proposition is too high to skip it.
   
   <!-- agent -->
   ## Traversal storage
   
   - Replace `petgraph` with a 64-byte arena-backed tree and compact stable indices.
   - Store native filenames in one append-only arena and reuse deleted node slots.
   - Route parents through dense directory IDs and keep glob matches outside the filesystem topology.
   
   ## Snapshot replay
   
   - Lend names from reusable record buffers and reuse sibling-order buffers by depth.
   - Copy names only for retained entries and sort snapshots directly from arena bytes.
   - Preserve the V1 encoding, validation, and platform-native path handling.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v3.3.1, dua-cli v2.44.0 ([`9553f5d`](https://github.com/Byron/dua-cli/commit/9553f5d9e6c5aef939df00c945fddb51f25da349))
    - Compact traversal storage and snapshot replay ([`9513699`](https://github.com/Byron/dua-cli/commit/95136995ea39b35e6575f9ae1c5df9618ff5c0a9))
</details>

## 3.3.0 (2026-08-28)

### New Features

 - <csr-id-c0005aa9f8f1f225f609eb89c7d09a98ebbcb163/> reuse workers when restarting a walk
   <!-- agent -->
   Retain a completed single-root walk pool and add `Walk::restart()` so callers
   that rescan a changing directory avoid rebuilding worker threads.
   
   Declare Rust 1.88 support for let-chain and integer-cast syntax.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v3.3.0 ([`6fed577`](https://github.com/Byron/dua-cli/commit/6fed577b904a12d548692978b34d187a0c74a86d))
    - Merge pull request #381 from Byron/reuse-walk-pool ([`28a7279`](https://github.com/Byron/dua-cli/commit/28a727921f55a5bb10449dfe2afa89a8eaffed39))
    - Reuse workers when restarting a walk ([`c0005aa`](https://github.com/Byron/dua-cli/commit/c0005aa9f8f1f225f609eb89c7d09a98ebbcb163))
</details>

## 3.2.0 (2026-08-25)

### New Features

 - <csr-id-ddee9e48fdd90575c91006ee52c4088ddf028b68/> Match macOS `du` allocation accounting
   Mirror Apple FTS complete common/file attribute request and packed-invalid records so macOS allocation totals match /usr/bin/du. Reject unknown vnode types, round allocation bytes to 512-byte stat blocks, and fall back to symlink metadata for directories, firmlinks, and incomplete records.
   
   Layer optional APFS clone and data-fork attributes onto the same packed layout so the upstream clone-deduplication path remains intact.

### Bug Fixes

 - <csr-id-b1e16a0cc5ea3792b477f5b38a2d98e88b4f53c3/> don't output colors when stdout isn't a terminal.
   This facilitates piping into a file.
 - <csr-id-a6b6f6899910ca4960a972f6299640dc5ae66eb7/> a relative root ending in .. is mis-resolved

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 9 calendar days.
 - 10 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v3.2.0, dua-cli v2.43.0 ([`7baa926`](https://github.com/Byron/dua-cli/commit/7baa9266a593fd6b7f7568c741798d76660eb3ab))
    - Prepare changelog priot to release ([`c042449`](https://github.com/Byron/dua-cli/commit/c0424490a47477dd516c3f52bbb6d130b210c317))
    - Merge pull request #379 from nshcr/codex/macos-du-allocation-accounting ([`2262295`](https://github.com/Byron/dua-cli/commit/226229565fc978e22a14ff2531044c688c87e1ed))
    - Review ([`df12088`](https://github.com/Byron/dua-cli/commit/df12088f444aa934ed922ab360279fabdf7a5ea0))
    - Match macOS `du` allocation accounting ([`ddee9e4`](https://github.com/Byron/dua-cli/commit/ddee9e48fdd90575c91006ee52c4088ddf028b68))
    - Don't output colors when stdout isn't a terminal. ([`b1e16a0`](https://github.com/Byron/dua-cli/commit/b1e16a0cc5ea3792b477f5b38a2d98e88b4f53c3))
    - Merge pull request #377 from tamird/perf-compact-apfs-metadata ([`8b1d8da`](https://github.com/Byron/dua-cli/commit/8b1d8da2316834b034449138a746779ffa78419f))
    - Review ([`d812272`](https://github.com/Byron/dua-cli/commit/d8122729a9f41b69f80cc7e794ecb56e7f9d174e))
    - Shrink native macOS traversal metadata ([`6156641`](https://github.com/Byron/dua-cli/commit/61566414896f380c82c647fa39b4809c01a857ed))
    - A relative root ending in .. is mis-resolved ([`a6b6f68`](https://github.com/Byron/dua-cli/commit/a6b6f6899910ca4960a972f6299640dc5ae66eb7))
    - Merge pull request #371 from tamird/macos-apfs-clone-accounting ([`7231d83`](https://github.com/Byron/dua-cli/commit/7231d838d6dad0f4a2ac649959788ba6ec844853))
    - Review ([`0acd5fa`](https://github.com/Byron/dua-cli/commit/0acd5fa421d287a1a8ef8c6d6efc8244e0622940))
    - Count fully shared APFS clones once ([`6b3a231`](https://github.com/Byron/dua-cli/commit/6b3a23176f69f5f18b3eb87529d585dfb2f5b9cf))
</details>

## 3.1.0 (2026-08-15)

Support providing walk roots with their metadata already set due to bulk-reading.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v3.1.0, dua-cli v2.42.1 ([`9560df3`](https://github.com/Byron/dua-cli/commit/9560df3113868662a5c1922299371e5b1f75ed25))
    - Prepare changelog prior to release ([`b32d912`](https://github.com/Byron/dua-cli/commit/b32d91204c72dedb77d4ce7987b711944b05ad2a))
    - Merge pull request #369 from tamird/macos-prepared-roots ([`3b1659e`](https://github.com/Byron/dua-cli/commit/3b1659e47ae28b3facd1d7d261132023b2509d1a))
    - Review ([`ffb2a2e`](https://github.com/Byron/dua-cli/commit/ffb2a2ef0fc3b95c4d2e0ef07f228699aa750a44))
    - Reuse bulk metadata for macOS aggregation roots ([`d45a2f8`](https://github.com/Byron/dua-cli/commit/d45a2f85451d900b19770f1e64bddd201e9d7429))
</details>

## 3.0.0 (2026-08-14)

macOS specificy bulk-readtree implementation for 30% performance boost on supported filesystems.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 9 calendar days.
 - 9 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v3.0.0 ([`ac9766c`](https://github.com/Byron/dua-cli/commit/ac9766cdd01b6ea7ff81e76a8fd9e9332d12cbcc))
    - Prepare changelog prior to release ([`40c2816`](https://github.com/Byron/dua-cli/commit/40c28166a4beb903e87df85e23916f27e15ef26b))
    - Merge pull request #367 from tamird/macos-native-traversal ([`e68868c`](https://github.com/Byron/dua-cli/commit/e68868c1fd28d089baa91126998dc25fb5216e23))
    - Review ([`872b6be`](https://github.com/Byron/dua-cli/commit/872b6beffaf3f354dee3e1f670d4fe45615b76a8))
    - Avoid per-entry macOS metadata queries ([`7d115c0`](https://github.com/Byron/dua-cli/commit/7d115c014eff2f85ab60e82266f4f1beb4782598))
    - Merge pull request #362 from Byron/dua-lib ([`b6e7caf`](https://github.com/Byron/dua-cli/commit/b6e7cafd305c150834eb887e1de99bcdd3fca85d))
</details>

## 2.41.1 (2026-08-05)

The first release of the directory walk implementation of the `dua-cli`, to allow its usage in other places as well.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-core v2.41.1 ([`9fe4ce0`](https://github.com/Byron/dua-cli/commit/9fe4ce0f644b823cfec79db1eda9b02ff55a1c37))
    - Prepare changelog prior to `dua-core` release ([`965ce7c`](https://github.com/Byron/dua-cli/commit/965ce7cdbca6f4d8954e2f45d1967f4df786ddb1))
    - Extract filesystem walker into dua-lib ([`3b1c8cf`](https://github.com/Byron/dua-cli/commit/3b1c8cfbf206d92f60a33049dd741251024a027f))
</details>

