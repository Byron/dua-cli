#### v2.8.2

* Switch back to `structopt` from `argh` to support non-UTF-8 encoded paths to be passed to dua

I hope that `argh` or an alternative will one day consider supporting os-strings, as it would in theory be an issue
for anyone who passes paths to their command-line tool.

#### v2.8.1

* Switch from deprecated `failure` to `anyhow` to reduce compile times a little and binary size by 130kb.

#### v2.8.0

* Switched from `structopt` to `argh` for a 300kb reduction in binary size and 1 minute smaller compile times.

#### v2.7.0

* [Support for extremely large][issue-58], zeta byte scale, files or filesystem traversals.
* [Fix possibly incorrect handling of hard links][pr-57] in traversals spanning multiple devices.

Both changes were enabled by a single contributor, and I am very thankful towards them.

[issue-58]: https://github.com/Byron/dua-cli/issues/58
[pr-57]: https://github.com/Byron/dua-cli/pull/57

#### v2.6.1

* quit without delay from interactive mode after `dua` was opened on huge directories trees. 
  See [this commit](https://github.com/Byron/dua-cli/commit/91aade36c71e4e14167030b6ec8c3c13dcdc1b2b) for details.

#### v2.6.0

* Use `x` to only mark entries for deletion, instead of toggling them.
* Add `-x` | `--stay-on-filesystem` flag to force staying on the file system the root is on, similar to `-x` in the venerable `du` tool.

#### v2.5.0 Much more nuanced percentage bars for a more precise visualization of space consumption

#### v2.4.1 Bugfix: Update currently visible entries when scanning

#### v2.4.0 Full interaction during scanning phase; add inline-help for better UX

#### v2.3.9 Do not follow symlinks unless it's the only root path to follow

This brutally fixes an issue where symbolics links are honored when they are placed in the current working directory, as internally `dua` will 
treat each cwd directory entry as individual root path.

#### v2.3.8 `dua interactive` (`dua i`) is now about twice as fast due to using all logical cores, not just physical ones

This is also the first release with github releases: https://github.com/Byron/dua-cli/releases/tag/v2.3.8

#### v2.3.7 Upgrade to filesize 0.2.0 from 0.1.0; update dependency versions

#### v2.3.6 Upgrade to jwalk 0.5 bringing better threading control and no symlink following during traversal

#### v2.3.5 Fast exit from interactive mode for a responsive exit; dependency updates (except jwalk)

#### v2.3.4 YANKED - jwalk 0.5.0 wasn't used correctly which led to a performance regression

#### v2.3.3 YANKED - journey tests failed to changed method signature

#### v2.3.2 Incude the license file in crate

#### v2.3.1 Include .md files in Crate, update dependencies

#### v2.3 Show size on disk by default; Dependency Update

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/37), hard links are now not counted anymore.
The `-l` flag will count hard links as it did before. 

And of course, this has no noticable performance impact.

#### v2.2 Show size on disk by default; Dependency Update

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/35), the old apparent size can be displayed with the
`-A` flag, and the much more useful 'size on disk' is now shown by default.

To my pleasant surprise, this does not seem to affect performance at all - everything stays speedy.

#### v2.1.13-- Dependency Update; Github Releases

Binaries for Linux and MacOS are now available on GitHub Releases.

#### v2.1.12-- More obvious highlighting of active panel

Depending on the terminal used, it might not have been obvious which panel was active. This might be
confusing to new and current users.
Now the color of the widget frame is changed to light gray, instead of remaining gray.

#### v2.1.11 - Finally fix symlink handling

`dua` will not follow symbolic links when deleting directories. Thank a ton, @vks!

_Technical Notes_: Handling symbolic links properly is impossible without usage of `symlink_metadata()`.

#### v2.1.10 - compatibility with light terminals

* the TUI is now usable on light terminals, and highlighting is more consistent. Thank you, @vks!
* Fixes misaligned columns when displaying '100.00%' alongside other rows by displaying `100.0%` instead. Thanks, @vks, for pointing it out.

#### v2.1.9 - improved handling of broken symlinks

* during symlink deletion, now broken symlinks will be deleted as expected.
* always return to the previous terminal screen so the TUI doesn't stick to the current one.
* display broken symlinks on the first level of iteration.

#### v2.1.8 - don't follow symbolic links when deleting directories

[A critical bug was discovered](https://github.com/Byron/dua-cli/issues/24) which would lead to deletion
of unwanted `directories` as `dua` would follow symbolic links during traversal during deletion.

Please note that symbolic links to files would be treated correctly, only removing the symbolic link.

This is now fixed.
 
#### v2.1.7 - use latest version of open-rs

That way, pressing `shift + O` to open the currently selected file won't possibly spam the terminal
with messages caused by the program used to find the system program to open the file.

Fixes [#14](https://github.com/Byron/dua-cli/issues/14)

#### v2.1.5 - re-release with Cargo.lock

#### v2.1.2 bug fixes and improvements

* Performance fix when showing folders with large amounts of files
* Display of amount of entries per directory

#### v2.1.1 bug fixes and improvements

* Better information about deletion progress
* removal of windows support

#### v2.1.0- bug fixes and improvements

* windows support (never actually worked), usage of crossterm is difficult thanks to completely
  different input handling.
* additional key-bindings
* auto-restore previous selection in each visited directory

#### v2.0.1- bug fixes and improvements

* fix typo in title 
* better display of IO-Errors in aggregate mode

#### v2.0.0 - interactive visualization of directory sizes with an option to queue their deletion

A sub-command bringing up a terminal user interface to allow drilling into directories, and clearing them out, all using the keyboard exclusively.

##### Other Features

 * [x] Single Unit Mode, see [reddit](https://www.reddit.com/r/rust/comments/bvjtan/introducing_dua_a_parallel_du_for_humans/epsroxg/)

#### 1.2 (_released_) - - the first usable, read-only interactive terminal user interface

That's that. We also use `tui-react`, something that makes it much more pleasant to handle the
application and GUI state.

#### 1.0 (_released_) - aggregate directories, fast

Simple CLI to list top-level directories similar to sn-sort, but faster and more tailored to getting an idea of where most space is used.
