# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 3.3.0 (2026-08-28)

### New Features

 - <csr-id-c0005aa9f8f1f225f609eb89c7d09a98ebbcb163/> reuse workers when restarting a walk
   <!-- agent -->
   Retain a completed single-root walk pool and add `Walk::restart()` so callers
   that rescan a changing directory avoid rebuilding worker threads.
   
   Declare Rust 1.88 support for let-chain and integer-cast syntax.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
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

