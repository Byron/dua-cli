# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

This release fixes builds for free-desktop/linux, which is `not(target_os = "macos"), not(target_os = "ios"), not(target_os = "android")`

### Bug Fixes

- <csr-id-605160e89d37c9134698afd4eb4f1f37935ab0ee/> select the Unix trash backend explicitly
  <!-- agent -->
  Installing `dua-cli` with an unlocked dependency graph selects trash 5.3.0,
  which now requires either freedesktop or portal on supported Unix targets.
  dua disables dependency defaults, so the platform module was missing.
  The existing lockfile pinned 5.2.5 and hid the failure from CI.
  
  Require trash 5.3.0 and enable freedesktop only for its supported Unix
  targets, retaining the native macOS and Windows backends and the optional
  trash-move feature. Refresh the lockfile so CI builds the affected version.

### Commit Statistics

<csr-read-only-do-not-edit/>

- 3 commits contributed to the release over the course of 7 calendar days.
- 7 days passed between releases.
- 1 commit was understood as [conventional](https://www.conventionalcommits.org).
- 1 unique issue was worked on: [#428](https://github.com/Byron/dua-cli/issues/428)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

- **[#428](https://github.com/Byron/dua-cli/issues/428)**
  - Select the Unix trash backend explicitly ([`605160e`](https://github.com/Byron/dua-cli/commit/605160e89d37c9134698afd4eb4f1f37935ab0ee))
- **Uncategorized**
  - Merge pull request #429 from Byron/trash-build ([`e59cb79`](https://github.com/Byron/dua-cli/commit/e59cb79efebe2033f8cb29f0d99c06041c96666b))
  - Merge pull request #426 from Byron/api-check ([`12fa207`](https://github.com/Byron/dua-cli/commit/12fa207e8c23bf16cceff27c064d68c4a6feb3fc))

</details>

## 3.0.0 (2026-11-02)

### Documentation (BREAKING)

- <csr-id-c45461d32daf5ac7ffd0db682726d112be6c9870/> declare the Rust library API experimental
  Mark the crate root so all Rust library exports participate in experimental
  compatibility checks. Future API breaks require minor releases while patch
  compatibility remains checked. Document tilde requirements for library
  consumers and retain the stable CLI and dua-core compatibility policies.
  
  The transition removes the previous stable Rust library API promise and
  requires dua-cli 3.0.0. Let smart-release apply versions during release.

### Bug Fixes (BREAKING)

- <csr-id-d5114df1baad7f63c008fe725a6f4d4d0c4ce0e4/> retain filesystem paths in traversal errors
  Just looked at the production code and rubber-stamped the tests.
  All that really happens is a new error type that retains more information
  
  <!-- agent -->
  RootEvent and Walk previously yielded plain I/O errors, leaving consumers
  unable to identify a failing descendant. Return a typed dua_core::Error
  with the entry path, or the directory path when enumeration cannot identify
  an entry, and retain the original I/O error for kind and OS-code inspection.
  
  Keep sibling failures as separate iterator items. Update native and ordinary
  reads, streamed roots, prepared roots, cancellation, and CLI forwarding.
  Metadata errors remain attached to their Entry. Document the breaking API
  change and conversion back to io::Error.
  
  Regression coverage removes two sibling directories before descent in both
  orders and metadata modes; it also checks missing and streamed roots,
  original OS errors, source access, and conversion without losing context.
  The regression initially failed because io::Error has no path accessor.

## 2.46.0 (2026-10-15)

A bugfix release, and one that should also restore cargo installations, making them function.

### New Features

- <csr-id-2989d30d43c035c9c9ff9eb8eff8584b6244cf12/> make walk result buffering configurable per worker
  <!-- agent -->
  Replace the fixed QUEUED_BATCHES_PER_WORKER constant with
  Options::queued_batches_per_worker and a chainable builder method. Use
  two queued batches per worker by default in both production and tests,
  preserving the original bounded buffering behavior.
  
  Consumers that pause to process entries can opt into a larger queue to
  keep traversal workers busy, trading memory for less idle time. A value
  of zero selects an unbuffered channel. Check the worker-count
  multiplication so excessive capacities cannot silently overflow.
  
  Update existing Options literals to inherit the new default. Verify the
  actual queue capacity with one and two workers, and exercise larger and
  unbuffered queues through the public single-root and multi-root walkers.

### Bug Fixes

- <csr-id-afbe487bd0d7b158b6b4ef9e80b4e0113f55eee6/> pass strict workspace Clippy on Rust 1.99
  <!-- agent -->
  Replace empty-container assertions with equality comparisons and empty
  string checks with inequality comparisons. This satisfies Clippy's
  assert_is_empty lint while showing actual values when assertions fail.
  
  Locally allow the deprecated fetch_update alias in directory identifier
  allocation. Its replacement, try_update, requires Rust 1.95, while
  dua-core supports Rust 1.88. Preserve the existing atomic update and
  overflow handling without raising the minimum supported Rust version.
- <csr-id-92260297b8d7add5b9402deeedf715becc788816/> preserve flame graphs when export fails

## 2.45.1 (2026-09-30)

### Documentation

- <csr-id-d6acb509fe77af72f7cf41015e4f4a12f592d913/> correct platform defaults in threads help

### Bug Fixes

- <csr-id-2fbb8f5711fd503126bcaf652d3627f64eb78c1a/> don't fail on sole directory inputs beyond `MAX_PATH` on Windows
  A sole directory input is changed into so its entries can be listed and
  reported relative to it, but `set_current_dir` enforces `MAX_PATH` on
  Windows (os error 206) even though the directory itself can still be
  traversed through verbatim paths.
  
  When the change fails, expand the directory into absolute input paths
  directly instead of aborting, which makes a single long directory input
  work in list, aggregate, stacks and interactive modes alike. Reported
  paths are absolute then, consistent with how multiple inputs are already
  displayed.
  
  The expanded root is returned to callers so the interactive mode still
  knows which directory the inputs belong to, and recorded in the new
  `WalkOptions::base_dir`, which every traversal uses as the working
  directory stand-in for ignore-pattern matching. That keeps `--ignore-from`
  patterns matching descendants relative to the expanded root - identical to
  what a successful `chdir` would have produced.
  
  Fixes https://github.com/Byron/dua-cli/issues/410
- <csr-id-22d5588a63021599e6cb5c03e26d889f345d5a81/> don't panic on non-UTF-8 filenames when matching ignore patterns
  On Windows, NTFS filenames may contain unpaired surrogates that cannot be
  represented in UTF-8. `IgnorePatterns::is_excluded` converted each entry's
  relative path with the panicking `gix::path::into_bstr`, so a single such
  name anywhere in a scanned tree aborted the whole process when
  `--ignore-from` was used - in list, aggregate and stacks modes alike, as
  well as in `clean`'s entry filtering.
  
  Match on the OS string's encoded bytes instead - `OsStr::as_encoded_bytes`
  yields a self-synchronizing UTF-8 superset (WTF-8 on Windows, the raw bytes
  on Unix) which represents lone surrogates losslessly. The glob matcher then
  compares bytes, so wildcard patterns that don't spell out the invalid code
  unit, like `*` or `*.txt`, still match such names instead of silently
  letting them through.
  
  Fixes https://github.com/Byron/dua-cli/issues/408

## 2.45.0 (2026-09-12)

This release packs an incredibly useful new sub-command called `dua clean [--depth N]`,
to automatically and efficiently find typical build products that you may want to delete,
for Rust and Zig and Node for now.

Also, when deleting entries, the TUI does not block anymore, but shows the decreasing number
of bytes as files are deleted, until the marked entry finally disappers. The TUI can still be
used then, albeit in read-only mode.

Oh, and it also got faster on macOS when there are folders with a lot of files, when it will
switch aways from bulk-reads to multi-threaded stat calls instead, which in my case led to
a 2.5x speedup.

### New Features

- <csr-id-0661576c0271e493608d5875031614cda0aa9092/> add Git-aware dua clean and background deletion
  This commit was mostly rubber-stamped, but tested for a week or so before that.
  By now it's fair to accept that `dua` is in "tool that needs to work, without
  needing to be very familiar with how it works" territory, while I still look
  at the code at least but without the requirement to understand it all.
  
  <!-- agent -->
  Add `dua clean [--depth N] [DIRECTORY]...` to discover disposable build
  artifacts and caches, size accepted candidates, and offer them in a dedicated
  cleanup hub. Keep the TUI responsive during scanning, deletion, and trashing.
  
  ## Discovery and safety
  
  - Search directory entries and types before collecting size metadata. Recognize
  Node dependencies, Python caches and virtual environments, Cargo build output,
  and Zig caches and output. Require project markers for ambiguous names.
  - In Git repositories, require expendable ignore matches and reject tracked
  contents, conflict stages, gitlinks, and sparse ancestors. Consult HEAD when
  the index is absent and reject candidates when validation fails.
  - Reject nested repositories, case variants of Git markers, unreadable contents,
  excluded descendants, and disallowed filesystem boundaries.
  - Feed discovered roots into dua-core's running walker within the configured
  thread budget. Bound active candidates and publish each independently after
  sizing and validation; commit hardlink/APFS accounting only on acceptance.
  - Search without a depth limit by default; `--depth` bounds discovery. Support
  traversal filters and `--once`, with no snapshot import/export or parent
  scanning in clean mode.
  
  ## Cleanup hub and browsing
  
  - Title the hub with its input directories, group siblings and contained
  candidates, and keep rows in largest-first order as scan results arrive.
  - Open groups as scoped views of existing candidate nodes. Reuse ordinary
  sorting, glob search, annotations, navigation, and marking within the group;
  keep group membership and controls local to the hub.
  - Group marks target actual candidates. Complete partial marks and toggle fully
  marked groups off, preserving the real parent and unrelated contents.
  - Refresh recorded candidates without widening their deletion scope. Revalidate
  whole candidates with their original discovery roots, restore browser/search
  positions, and clear stale marks before tree indices are reused.
  
  ## Background deletion
  
  - Run deletion and trash work off the main thread, updating remaining bytes in
  the tree, mark pane, and progress message about once per second.
  - Allow navigation, search, sorting, pane controls, and suspend/resume while
  freezing marks and blocking further filesystem actions until the batch ends.
  - Preserve bounded filesystem concurrency, parallel file removal, and removal
  of child directories before parents. Batch tree updates using scanned sizes
  without double-counting overlapping marks or completed subtrees.
  - Keep failed entries with their remaining sizes and errors. On quit, cancel
  pending work, apply in-flight results, and join workers. On event-loop failure,
  cancel and disconnect events before joining. Trash progress advances when
  each native trash call returns.
- <csr-id-b15735b4aa512ce2183316100ecfe89467fdaaad/> minimize right-hand panels with ]
  A very quick review, but one that showed that the GUI probably shouldn't get more complicated.
  Or else it would be batter with more of a layout system than what it has now - manual layout.
  <!-- agent -->
  ## Behavior
  
  Add a configurable `toggle_right_panes` action, defaulting to `]`, that
  minimizes or restores the shared Help and Marked sidebar. Minimized panes
  display their translated names vertically with a blank column on each side:
  three columns for single-width labels and four for double-width labels.
  The main list uses the remaining terminal width.
  
  - Keep Help and Marked stacked when both exist, preserving their contents.
  - Return focus to the main list on collapse and skip minimized panes with
  Tab. Restoring with `]` keeps focus in the main list; `?` restores and
  focuses Help without resetting its scroll position.
  - Keep marks and quit protection intact. Do not dispatch deletion actions
  to a minimized Mark pane, and preserve literal brackets in search input.
  - Retain the minimized preference for the session, including while marking
  more entries or temporarily having no right-hand panes. With no panes,
  `]` is a no-op and the sidebar consumes no space.
  - Advertise `toggle-collapse = ]` on the bottom border of each unfocused,
  expanded Help or Mark pane, using the configured key and translated label.
  Hide the hint during search input, when disabled, or when it cannot fit
  between the border corners.
- <csr-id-38d30dbfbfc40eefdc684a3ef9ba0abefb25bd0b/> add directory_suffix option for directory markers
  <!-- agent -->
  Keep the ncdu-style leading slash by default and render a trailing slash when
  `directory_suffix = true` in the configuration.
- <csr-id-c1f5dda106964c04f4cdb132a26deafeb974727c/> localize the interactive interface
  <!-- agent -->
  Translate the remaining interactive panes, status messages, progress text,
  prompts, errors, and notifications for every supported locale. Detect the
  session language once and keep application, command, key, path, unit, Git-Glob,
  and mtime names unchanged, while using compact locale-specific action labels
  where they fit better.
- <csr-id-5b5e83f038ede02d8db71011a5168defb44aefac/> add German help translation
  Add de locale detection and a complete German translation for the interactive
  help pane, with documentation.
  
  Regression coverage verifies de_DE.UTF-8 and de selection plus English fallback for de_DE.ISO-8859-1.
- <csr-id-7b5e8e48d62dba73c2c66c6b5876f696c684926f/> add Simplified Chinese help translation
  Add zh locale detection and a complete Simplified Chinese translation for the interactive help pane, with documentation.
  
  Regression coverage verifies zh_CN.UTF-8 and zh selection plus English fallback for zh_CN.GB18030.
  
  Validated with: cargo test interactive::widgets::i18n::tests
- <csr-id-793ab720f63fc486e594b6fb4d1d60396328b67e/> add Korean help translation
  Add Korean locale detection and a complete Korean translation for the interactive help pane. Generalize the existing UTF-8 locale parser so supported language codes share one path, while preserving English fallback for non-UTF-8 locales.\n\nRegression: ko_KR.UTF-8 and ko now select Korean; ko_KR.EUC-KR falls back to English.\n\nValidated with: cargo test interactive::widgets::i18n::tests
- <csr-id-8a656e862b438ee39024254601ac3aaac0d63924/> repaint screen with Ctrl-L
  Interactive sessions had no way to recover when terminal contents were corrupted
  because unchanged frames produce no backend writes.
  
  Add a configurable Ctrl-L repaint action. It clears the backend directly
  to avoid Terminal::clear's cursor-position query racing the input thread,
  invalidates Ratatui's cached frame, and redraws through the normal event path.
- <csr-id-387636795f66a108926a0b7cd5921514e0bb7ee1/> configure flamegraph SVG rendering
  <!-- agent -->
  - Add palette, width, minimum-frame-width, title, and inverted-layout flags.
  - Label generated graphs in disk-usage terms with byte counts and paths.
  - Validate rendering arguments and exercise them in parser and end-to-end tests.
- <csr-id-6f797d6810793c5f57b3cee3132f96d1d32f9fce/> add `stacks` and `flamegraph` commands
  <!-- agent -->
  ## Summary
  
  - Promote folded output previously at `dua aggregate --stacks` to the first-class `dua stacks` command
  - Add `dua flamegraph`, backed by Inferno, with temporary opening by default and `--output` for file-only output.
  - Keep `aggregate --stack` as a hidden compatibility path and limit the new commands to relevant traversal flags.

### Bug Fixes

- <csr-id-73f27e8c313bbe4e029924771d8b3b62e8c4ba15/> prevent deletion during scans
  <!-- agent -->
  Deleting a directory while a scan was active could leave queued traversal
  events referring to removed nodes, aborting dua i with InvalidIndex.
  
  Reject delete and trash actions while a scan is active, retaining marked
  entries and showing "Traversal already running" so the user can retry
  afterward. Deletion already blocks the event loop, preventing new scans
  until it finishes.
- <csr-id-49792768817d3af6b93755430f52a80b0cfb8611/> keep Traditional Chinese locales in English
  Restrict the Simplified Chinese help table to bare zh, zh_CN, zh_SG, and zh_Hans while retaining English fallback for zh_TW, zh_HK, zh_MO, and zh_Hant.
  
  This addresses the post-commit review finding for 7b5e8e48.
  
  Validated with: cargo test interactive::widgets::i18n::tests

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
- <csr-id-de6c3c3bc7f2f397386912b4c135c50c8b05e5d3/> preallocate snapshot v2 imports
  <!-- agent -->
  Snapshot V2 prefixes the record stream with the total node count and native
  path-name bytes. Interactive imports use them for exact reservations while V1
  remains readable and declared requirements are validated.
  
  ## `~/dev` benchmark
  
  `dua i --import SNAPSHOT --once`, release builds; one frozen V1 traversal converted to V2 for identical contents.
  
  | Metric | V1 | V2 | Change |
  | --- | ---: | ---: | ---: |
  | Median peak RSS (5 runs) | 369,868,800 B | 230,473,728 B | -37.7% |
  | Mean wall time (20 runs) | 430.5 ± 1.8 ms | 438.1 ± 2.8 ms | +1.8% |
  | Compressed snapshot size | 38,215,358 B | 38,215,404 B | +46 B |

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

## 2.44.0 (2026-08-30)

This is a massive release that cost my weekend and was motivated by the chance to 
reduce the amount of open issues to 0.
Besides the memory usage being cut in half, one can now keep snapshots of interactive
runs, and compare them to older snapshots, or simply view them again with different
settings.
Finally, it's now possible to configure keyboarding as well.
What follows is the more elaborate, generated version of this text, and not my speech.

<!-- agent -->
This release turns traversal results into durable artifacts, shows exactly where
disk usage moved, puts every interactive key under user control, and makes large
traversals substantially leaner.

```console
dua interactive --export before.dua
# ... later ...
dua interactive --export after.dua
dua diff before.dua after.dua
```

**Snapshots.** Completed interactive traversals can now be exported atomically to
a deterministic, SHA-256-protected snapshot, with streaming zlib compression by
default. Load one with `dua interactive --import` for a read-only tour, or replay
it through `dua aggregate --import` as a flat list, a depth-limited tree, or
folded stacks, with the usual sorting and total controls. Snapshot imports retain
root order and traversal metadata; the aggregate renderers replay them with
bounded memory. The interactive view
still supports navigation, searching, sorting, marking, and opening stored paths
that continue to exist, while refresh, trash, and deletion stay safely disabled.
Its mark pane drops destructive prompts and colors for snapshots, or whenever
both destructive actions are unbound. The `dua` crate exposes the snapshot,
replay, and diff APIs for library users as well.

**Diffing.** `dua diff OLD NEW` streams additions, removals, and signed size
changes as a compact, colored context tree, then shows the largest additions and
removals. Use `--directories-only`, `--prefix`, `--depth`, `--format`, and
`--summary-limit` to shape the report. Both snapshots are checksum-verified in
parallel before their canonical entry streams are merged in bounded memory;
whole added or removed directories collapse into a single useful change.

**Configurable keybindings.** Every interactive action can now be rebound or
disabled, and `dua config show-default` documents all bindings. Pane hints follow
the configured keys, while `--once` understands the same named and modified
bindings as well as compact character sequences.

**Lower memory use.** Traversal state now uses a compact arena-backed tree,
shared filename storage, and dense directory IDs; snapshot replay borrows names
and reuses decoding buffers. In a local `~/dev` scan, `/usr/bin/time -lp` measured
maximum RSS at 268,206,080 bytes versus 524,812,288 bytes for 2.43.1—a 49%
reduction.

**More interactive polish.** Shift+U can extend a completed scan into its parent,
the mark pane shows the selected percentage of the root, and Ctrl+Z now suspends
and cleanly resumes the TUI on Unix.

### New Features

- <csr-id-2a9deb5b01e26977dcd788a0e42f77d1eaf035da/> accept configured keys in --once
  <!-- agent -->
  Keyboard shortcuts are configurable, but `--once` could only synthesize plain
  character events. That made named and modified bindings impossible to reproduce
  in one-shot debugging runs.
  
  - Reuse the configuration keybinding parser for `--once`.
  - Convert parsed bindings into terminal events.
  - Preserve compact character sequences such as `--once=jko`.
- <csr-id-931472011b13234db9f22381f9fb8fa473f0bd3a/> add streaming snapshot diff command
  <!-- agent -->
  ## Command-line interface
  
  - Add `dua diff OLD NEW` to compare an earlier traversal snapshot with a later
  one.
  - Support byte-format selection either globally or after the subcommand, with
  the configured format as the fallback.
  - Add `--directories-only` for aggregate directory changes, `--prefix PATH`
  for a component-aware subtree filter, `--depth DEPTH` for limiting the
  displayed tree, and `--summary-limit COUNT` for sizing or hiding the largest
  additions and removals lists.
  - Treat the selected root or prefix as the first displayed depth level while
  leaving summary calculations unbounded.
  - Reject traversal-only options and input paths because diffing never walks
  the live filesystem.
  - Hash both decompressed snapshots in parallel before comparison, then validate
  their records as they stream and verify each digest again at the end.
  
  ## Streaming comparison
  
  - Refactor snapshot decoding into a resumable decoder and entry iterator that
  can replay a verified, seekable snapshot without materializing its tree.
  - Rewind each snapshot for every pass and verify its digest again at the end
  of replay, detecting files changed after initial validation.
  - Expose native encoded names and duplicate-sibling ordinals from decoded
  entries to build deterministic comparison keys on Unix and Windows.
  - Enforce canonical sibling ordering during decoding and distinguish repeated
  sibling names without changing the snapshot format.
  - Merge the two canonical depth-first streams in lock-step on the main thread
  while a background collector retains only bounded summary entries.
  - Pair roots in stored order and deliberately avoid cross-root matching or
  rename detection.
  - Collapse an entirely added or removed directory to one change and skip its
  descendants, keeping large subtree changes concise.
  
  ## Diff output
  
  - Report additions, removals, and signed size deltas with `+`, `-`, and `~`
  markers in a compact tree with shared directory context.
  - Stream the tree first, then report a configurable number of the largest
  additions and removals (five by default) and the total number of changes.
  - Compare file sizes in the default mode and aggregate directory sizes in
  directory-only mode, including file-to-directory type changes.
  - Preserve full `u128` size deltas and use the selected human-readable byte
  format.
  - Mark depth-collapsed branches with an ellipsis without hiding their entries
  from the largest-change summary.
  - Render terminal additions in green, removals in red, modifications in
  yellow, and directory context in cyan while keeping redirected output free
  of escape sequences.
  - Use native path separators, sanitize control characters that could split
  output lines, and emit no output for identical snapshots or unmatched
  prefixes.
- <csr-id-3a30cc5319b9f472fe2d343a7b1d990e3e8d9f09/> add traversal snapshot codec
  <!-- agent -->
  ## Snapshot format
  
  - Add a deterministic version-1 `DUASNAP\0` codec for depth-first traversal
  forests while preserving input-root order and native Unix and Windows path
  representations.
  - Encode parent distances, node flags, names, `u128` sizes, nanosecond
  modification times, and optional entry counts using canonical ULEB128 fields.
  - Validate path components, tree structure, record limits, integer encodings,
  timestamps, node counts, checksums, truncation, and trailing data.
  - Protect decoded snapshot data with SHA-256 and preserve deterministic child
  ordering.
  - Support raw snapshots and streaming zlib compression through the existing
  `gix::zlib` dependency. Export at level 2 by default, accept levels 1–9, and
  use level 0 for raw output.
  - Verify seekable snapshots once and rewind them for bounded-memory replay
  without materializing the full tree.
  
  ## CLI integration
  
  - Add `dua interactive --export` to atomically persist a completed traversal
  through a neighboring temporary file before replacing the destination.
  - Add `dua interactive --import` to load a snapshot into the terminal UI
  without traversing the filesystem.
  - Add `dua aggregate --import` with flat, tree, depth-limited, sorted, total,
  and folded-stack output modes.
  - Reject traversal-only paths and options when importing and print snapshot
  provenance to standard error.
  - Preserve stored root ordering, sizes, entry counts, modification times, and
  metadata IO errors.
  
  ## Replay rendering
  
  - Add completed-traversal and replay renderers for aggregate, tree, and
  folded-stack output.
  - Keep only roots, displayed levels, or the current stack path in memory as
  required by each output mode.
  - Use checked arithmetic for aggregate and exclusive-size calculations and
  report malformed snapshot totals instead of silently saturating.
  - Handle the full `u128` snapshot size range and sanitize control characters
  when writing terminal output.
  
  ## Interactive safety
  
  - Treat imported snapshots as read-only while retaining navigation, sorting,
  searching, marking, and display controls.
  - Disable entry existence checks, filesystem refreshes, upward scans,
  gitignore discovery, permanent deletion, and trash operations.
  - Show the actual snapshot load time until the first user event and replace
  destructive mark prompts and styling with an explicit read-only safety notice.
  - Allow opening a stored path when it still exists, without weakening the final
  read-only guards around trash and permanent deletion.
  - Expose completed traversal roots in original input order for deterministic
  export.
- <csr-id-707b3166b2b9f6429c0d7ce222e1ee66f3a1940d/> make interactive keybindings configurable
  <!-- agent -->
  Interactive shortcuts were hard-coded, preventing users from avoiding terminal
  conflicts or adapting the controls to their habits.
  
  Add per-action [keys] arrays whose defaults preserve the existing controls,
  route every interactive pane and visible shortcut hint through those settings,
  and expose all bindings as commented config show-default templates. Parse common
  named keys and modifiers while handling shifted character events consistently
  across terminal backends.
  
  Regression coverage verifies that overrides replace only their own defaults,
  the documented template round-trips to the built-in defaults, modified shortcuts
  remain distinct, and shifted punctuation works on Windows.
- <csr-id-2fef192f1420a5fdd1acf565703a408ef9ce8a6a/> traverse beyond the interactive root
  <!-- agent -->
  Interactive mode stopped at its initial root and required a restart to inspect
  its parent.
  
  Handle Shift+U by scanning the parent while pruning already represented paths,
  then reattach the existing subtree at its natural position so node identities
  and prior work are preserved. The top-level message now advertises the shortcut
  when expansion is possible.
- <csr-id-5c6f4d3cc4a0d396e086871d936a7067bf6b3e3d/> show marked percentage of total size
  <!-- agent -->
  Show the marked byte total as a percentage of the top-level root total in the
  mark pane title.
- <csr-id-49f78a8d7d53f0c553fa04a63d858d533ea844ee/> show gross changes for collapsed diff branches
  Not reviewed at all, only functional tests.
  
  <!-- agent -->
  Depth-limited snapshot diffs now reveal the churn hidden beneath each collapsed directory.
  
  - Accumulate gross additions and removals while preserving bounded streaming memory.
  - Keep directory-only output and largest-change summaries unchanged.
  - Cover mixed changes, overflow, root boundaries, context ordering, and terminal colors.

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

### Bug Fixes

- <csr-id-1fbf73caf93f7e04b7217d8db87dd2b4ac8d478b/> support Ctrl+Z in interactive mode
  <!-- agent -->
  Interactive mode consumes Ctrl+Z in raw mode, so the shell never receives the
  usual job-control signal and the TUI cannot be resumed cleanly.
  
  Recognize the key globally on Unix, release crossterm's focus, cursor, raw-mode,
  and alternate-screen state, raise SIGTSTP, then reclaim and clear the Ratatui
  terminal after fg resumes the process. A focused regression test covers the
  Ctrl+Z mapping.
  
  Validated with the repository check, unit, journey, format, and clippy targets;
  an Expect-driven PTY run also verified stop, fg redraw, and clean shell
  restoration.

## 2.43.1 (2026-08-29)

### Bug Fixes

- <csr-id-1f2575a9d176c03701034a88e1e2737fc2c90c1f/> stream folded stacks during traversal
  <!-- agent -->
  Unlimited-depth aggregate --stack retained every traversal node and emitted only
  after walking, allowing memory to grow with entry count.
  
  Write each exclusive-size line as its parent-first event is integrated and
  retain only root accounting. Keep depth-limited mode unchanged because it must
  roll hidden descendants into visible frames.

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

## 2.42.1 (2026-08-15)

Starting directories with a large amount of files, like 50k, now see a 5x speedup on macOS and Windows as bulk-reading is also done there. Note also that this is still a small absolute difference, 100ms vs 500ms, but a good demonstration of how much large trees with a lot of such directories will benefit  by this, as these small absolute improvements accumulate.

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

## 2.41.1 (2026-08-04)

A one-line change that makes all the difference: Windows TUI performance isn't sluggish anymore as the drawing is now buffered!

### Bug Fixes

- <csr-id-bccddb7e78965e8bd06c83de6094bdd92ff5e81a/> buffer terminal frame output

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

## 2.38.1 (2026-07-20)

This release fixes a long-standing bug where `NO_COLOR=1` would make all styling disapear, including
the selection indicator itself. Now it's usable, finally.

### Bug Fixes

- <csr-id-ad19f3fe13322d548f645c16f30f3ac4553f812b/> only strip colors when `NO_COLOR` is enabled

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

## 2.37.1 (2026-06-29)

### Bug Fixes

- <csr-id-a83583c40b3cea909823510a0ccf6c92255c0e02/> degrade entries title on narrow terminals
  improve the interactive top-bar so narrow terminal
  sizes degrade gracefully. Statistics should disappear when the current path
  needs the space, and the path should compact by removing the fewest consecutive
  middle components needed to fit.

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

## 2.33.0 (2026-01-05)

### New Features

<csr-id-2f720cf5610c215dc4fbb7cd270fe055fd403b42/>

- <csr-id-85c7c7218cbb70b0626f430afd03ed819387e9e2/> Add environment variable support for all global arguments
  - `DUA_THREADS` → `--threads`

## 2.32.2 (2025-10-28)

### Bug Fixes

- <csr-id-847af46ba643c53b8d5aa7a9a3abd9ff37032311/> don't let 'q' quit instantly if it's still collecting files.

## 2.32.1 (2025-10-28)

'q' to quit is now more usable as it will insta-quit if the traversal took less than 10s and
if nothing is still marked for deletion.

This makes it easy to use in 'quick-view' scenarios.

### Bug Fixes

- <csr-id-d769de92b7abc842dab45141750e809b1141ba26/> instantly quit when no items marked for deletion

## 2.32.0 (2025-09-15)

### New Features

- <csr-id-bbe368f3c33cf58625e0f2a24f198ee8a6f836a6/> `Ctrl+f` in the glob prompt now toggles the mode from case-insensitive to sensitive.

## 2.31.0 (2025-08-02)

This release prominently adds a prompt that shows before quitting the app. When you pressed esc or q, the status bar will show the prompt first. To really quit, you need to press esc or q again. You can also cancel the quit operation by pressing any key else. Meanwhile, ctrl-c still quits the app directly since it's a combination key.

That way, `dua` will not cause users to accidentally quit the app when they only want to dismiss some other panels. It's especially frustrating if the scan took a long time

### New Features

- <csr-id-f3c9bf65b97ac029d444e32fe23f5976b0c480b2/> prompt before quitting
  This release prominently adds a prompt that shows before quitting the app. When you pressed esc or q, the status bar will show the prompt first. To really quit, you need to press esc or q again. You can also cancel the quit operation by pressing any key else. Meanwhile, ctrl-c still quits the app directly since it's a combination key.
  
  That way, `dua` will not cause users to accidentally quit the app when they only want to dismiss some other panels. It's especially frustrating if the scan took a long time.

## 2.30.1 (2025-05-10)

In this release, the size of directories is also taken into consideration, for more realistic sizes similar to what `du` does.

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

## 2.28.0 (2024-01-23)

### New Features

- <csr-id-78b9a8e22568c902132ed98d32e223ff71eb7b06/> add `dua i --no-entry-check` flag.
  With it, in interactive mode, entries will not be checked for presence.
  
  This can avoid laggy behaviour when switching between directories
  as `lstat` calls will not run, which can be slow on some filesystems.

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

## 2.27.1 (2024-01-21)

### Bug Fixes

- <csr-id-f70d1a8e6ace812a7949cd7d0299507b71306d48/> Explicit refreshes with 'r and 'R' now work with multiple root paths as will.
  This can happen in cases of `dua i root-a root-b` for instance.

## 2.27.0 (2024-01-17)

### New Features

- <csr-id-bed351ed2190e50e2932278b9b13b83c2969401b/> Press `r` or `R` for refresh
  Lower-case `r` will refresh the currently selected entry, while upper-case `R`
  will refresh the entire displayed directory, and all entries in it.
  
  Further, what was called `item` is now called `entry` across the
  user-interface.
- <csr-id-1544e8dffeacb55940deae2d06534d8a500765d4/> show and hide mtime and item count columns with 'M' and 'C' respectively

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

## 2.24.2 (2023-12-26)

### Bug Fixes

- <csr-id-b5b8aa26b648d8a034667bca8320ba7952a27780/> avoid duplicate key input on windows.
  On Windows, key-states like press/release/repeat are made available
  separately, which means we should avoid responding to key-releases
  as it would incorrectly double the actual user inputs.

## 2.24.1 (2023-12-25)

### Bug Fixes

- <csr-id-8ae727e462b38541636c8e03d140953cad8f34cf/> keep checking for existance of entries outside of the glob top-level.
  The glob top-level is used to display all search results which means
  that there can be a lot of them, which would unnecessarily slow down
  the search operation.
  
  Previously it would never check for the existence of an entry in glob mode,
  but now it will do so outside of the top-level.

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

## 2.22.0 (2023-12-09)

### New Features

- <csr-id-45ccb7cb5a4765190ea6b8d02e0b29f63b1bd702/> Press `c` to sort by count of entries in a directory.
  That way it's easy to spot places that have a lot of (possibly small) files,
  which otherwise would remain under the radar when sorting by size.

## 2.21.0 (2023-12-06)

### New Features

- <csr-id-de4c2b3bd368fd032319b606b84fa488299bc9e1/> With a single path provided as root, pretend it's the current working dir
  This makes it seem like the user started the directory walk directly in the given directory,
  which is more intuitive than the previous approach only showed the given directory as
  top-level directory.
  
  Note that this change only affects invocations like `dua <dir>` or `dua i <dir>`.
- <csr-id-dd523e389bcc940a5d3e72099bb0c76f40371164/> press `m` to sort by modification date, ascending and descending.

## 2.20.3 (2023-11-21)

### Bug Fixes

- <csr-id-7ab0070dcfda573cfbdc8451ddba5fcf15067132/> mark-pane now doesn't double-count sizes anymore.

## 2.20.2 (2023-11-20)

### Bug Fixes

- <csr-id-49c3e3d02ad0c14c4123fe1a7fea1f2a5e7a990f/> alignment when in interactive mode and -f binary

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

## 2.19.2 (2023-02-23)

<csr-id-fe956ca6f244613762bb48de79eac1f6fa399e1b/>

### Bug Fixes

- <csr-id-31dacad6f723f379a2d12417d65177faccd67b76/> `-x` is applied to traversal as well.
  Previously `dua` would cross filesystems for traversal and simply not
  yield them, which somewhat defeated the purpose.
  
  Now it will avoid traversing into filesystem entries that are on a different
  filesystem, which should improve its performance visibly whenever multiple
  filesystems are involved.

## 2.19.1 (2023-02-05)

### Bug Fixes

- <csr-id-fb5a39ffb67fad80be0d2090efd34d259d439e98/> redraw window while gathering metadata in interactive mode.
  This fixes a by now long-standing issue with interactive mode only updating
  when keys are pressed, but not automatically.

## 2.19.0 (2022-12-13)

### New Features

- <csr-id-f073375938f742db3259ec284c3c0d4a56fd0077/> Remove the handbrake on MacOS which can now deliver the expected performance.
  Previously it would limit itself to only using 4 threads as it would
  use a lot of time in user space. This has changed now, and the traversal
  itself is much more efficient (even though it could definitely be more
  efficient when comparing to `pdu`).
  
  In any case, counting performance should now greatly improve on M1
  MacOS machines.

## 2.18.2 (2022-12-13)

## 2.18.1 (2022-12-13)

<csr-id-946806e7390799807361562b038fb12eeb2ddf11/>
<csr-id-d3fa946029ef44e5032762ff265180c23a629316/>

Update all dependencies to the latest version. This most notably changes the look of the CLI
to something without color by default thanks to the upgrade to `clap` 4.

## 2.18.0 (2022-09-12)

<csr-id-6a636d542594a76ef8b2faf2ec6347e4c8cb6b38/>

### Fixes

- Remove a duplicate draw call which would have doubled the time it takes to refresh on user input.
  This might have been noticable when large amounts of files are displayed.

### New Features

- <csr-id-28f5ac90cc1ba7d668ae8a83eb5cd899294a8301/> Automatically resize if the terminal changes in size.

## 2.17.8 (2022-07-05)

## 2.17.7 (2022-06-14)

### Fixes

- Improve readability of the currently visible path in light terminal color themes [(#129)](https://github.com/Byron/dua-cli/pull/129).

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

## 2.17.0 (2022-01-21)

### New Features

- <csr-id-e2686952b4daf4c35303689c36bebc3dfe3faf29/> interactive mode learns 'toggle [a]ll' and 'remove [a]ll'.
  In the mark pane, the 'a' key will now toggle all entries.
  This is particularly interesting for selecting entries to
  exclude by hande and then invert the selection by toggling [a]ll.
  
  In the mark pane, toggling all with the 'a' key means removing
  all entries and closing the pane.

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

## 2.14.10 (2021-10-26)

## 2.14.9 (2021-10-26)

## 2.14.8 (2021-10-26)

### Changed

- <csr-id-49193f0506946981bc056b29c3f09c94e30ac457/> auto-config support for Apple M1 Pro and Apple M1 Max

## v2.14.7 (2021-09-18)

- Fix deletion which broke with Rust 1.55, for those who are compiling the tool themselves.

## v2.14.6 (2021-08-22)

- Support for arrow keys as well as Home & End. The help pane was updated to reflect these changes.
- More readable information on how to delete or trash files in the mark pane.

## v2.14.5 (2021-08-16)

- Fix installation via `cargo install dua-cli`. Please note that it might break again as it still depends on the unsable `clap-3 beta 4`. Even when pinning it breakage is possible as its dependencies itself aren't pinned.

## v2.14.4 (2021-08-05)

- upgrade depencies
- upgrade to tui 0.16

## v2.14.3 (2021-07-25)

- upgrade `open` crate to v2

## v2.14.2 (2021-07-14)

- `Ctrl-T` to trash (instead of removal) is now an optional default feature, allowing it to be
  disabled on FreeBSD which isn't currently supported.
- Update dependencies

## v2.14.1 (2021-06-30)

- Pressing `ctrl+t` in the mark pane now trashes entries instead of deleting them. Not only does that make
  'deletion' reversible but it makes removal of the entry faster in many cases as well.
- updated dependencies

## v2.14.0 (2021-06-30)

<csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/>

## v2.13.1 (2021-06-09)

<csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/>

- Allow usage of the feature introduced in v2.13 by writing the TUI to stderr instead of stdout.
  That way the output can be redirected.

## v2.13.0 (2021-06-08)

- Print remaining marked paths upon exit on stdout. This may help to use `dua i` with other programs
  who want to process the marked paths on their own.

## v2.12.2 (2021-06-07)

- Prepare for release of new Apple hardware and be more specific when auto-configuring the correct amount of threads.
  Instead an error message will be printed to inform that the given CPU brand isn't configurable yet.

## v2.12.1 (2021-05-30)

- Fixed bug that would cause `dua` to unconditionally sleep for 1 second. This sleep was intended for a spawned thread,
  but it slipped into the main thread.

## v2.12.0 (2021-05-29)

YANKED.

- Add minimal progress for when `dua` invocations take longer than 1 second

## v2.11.3 (2021-05-09)

- re-add arm builds
- dependency updates (including tui 0.15)

## v2.11.2 (2021-05-03)

- dependency updates (including tui 0.15)

## v2.11.1 (2021-02-22)

<csr-id-59315b7c63b7328fa70bfe5fc43fdbe9dc5f92e7/>

- The `-x/--stay-on-filesystem` flag is now respected for multiple root paths, as in `dua -x
path-FS1/ path-FS2/`, as such `dua` will stay in FS1 if the CWD is in FS1.

## v2.11.0 (2021-02-15)

### Features

- Add binding capital 'H' to go to the top of any pane/list
- Add binding capital 'G' to go to the bottom of any pane/list

### Fixes

- Without user input during `dua i [<multiple paths>]` the top-most entry will remain selected.
- Avoid stale frame at the end of traversal in interactive sessions when there is no user input.

## v2.10.10 (2021-01-23)

<csr-id-9384cdb5b95e5260f46ccd23e7ca276304190a34/>

Fix --version flag.
It looks like the latest BETAs of clap removed setting the version implicitly.

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

## v0.14.0 (2021-01-04)

## v2.10.7 (2020-12-16)

Better performance on Apple Silicon (M1).

The IO subsystem on Apple Silicon is different and won't scale nicely just by using all amount of available cores. Instead it seems best to only
use as many threads as performance cores are present on the system - otherwise the performance might actually get worse while using more power.

On all other systems, the default number of threads did not change.

**Please note that for optimial performance** one would need an arm build on MacOS, currently provided is only intel builds.

## v2.10.5 (2020-11-15)

Dependency update.

- upgrade to TUI v0.13.0

## v2.10.4 (2020-11-15)

## v0.13.0 (2020-11-15)

## v0.0.1 (2020-10-26)

## v2.10.3 (2020-10-15)

Dependency update.
Should fix [this issue](https://github.com/Byron/dua-cli/issues/66)

## v0.12.0 (2020-09-28)

## v2.10.2 (2020-07-27)

Change light-grey color in command-line mode to Cyan to fix disappearing text.

## v2.10.1 (2020-07-24)

Change light-grey color in interactive mode to Cyan to fix disappearing text.

See [this PR](https://github.com/Byron/dua-cli/pull/62) for reference.

## v2.10.0 (2020-07-22)

Minor improvements of looks; improved windows support.

- previously in interactive mode on Windows, directory sizes would appear as 0 bytes in size. This is now fixed!

## v0.10.1 (2020-07-22)

## v0.10.0 (2020-07-22)

## v0.4.1 (2020-07-10)

## v2.9.1 (2020-07-07)

Globs for Windows; fixed handling of colors.

- On widnows, `dua` will now expand glob patterns by itself as this capability is not implemented by shells `dua` can now run in.
- A bug was discovered that could cause `dua a` invocation to now show paths behind their size in an incorrect attempt to not print with color.

## v2.9.0 (2020-07-06)

Full windows support!

- On Windows, we will now build using `crossterm`, which was greatly facilitated by `crosstermion`.
- On Unix systems, the backend is still `termion`.

## v2.8.2 (2020-07-02)

- Switch back to `clap` from `argh` to support non-UTF-8 encoded paths to be passed to dua

I hope that `argh` or an alternative will one day consider supporting os-strings, as it would in theory be an issue
for anyone who passes paths to their command-line tool.

## v2.8.1 (2020-07-02)

- Switch from deprecated `failure` to `anyhow` to reduce compile times a little and binary size by 130kb.

## v2.8.0 (2020-07-02)

- Switched from `clap` to `argh` for a 300kb reduction in binary size and 1 minute smaller compile times.

## v2.7.0 (2020-07-02)

- [Support for extremely large][issue-58], zeta byte scale, files or filesystem traversals.
- [Fix possibly incorrect handling of hard links][pr-57] in traversals spanning multiple devices.

Both changes were enabled by [@Freaky](https://github.com/Freaky) whom I hereby thank wholeheartedly :).

## v2.6.1 (2020-05-31)

- quit without delay from interactive mode after `dua` was opened on huge directories trees.
  See [this commit](https://github.com/Byron/dua-cli/commit/91aade36c71e4e14167030b6ec8c3c13dcdc1b2b) for details.

## v2.6.0 (2020-05-04)

- Use `x` to only mark entries for deletion, instead of toggling them.
- Add `-x` | `--stay-on-filesystem` flag to force staying on the file system the root is on, similar to `-x` in the venerable `du` tool.

## v2.5.0 (2020-04-05)

Much more nuanced percentage bars for a more precise visualization of space consumption.

## v0.3.0 (2020-04-03)

## v2.4.1 (2020-03-30)

Bugfix: Update currently visible entries when scanning.

## v2.4.0 (2020-03-29)

Full interaction during scanning phase; add inline-help for better UX.

## v0.2.2 (2020-03-29)

## v2.3.9 (2020-03-27)

Do not follow symlinks unless it's the only root path to follow.

This brutally fixes an issue where symbolics links are honored when they are placed in the current working directory, as internally `dua` will
treat each cwd directory entry as individual root path.

## v2.3.8 (2020-03-26)

`dua interactive` (`dua i`) is now about twice as fast due to using all logical cores, not just physical ones.
This is also the first release with github releases: https://github.com/Byron/dua-cli/releases/tag/v2.3.8

## v2.3.7 (2020-03-24)

<csr-id-45d1ef31181cd9b430d855a4fe23550ea97e685e/>

Upgrade to filesize 0.2.0 from 0.1.0; update dependency versions

## v2.3.6 (2020-03-16)

Upgrade to jwalk 0.5 bringing better threading control and no symlink following during traversal

## v2.3.5 (2020-03-15)

Fast exit from interactive mode for a responsive exit; dependency updates (except jwalk)

## v2.3.4 (2020-03-15)

YANKED - jwalk 0.5.0 wasn't used correctly which led to a performance regression

## v2.3.3 (2020-03-14)

YANKED - journey tests failed to changed method signature.

## v2.3.2 (2020-02-25)

Incude the license file in crate.

## v2.3.1 (2020-02-23)

Include .md files in Crate, update dependencies.

## v2.3.0 (2020-02-22)

Show size on disk by default; Dependency Update.

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/37), hard links are now not counted anymore.
The `-l` flag will count hard links as it did before.

And of course, this has no noticable performance impact.

## v2.2.0 (2020-02-22)

Show size on disk by default; Dependency Update.

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/35), the old apparent size can be displayed with the
`-A` flag, and the much more useful 'size on disk' is now shown by default.

To my pleasant surprise, this does not seem to affect performance at all - everything stays speedy.

## v2.1.13 (2020-02-01)

Dependency Update; Github Releases.
Binaries for Linux and MacOS are now available on GitHub Releases.

## v2.1.12 (2019-10-23)

More obvious highlighting of active panel.

Depending on the terminal used, it might not have been obvious which panel was active. This might be
confusing to new and current users.
Now the color of the widget frame is changed to light gray, instead of remaining gray.

## v2.1.11 (2019-07-26)

Finally fix symlink handling.

`dua` will not follow symbolic links when deleting directories. Thank a ton, @vks!

_Technical Notes_: Handling symbolic links properly is impossible without usage of `symlink_metadata()`.

## v2.1.10 (2019-07-25)

Compatibility with light terminals.

- the TUI is now usable on light terminals, and highlighting is more consistent. Thank you, @vks!
- Fixes misaligned columns when displaying '100.00%' alongside other rows by displaying `100.0%` instead. Thanks, @vks, for pointing it out.

## v2.1.9 (2019-07-21)

Improved handling of broken symlinks.

- during symlink deletion, now broken symlinks will be deleted as expected.
- always return to the previous terminal screen so the TUI doesn't stick to the current one.
- display broken symlinks on the first level of iteration.

## v2.1.8 (2019-07-14)

Don't follow symbolic links when deleting directories.

[A critical bug was discovered](https://github.com/Byron/dua-cli/issues/24) which would lead to deletion
of unwanted `directories` as `dua` would follow symbolic links during traversal during deletion.

Please note that symbolic links to files would be treated correctly, only removing the symbolic link.

This is now fixed.

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

## v2.1.1 (2019-06-16)

Bug fixes and improvements.

- Better information about deletion progress
- removal of windows support

## v2.1.0 (2019-06-16)

Bug fixes and improvements.

- windows support (never actually worked), usage of crossterm is difficult thanks to completely
  different input handling.
- additional key-bindings
- auto-restore previous selection in each visited directory

## v2.0.1 (2019-06-16)

Bug fixes and improvements.

- fix typo in title
- better display of IO-Errors in aggregate mode

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

## v1.1.0 (2019-06-01)

## 1.0.0 (2019-06-01)

Simple CLI to list top-level directories similar to sn-sort, but faster and more tailored to getting an idea of where most space is used.

## v2.10.6

Fix `dua -h` usage string.

## 1.2.0

The first usable, read-only interactive terminal user interface.
That's that. We also use `tui-react`, something that makes it much more pleasant to handle the
application and GUI state.
