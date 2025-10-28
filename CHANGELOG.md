# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

 - 7 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.32.1 ([`d55602a`](https://github.com/Byron/dua-cli/commit/d55602a515e9c888243fedbc2f07f54bb2d2cc1c))
    - Prepare next release changelog ([`fb8571f`](https://github.com/Byron/dua-cli/commit/fb8571fec263a27a03700dacdf1d1f51e0e844e7))
    - Merge pull request #297 from Byron/copilot/fix-instant-quit-on-q ([`938ae33`](https://github.com/Byron/dua-cli/commit/938ae33195498ab3451d83dac90eeb80302b0d12))
    - Only quit immediately if the traversal didn't take too long. ([`7f27170`](https://github.com/Byron/dua-cli/commit/7f271701240d89799b3dd6a8f95cc613dd5c1340))
    - Update dependencies ([`d0f66d6`](https://github.com/Byron/dua-cli/commit/d0f66d6dcb859c4a27d30dc74da60eda1b5fb758))
    - Refactor ([`b710cb1`](https://github.com/Byron/dua-cli/commit/b710cb164615b6c68416ce8bb882e41ea12bd0df))
    - Instantly quit when no items marked for deletion ([`d769de9`](https://github.com/Byron/dua-cli/commit/d769de92b7abc842dab45141750e809b1141ba26))
</details>

## 2.32.0 (2025-09-15)

### New Features

 - <csr-id-bbe368f3c33cf58625e0f2a24f198ee8a6f836a6/> `Ctrl+f` in the glob prompt now toggles the mode from case-insensitive to sensitive.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 43 calendar days.
 - 43 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.32.0 ([`3f364c1`](https://github.com/Byron/dua-cli/commit/3f364c1c44a7d195eacda453d4da23c3c60fc632))
    - Merge pull request #293 from Byron/copilot/fix-2a5bb691-5ca0-4cf5-af1c-895f4fcb1f06 ([`91bc45d`](https://github.com/Byron/dua-cli/commit/91bc45da799e7bd41d75f71a67091c6537de7ef1))
    - `Ctrl+f` in the glob prompt now toggles the mode from case-insensitive to sensitive. ([`bbe368f`](https://github.com/Byron/dua-cli/commit/bbe368f3c33cf58625e0f2a24f198ee8a6f836a6))
    - Implement case-sensitive glob search with '^f' shortcut ([`32ab50f`](https://github.com/Byron/dua-cli/commit/32ab50f5b91cb9e4b4e4fa342d6a36adc944c14e))
    - See if the windows-2022 image will work for us. ([`7abf86a`](https://github.com/Byron/dua-cli/commit/7abf86a7c214452425bacb8290b0436119f795da))
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

 - 12 commits contributed to the release.
 - 83 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.31.0 ([`5daa8d5`](https://github.com/Byron/dua-cli/commit/5daa8d5661eef1269c6f71fe0a20da9f3f1fefaf))
    - Bump minor version and add changelog notes ([`d238dbb`](https://github.com/Byron/dua-cli/commit/d238dbb12b8491393a41ff5da425497f04c343d0))
    - Update dependencies ([`39ca5e6`](https://github.com/Byron/dua-cli/commit/39ca5e616d28765a79a619015c4f259b82f1ed5f))
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

 - 7 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.30.1 ([`ebf906a`](https://github.com/Byron/dua-cli/commit/ebf906af390c330aa5983ba0699127a6eb0e16d5))
    - Prepare changelog for next release. ([`4ed0acf`](https://github.com/Byron/dua-cli/commit/4ed0acfe47836a7dd0dd2e22998c7ea227555fe2))
    - Merge pull request #284 from joehasson/feat/include-directory-inodes-in-size-calculations ([`b5b411b`](https://github.com/Byron/dua-cli/commit/b5b411b2757d61ebdd64f2254cac002234ed1d5d))
    - Include directory inode in directory size aggregation ([`a93b28e`](https://github.com/Byron/dua-cli/commit/a93b28ead02714bb80cda296e4b4ad2a1248ba0e))
    - Merge pull request #283 from Byron/dependabot/cargo/crossbeam-channel-0.5.15 ([`ca6122f`](https://github.com/Byron/dua-cli/commit/ca6122f914d0626a5f51ff0986bcb6a1ff98e9ec))
    - Thanks clippy ([`49bbd2c`](https://github.com/Byron/dua-cli/commit/49bbd2c05d091ef344feb83e6a25d825267025e7))
    - Bump crossbeam-channel from 0.5.13 to 0.5.15 ([`af3f3fe`](https://github.com/Byron/dua-cli/commit/af3f3fed990be166595b0cb5133272d2091483e8))
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

### Style

 - <csr-id-c1dc1b26735279e976d36597bfe45eb3557458fe/> simplify some statements for readability

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 55 calendar days.
 - 84 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.30.0 ([`b5377bc`](https://github.com/Byron/dua-cli/commit/b5377bcf07ced300ab1493ed969999ac58825e83))
    - Merge pull request #275 from joehasson/feat/interactive-mode-sort-by-name ([`364f732`](https://github.com/Byron/dua-cli/commit/364f73206dc89277496486da6a6b462fb38e1262))
    - Thanks clippy ([`092a6c5`](https://github.com/Byron/dua-cli/commit/092a6c53cdd0b01f4041f7a79c736b27a1c2a3ce))
    - Allow sorting by name in interactive mode ([`73224e6`](https://github.com/Byron/dua-cli/commit/73224e63bc21d1ffa416b3e685a95c04afb72657))
    - Merge pull request #274 from musicinmybrain/petgraph0.7 ([`14a276e`](https://github.com/Byron/dua-cli/commit/14a276eb513cec833f5137670ff55f1391d1b4d7))
    - Update petgraph to 0.7 ([`b83a2f6`](https://github.com/Byron/dua-cli/commit/b83a2f6dc348f696d7f675e1266ecc2527382d8e))
    - Merge pull request #271 from hamirmahal/style/simplify-some-statements-for-readability ([`3bc25bd`](https://github.com/Byron/dua-cli/commit/3bc25bd5e337bdebce706a89e0fe4227d9997b9a))
    - Formatting in src/interactive/app/handlers.rs ([`0a4d09e`](https://github.com/Byron/dua-cli/commit/0a4d09eae898c80f8f81bbf8f8c652883d9424e7))
    - Simplify some statements for readability ([`c1dc1b2`](https://github.com/Byron/dua-cli/commit/c1dc1b26735279e976d36597bfe45eb3557458fe))
    - On MacOS use only 3 threads by default. ([`8933be4`](https://github.com/Byron/dua-cli/commit/8933be4fa8a801a7f79d994d735eee1105bd30ba))
</details>

## 2.29.4 (2024-11-03)

<csr-id-44d25a64475ff861875fe97c4612356eb697f4bf/>

### Bug Fixes

 - <csr-id-4efb5d0cd46c00f13d06dfcb13279f2c5d852d95/> enable "chrono" in `trash` crate
   This hopefully adds deletion dates to trashed files.

### Other

 - <csr-id-44d25a64475ff861875fe97c4612356eb697f4bf/> update actions/upload-artifact to v4
   The release workflow failed because actions/upload-artifact@v1 doesn't work anymore.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#268](https://github.com/Byron/dua-cli/issues/268)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#268](https://github.com/Byron/dua-cli/issues/268)**
    - Enable "chrono" in `trash` crate ([`4efb5d0`](https://github.com/Byron/dua-cli/commit/4efb5d0cd46c00f13d06dfcb13279f2c5d852d95))
 * **Uncategorized**
    - Release dua-cli v2.29.4 ([`6386036`](https://github.com/Byron/dua-cli/commit/63860365d18e156656b7f00e1c99be93186b04b2))
    - Merge pull request #267 from suzuki-shunsuke/chore-update-upload-artifact-v4 ([`904b3fc`](https://github.com/Byron/dua-cli/commit/904b3fce3a1df7a57c8564e57e65ed582d61ba44))
    - Update actions/upload-artifact to v4 ([`44d25a6`](https://github.com/Byron/dua-cli/commit/44d25a64475ff861875fe97c4612356eb697f4bf))
</details>

## 2.29.3 (2024-11-03)

<csr-id-25a6ad73a6571bffe7fac56c61ff2e52ccda0b53/>
<csr-id-c66e585ec73707d113d481ae2627187c9071539d/>
<csr-id-fa203b1b955b896d989eb46e72f13fd5e6cd6120/>

### Chore

 - <csr-id-25a6ad73a6571bffe7fac56c61ff2e52ccda0b53/> upgrade `gix-glob` and `trash`

### Other

 - <csr-id-c66e585ec73707d113d481ae2627187c9071539d/> add x-cmd install steps
   Hi, I've added instructions on how to install dua using x-cmd to the README file.
   - [x-cmd](https://www.x-cmd.com/) is a **toolbox for Posix Shell**, offering a lightweight package manager built using shell and awk.
   ```sh
   x env use dua
   ```
 - <csr-id-fa203b1b955b896d989eb46e72f13fd5e6cd6120/> Auto submit new manifest to winget-pkgs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#265](https://github.com/Byron/dua-cli/issues/265)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#265](https://github.com/Byron/dua-cli/issues/265)**
    - Upgrade `gix-glob` and `trash` ([`25a6ad7`](https://github.com/Byron/dua-cli/commit/25a6ad73a6571bffe7fac56c61ff2e52ccda0b53))
 * **Uncategorized**
    - Release dua-cli v2.29.3 ([`f5cb1ce`](https://github.com/Byron/dua-cli/commit/f5cb1ce4bab5c225bf8053229e86ab21b5f273e5))
    - Merge pull request #266 from Byron/upgrades ([`297be49`](https://github.com/Byron/dua-cli/commit/297be49c4502ca05104693a8135ccf1f63b8a5c7))
    - Merge pull request #261 from Byron/dependabot/github_actions/dot-github/workflows/actions/download-artifact-4.1.7 ([`ee55bf5`](https://github.com/Byron/dua-cli/commit/ee55bf584ffa3e7adf25f30190e091a54102959f))
    - Merge pull request #260 from Byron/dependabot/cargo/gix-path-0.10.10 ([`4d859aa`](https://github.com/Byron/dua-cli/commit/4d859aac4fd95b2a3c12dbbc5bfaa414490d0316))
    - Bump actions/download-artifact from 1 to 4.1.7 in /.github/workflows ([`57a67a5`](https://github.com/Byron/dua-cli/commit/57a67a59da48e042332fd7e8b6bd046c2155a391))
    - Bump gix-path from 0.10.9 to 0.10.10 ([`4db9b9e`](https://github.com/Byron/dua-cli/commit/4db9b9e1269a562a93e6a94a641eed2498da9621))
    - Merge pull request #259 from lunrenyi/patch-1 ([`b3fdc92`](https://github.com/Byron/dua-cli/commit/b3fdc923cfa5ef03a932aa4d7c994e322f212da3))
    - Add x-cmd install steps ([`c66e585`](https://github.com/Byron/dua-cli/commit/c66e585ec73707d113d481ae2627187c9071539d))
    - Merge pull request #256 from Dragon1573/issues/255 ([`3309e3b`](https://github.com/Byron/dua-cli/commit/3309e3b59f167002f883fb79e98e8326f2ba1055))
    - Auto submit new manifest to winget-pkgs ([`fa203b1`](https://github.com/Byron/dua-cli/commit/fa203b1b955b896d989eb46e72f13fd5e6cd6120))
</details>

## 2.29.2 (2024-08-10)

A maintenance release without user-facing changes.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#253](https://github.com/Byron/dua-cli/issues/253), [#254](https://github.com/Byron/dua-cli/issues/254)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#253](https://github.com/Byron/dua-cli/issues/253)**
    - Revert "avoid `Cargo.lock` in exported archives" ([`452b30f`](https://github.com/Byron/dua-cli/commit/452b30f0a7c2cc0b5c33cb7d12700fd18dd1e4e1))
 * **[#254](https://github.com/Byron/dua-cli/issues/254)**
    - Prepare a new patch release to bring Cargo.lock back ([`dd2d7cd`](https://github.com/Byron/dua-cli/commit/dd2d7cda58c34d0ddf7af11c8b06b31f4c4ddd31))
 * **Uncategorized**
    - Release dua-cli v2.29.2 ([`60ed763`](https://github.com/Byron/dua-cli/commit/60ed763e98620664c06eac5fd7972f407ae97503))
</details>

## 2.29.1 (2024-08-10)

<csr-id-f0b9a8e07b24d963116da4dfaa3338a4d4e8a3bf/>

This is a maintenance release without user-facing changes.

### Bug Fixes

 - <csr-id-46ebf149548f10c1b144f596aa715062787fd141/> clippy warning

### Other

 - <csr-id-f0b9a8e07b24d963116da4dfaa3338a4d4e8a3bf/> update version in install instructions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 126 calendar days.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#253](https://github.com/Byron/dua-cli/issues/253)

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#253](https://github.com/Byron/dua-cli/issues/253)**
    - Avoid `Cargo.lock` in exported archives ([`2ef583d`](https://github.com/Byron/dua-cli/commit/2ef583d0485941c9cbe91e85c2b62ead80cbb490))
 * **Uncategorized**
    - Release dua-cli v2.29.1 ([`c82b3f9`](https://github.com/Byron/dua-cli/commit/c82b3f9cad2e84b4976c142faa7f7085d7cc20b7))
    - Prepare changelog prior to release ([`9a27130`](https://github.com/Byron/dua-cli/commit/9a271309624bd01c1814e4058c859e3fdc2ffcd3))
    - Update locked dependencies ([`319bdde`](https://github.com/Byron/dua-cli/commit/319bddec1812b4a732c8380208b5503f40841bbc))
    - Merge branch 'olastor/main' ([`80c14a9`](https://github.com/Byron/dua-cli/commit/80c14a9cd28e5a18f5e9df517f6a3332d90e7c30))
    - Thanks clippy ([`7ddbfbe`](https://github.com/Byron/dua-cli/commit/7ddbfbe37a56b845cc437e60509cb5bb6a89fe01))
    - Update version in install instructions ([`f0b9a8e`](https://github.com/Byron/dua-cli/commit/f0b9a8e07b24d963116da4dfaa3338a4d4e8a3bf))
    - Merge pull request #246 from matta/use-ratatui-terminal ([`ced3b4f`](https://github.com/Byron/dua-cli/commit/ced3b4f5e375278dbee52319eac8750b14eb328a))
    - Update dependencies to fix CI ([`69a7cdd`](https://github.com/Byron/dua-cli/commit/69a7cdd11068b2e4053460b42d8a4959501a7799))
    - Replace tui_react::Terminal with tui::Terminal ([`1350c2f`](https://github.com/Byron/dua-cli/commit/1350c2f5d7e7bd79909fe78584008385dec1b794))
    - Merge pull request #247 from matta/fix-clippy ([`e3aff9d`](https://github.com/Byron/dua-cli/commit/e3aff9d987a09910b52dbce84c0de806d4233b04))
    - Merge pull request #240 from denilsonsa/patch-1 ([`c1caa70`](https://github.com/Byron/dua-cli/commit/c1caa704cab61bf94abbf4de64f6cb890d1a1ea6))
    - Clippy warning ([`46ebf14`](https://github.com/Byron/dua-cli/commit/46ebf149548f10c1b144f596aa715062787fd141))
    - Added link to Filelight ([`e3883af`](https://github.com/Byron/dua-cli/commit/e3883af3f751a683b0ba208efc9127e0a07281e2))
    - Merge pull request #235 from blinxen/main ([`66e0166`](https://github.com/Byron/dua-cli/commit/66e01667ddf6cc8da8a2387ef9d457aef63f42a5))
    - Update gix-glob dependency to 0.16.2 ([`71a365e`](https://github.com/Byron/dua-cli/commit/71a365efadab4c8dffb4a0e10dcc282dd3f57516))
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

 - 19 commits contributed to the release.
 - 46 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.29.0 ([`ab52733`](https://github.com/Byron/dua-cli/commit/ab5273334bd55e01adc5ded64432bb40eefc4053))
    - Avoid crashes when the terminal is resized to unusually small sizes. ([`caa1e72`](https://github.com/Byron/dua-cli/commit/caa1e7261bad1b0e2b10628aa14c9d2b6868a14a))
    - Mark-pane help bar now shows closest to the selected item. ([`24a6c29`](https://github.com/Byron/dua-cli/commit/24a6c29b3f48289cb6374aa66e84357edb5d0d54))
    - Update to the latest versions of `tui-react` and `ratatui` ([`8ab4ee7`](https://github.com/Byron/dua-cli/commit/8ab4ee72423bba54402525b4ea4593f0c582261e))
    - Fix possible overflow during substraction in mark pane ([`a94c7d3`](https://github.com/Byron/dua-cli/commit/a94c7d31ec152ff2427092054b99d8c4f3f74cfd))
    - Add scrollbar for mark list ([`5fe858d`](https://github.com/Byron/dua-cli/commit/5fe858d771d286204d2ed911533869223ea20d2c))
    - Add scrollbar to the main entries list. ([`0c511ff`](https://github.com/Byron/dua-cli/commit/0c511ffa0f15e16520353ff712f6bcc11318e379))
    - Avoid iterating a potentially long list doubly ([`fd797e8`](https://github.com/Byron/dua-cli/commit/fd797e86787ca1675e0f0406828c06506b4b1a11))
    - Add scrollbar for main list ([`120a08a`](https://github.com/Byron/dua-cli/commit/120a08aefeed9581f5d9110861b15ee0cbcd5831))
    - Merge branch 'dev/pwach/ignore-vs-code' ([`b79fde4`](https://github.com/Byron/dua-cli/commit/b79fde47d07a712101ab97127fc3b23acd105b13))
    - Remove user and platform specific files from `.gitignore` ([`30f28a7`](https://github.com/Byron/dua-cli/commit/30f28a7426ea4093a0205e02674bd43943130b88))
    - Ignore .vscode directory (same as .idea) ([`27cbd21`](https://github.com/Byron/dua-cli/commit/27cbd2128c1b4213b8b2bd354e8205b2f8e1878d))
    - Merge pull request #231 from gosuwachu/dev/pwach/fix-clippy ([`250fdc4`](https://github.com/Byron/dua-cli/commit/250fdc420e12634a195f23f461dda07c998cacea))
    - Update dependencies ([`0086854`](https://github.com/Byron/dua-cli/commit/00868548eee4b57a5646aef0d9e0cf8cd6de8226))
    - Fixes clippy error in rust 1.76 ([`85c00cd`](https://github.com/Byron/dua-cli/commit/85c00cd44f7e3dbd862c5d02a7f8310de7ead670))
    - Merge pull request #229 from chessmango-forks/main ([`5052da9`](https://github.com/Byron/dua-cli/commit/5052da9567b7ba75ebba015171a70202a10d26ee))
    - Different image obvi ([`f5ec49f`](https://github.com/Byron/dua-cli/commit/f5ec49f09e9699e2b5d538cd77c9facf236753b0))
    - Strip arm64 bin ([`504ac52`](https://github.com/Byron/dua-cli/commit/504ac521afca3f0791d2ac2c02adffb34d1a2e34))
    - Add aarch64 releases ([`a88baf1`](https://github.com/Byron/dua-cli/commit/a88baf1d9494118cca405aa2639656e1d05ba533))
</details>

## 2.28.0 (2024-01-23)

### New Features

 - <csr-id-78b9a8e22568c902132ed98d32e223ff71eb7b06/> add `dua i --no-entry-check` flag.
   With it, in interactive mode, entries will not be checked for presence.
   
   This can avoid laggy behaviour when switching between directories
   as `lstat` calls will not run, which can be slow on some filesystems.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
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
    - Release dua-cli v2.28.0 ([`984fd97`](https://github.com/Byron/dua-cli/commit/984fd979e18ffaa20ba35bca3b85dc47c404328c))
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

 - 4 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#223](https://github.com/Byron/dua-cli/issues/223)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#223](https://github.com/Byron/dua-cli/issues/223)**
    - Don't check entry metadata while a scan is in progress ([`c70ca81`](https://github.com/Byron/dua-cli/commit/c70ca81f007f925c7891340d0d0e763bcfc4114d))
 * **Uncategorized**
    - Release dua-cli v2.27.2 ([`e512f31`](https://github.com/Byron/dua-cli/commit/e512f31df3aeba1410df5e250dd74569a48fdf66))
    - Merge branch 'fix-overhead' ([`7a4b271`](https://github.com/Byron/dua-cli/commit/7a4b27153c2cb47caca87e28c5e178921c3a3fd9))
    - Allow `/` (glob-mode) while scanning. ([`67c5bdb`](https://github.com/Byron/dua-cli/commit/67c5bdb74cfcf8cab647888afec26cd09ccf543a))
</details>

## 2.27.1 (2024-01-21)

### Bug Fixes

 - <csr-id-f70d1a8e6ace812a7949cd7d0299507b71306d48/> Explicit refreshes with 'r and 'R' now work with multiple root paths as will.
   This can happen in cases of `dua i root-a root-b` for instance.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.27.1 ([`346b944`](https://github.com/Byron/dua-cli/commit/346b944a45d4cec12f7689718893ebdef0f46546))
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

 - 45 commits contributed to the release.
 - 11 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#96](https://github.com/Byron/dua-cli/issues/96)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#96](https://github.com/Byron/dua-cli/issues/96)**
    - Press `r` or `R` for refresh ([`bed351e`](https://github.com/Byron/dua-cli/commit/bed351ed2190e50e2932278b9b13b83c2969401b))
    - Prepare for (R)efresh support ([`1812227`](https://github.com/Byron/dua-cli/commit/181222745ed50b7346bfd082473168634e01fa99))
 * **Uncategorized**
    - Release dua-cli v2.27.0 ([`322eeb1`](https://github.com/Byron/dua-cli/commit/322eeb1aa07dacdc82e147bae64f8bfd4d758e1d))
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
    - Merge pull request #217 from Geezus42/patch-1 ([`85005ce`](https://github.com/Byron/dua-cli/commit/85005ce364b08066a76ab2a8743ce18b4662fa3c))
    - Update README.md ([`e0ac3d1`](https://github.com/Byron/dua-cli/commit/e0ac3d1916b1e58bb6fd3619a7c7dbd4ce201e77))
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
    - Update demo ([`0e2e8c6`](https://github.com/Byron/dua-cli/commit/0e2e8c6c422a29ad25f055da8dd1611746827795))
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

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#209](https://github.com/Byron/dua-cli/issues/209)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#209](https://github.com/Byron/dua-cli/issues/209)**
    - Responsive and buttery-smooth UI while scanning in interactive mode. ([`3c8a31b`](https://github.com/Byron/dua-cli/commit/3c8a31b50da8230bb9268b857e00d0c90e8cb786))
 * **Uncategorized**
    - Release dua-cli v2.26.0 ([`b19c3e0`](https://github.com/Byron/dua-cli/commit/b19c3e02b3f6a810a717235d9742466c8e1bf7d9))
    - Refactor ([`0651cae`](https://github.com/Byron/dua-cli/commit/0651cae13b43104402ed9d90147ee8c63fe83b61))
    - Optimize UI responsiveness during scanning state ([`983ba61`](https://github.com/Byron/dua-cli/commit/983ba6172604b83c2e4efad0f03273206a43c5db))
</details>

## 2.25.0 (2024-01-03)

<csr-id-e992659db17f275b48e555afd6b18df737401f01/>
<csr-id-729e7e92410b138f2778ef70f0f59a439028ac29/>

### Chore

 - <csr-id-e992659db17f275b48e555afd6b18df737401f01/> remove obsolete `tui-shared` feature

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

### Other

 - <csr-id-729e7e92410b138f2778ef70f0f59a439028ac29/> clarify that (and why) `termion` isn't supported anymore in README.md
   This was triggered by `crossterm` essentially breaking event handling
   on Windows, which is when I decided to just use the seemingly more
   powerful crossterm events natively.
   
   Overall, this made event handling more complex, but also allows users of
   `crosstermion` (the crate that actually dropped `termion` support) to
   write even more interactive applications without worrying about
   the always out-of-date intermediate layer.
   
   Interestingly, the `crosstermion` crate adds some useful features around event handling,
   but also optimizes build times thanks to `termion`, which is something that
   on my current machine I don't notice anymore, so the value of it diminished greatly.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 14 commits contributed to the release over the course of 7 calendar days.
 - 7 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#196](https://github.com/Byron/dua-cli/issues/196), [#208](https://github.com/Byron/dua-cli/issues/208)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#196](https://github.com/Byron/dua-cli/issues/196)**
    - `--ignore-dirs` now work as expected. ([`49f98f5`](https://github.com/Byron/dua-cli/commit/49f98f537bf0ac41a7b1992094103f6d36f135f8))
 * **[#208](https://github.com/Byron/dua-cli/issues/208)**
    - Remove obsolete `tui-shared` feature ([`e992659`](https://github.com/Byron/dua-cli/commit/e992659db17f275b48e555afd6b18df737401f01))
    - Clarify that (and why) `termion` isn't supported anymore in README.md ([`729e7e9`](https://github.com/Byron/dua-cli/commit/729e7e92410b138f2778ef70f0f59a439028ac29))
 * **Uncategorized**
    - Release dua-cli v2.25.0 ([`ad7c77a`](https://github.com/Byron/dua-cli/commit/ad7c77ac8db663e7a870dd73e45159462c1cbe60))
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

 - 4 commits contributed to the release.
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
    - Release dua-cli v2.24.2 ([`c973826`](https://github.com/Byron/dua-cli/commit/c9738265c40598f0a79befc8dd7be0d47a9f8179))
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

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.24.1 ([`46fece5`](https://github.com/Byron/dua-cli/commit/46fece5f295a8fb6f90ff969741f79d7c736c140))
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

 - 30 commits contributed to the release.
 - 12 days passed between releases.
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
    - Release dua-cli v2.24.0 ([`f7958cc`](https://github.com/Byron/dua-cli/commit/f7958ccb36ab6023857bc3cdfba9ef90fdfe0c99))
    - Update dependencies ([`bc8c7a6`](https://github.com/Byron/dua-cli/commit/bc8c7a6bb0a96cc1562e02cf87182d13be6c264f))
    - Upgrade owo-colors to v4.0 ([`141c56c`](https://github.com/Byron/dua-cli/commit/141c56c560b34f35e67a699c6e88bea0c0bef0ab))
    - Update changelog prior to release; bump version ([`cfa8faa`](https://github.com/Byron/dua-cli/commit/cfa8faaf30a83b9517d5e10fd2702bd7a44ba1ad))
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
    - Merge pull request #201 from Byron/dependabot/cargo/zerocopy-0.7.31 ([`b23e134`](https://github.com/Byron/dua-cli/commit/b23e13431dad1ed9efc6728f4c9ee8ab2254a42c))
    - Bump zerocopy from 0.7.30 to 0.7.31 ([`3efc857`](https://github.com/Byron/dua-cli/commit/3efc85767a6f3274f6fe0f249554ec7af2c2500f))
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

 - 17 commits contributed to the release.
 - 2 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#194](https://github.com/Byron/dua-cli/issues/194)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#194](https://github.com/Byron/dua-cli/issues/194)**
    - Single files will not cause IO error ([`192460e`](https://github.com/Byron/dua-cli/commit/192460e5bc72781be1d238912c5d590bfed706cf))
 * **Uncategorized**
    - Release dua-cli v2.23.0 ([`491b558`](https://github.com/Byron/dua-cli/commit/491b5587ca0e9d157b9a4f8907c35137d57983d5))
    - Merge branch 'main_windows_render_refactor' ([`2da2e2e`](https://github.com/Byron/dua-cli/commit/2da2e2e7d264d19cc67ccee6bd8658f7a87901c1))
    - Refactor ([`49772d1`](https://github.com/Byron/dua-cli/commit/49772d17dca72006e602f8707121b3378f948981))
    - Display the total count of entries-to-be-deleted in the mark pane. ([`98d5b5a`](https://github.com/Byron/dua-cli/commit/98d5b5a2728e640f9d553648812df379c5534395))
    - Refactor ([`81eadf8`](https://github.com/Byron/dua-cli/commit/81eadf8cdfcfa964401b5cf5d1e80cc21ec4441f))
    - Calculates mark pane item count consistently with the rest of the app ([`2c69ea1`](https://github.com/Byron/dua-cli/commit/2c69ea1faf40499431616e632e02351a22bac249))
    - Refactors MainWindow render to make it more readable ([`8740d4b`](https://github.com/Byron/dua-cli/commit/8740d4b332290b7fa661b157ed190df9f40ad349))
    - Merge branch 'upgrades' ([`a9dd549`](https://github.com/Byron/dua-cli/commit/a9dd549dc85faf17ce211ff0ab5be4c9863440ed))
    - Upgrade to latest crossterm; switch to `ratatui` from `tui` ([`af2aa61`](https://github.com/Byron/dua-cli/commit/af2aa61813578ecc9f6ccaba5e94049fc6ddf727))
    - Update all dependencies ([`852ee28`](https://github.com/Byron/dua-cli/commit/852ee281ca89cb58cf404e8179d4e590f6da87b7))
    - Merge branch 'total_item_count' ([`ba2efe4`](https://github.com/Byron/dua-cli/commit/ba2efe48f327c92c021879cded7651d83cf99cec))
    - Add total size to header bar and change to aggregate, human-readable item count. ([`3241022`](https://github.com/Byron/dua-cli/commit/3241022a730dab89f13cbefbefdb583fd6a00994))
    - Merge branch 'make_ready_pre_pr_checks' ([`606d60f`](https://github.com/Byron/dua-cli/commit/606d60fa96654e030d1f82331c54f3dfa45e06b3))
    - Adjust names of `make fmt` and `make ready`; `fmt` now only checks ([`9135c18`](https://github.com/Byron/dua-cli/commit/9135c18e27a75b23921f9bba4281af9543017a66))
    - Adds new target that runs all the checks (to be used before creating a PR) ([`7fbc136`](https://github.com/Byron/dua-cli/commit/7fbc136f8d61676a89cd19c6035ec02a8bd14147))
    - Displays total item count ([`7b7bad5`](https://github.com/Byron/dua-cli/commit/7b7bad5564d0e87eea4b4bd2d32066063a13b554))
</details>

## 2.22.0 (2023-12-09)

### New Features

 - <csr-id-45ccb7cb5a4765190ea6b8d02e0b29f63b1bd702/> Press `c` to sort by count of entries in a directory.
   That way it's easy to spot places that have a lot of (possibly small) files,
   which otherwise would remain under the radar when sorting by size.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 15 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.22.0 ([`949baf3`](https://github.com/Byron/dua-cli/commit/949baf39c53060d1e5ef6df572b1cdc43a8a5cce))
    - Press `c` to sort by count of entries in a directory. ([`45ccb7c`](https://github.com/Byron/dua-cli/commit/45ccb7cb5a4765190ea6b8d02e0b29f63b1bd702))
    - Assure sorting of entry-counts takes files into consideration. ([`8439ba7`](https://github.com/Byron/dua-cli/commit/8439ba703d7f16b2a8f5bd0348b63b26a5fbe689))
    - Refactor ([`9fb3113`](https://github.com/Byron/dua-cli/commit/9fb3113d788ff746873bd67f6ed508ec1fcf1b02))
    - Adds keybinding for 'c' to toggle sorting by number of items ([`8df0b4c`](https://github.com/Byron/dua-cli/commit/8df0b4c5dc5ee3f512f8812dff709a77cfb18f2f))
    - Merge branch 'column_render' ([`bf4da4e`](https://github.com/Byron/dua-cli/commit/bf4da4e1c4444fb490f85516efc518bb238e1652))
    - Run clippy locally just like on CI ([`645474c`](https://github.com/Byron/dua-cli/commit/645474cfc6de5456a9ae7c0b50ee0302ca950cd2))
    - Refactor ([`bbcd308`](https://github.com/Byron/dua-cli/commit/bbcd30886f71fcb6e804d3f4170c5ae332c181ea))
    - Fix visual changes ([`b8ad16b`](https://github.com/Byron/dua-cli/commit/b8ad16b493c29c56d94f6ec01a9dc790687a1bdb))
    - Make clippy warnings fatal ([`1a46d8f`](https://github.com/Byron/dua-cli/commit/1a46d8f4ab64c78c6ff6495225872f28e58f309d))
    - Refactors entries panel by moving code to separate functions ([`b5b6aba`](https://github.com/Byron/dua-cli/commit/b5b6abae35a5f205cd57e172c7aa4e9dd16d2053))
    - Put release-build optimization back to level 3; remove windows GNU build. ([`917339f`](https://github.com/Byron/dua-cli/commit/917339ff02e52cc3d258a350f5cb078e152f439a))
    - Change optimization level back to 0 for hopefully more stable release builds. ([`2dced34`](https://github.com/Byron/dua-cli/commit/2dced34aef9b7beba3201e6a8b24d9a10a7b9a16))
    - Assure package-size check isn't fatal anymore ([`3e509db`](https://github.com/Byron/dua-cli/commit/3e509db11228b93e11e8a06895dcb7d167fc8399))
    - Pacify the package size checkf or a while. ([`09fd59f`](https://github.com/Byron/dua-cli/commit/09fd59febbf7fefe93d064718d640290b2957b21))
</details>

## 2.21.0 (2023-12-06)

### New Features

 - <csr-id-de4c2b3bd368fd032319b606b84fa488299bc9e1/> With a single path provided as root, pretend it's the current working dir
   This makes it seem like the user started the directory walk directly in the given directory,
   which is more intuitive than the previous approach only showed the given directory as
   top-level directory.
   
   Note that this change only affects invocations like `dua <dir>` or `dua i <dir>`.
 - <csr-id-dd523e389bcc940a5d3e72099bb0c76f40371164/> press `m` to sort by modification date, ascending and descending.

### Bug Fixes

 - <csr-id-72fd6d1936efa508aeb63e729cf75739a49acdb4/> slightly faster (0.5ms) startup times of the binary on MacOS and maybe Linux.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 14 commits contributed to the release over the course of 11 calendar days.
 - 14 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#110](https://github.com/Byron/dua-cli/issues/110), [#141](https://github.com/Byron/dua-cli/issues/141), [#179](https://github.com/Byron/dua-cli/issues/179), [#185](https://github.com/Byron/dua-cli/issues/185), [#186](https://github.com/Byron/dua-cli/issues/186)

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
 * **[#185](https://github.com/Byron/dua-cli/issues/185)**
    - Slightly faster (0.5ms) startup times of the binary on MacOS and maybe Linux. ([`72fd6d1`](https://github.com/Byron/dua-cli/commit/72fd6d1936efa508aeb63e729cf75739a49acdb4))
 * **[#186](https://github.com/Byron/dua-cli/issues/186)**
    - Assure `device_id` is taken from the final CWD ([`74e6d42`](https://github.com/Byron/dua-cli/commit/74e6d4222a7f70253f1d69eb8e7cf94114827852))
 * **Uncategorized**
    - Release dua-cli v2.21.0 ([`7e3e290`](https://github.com/Byron/dua-cli/commit/7e3e290940e3229efce69baabab7d169d8a6793a))
    - Manually set the desired version as `cargo smart-release` fails to see that 2.20.3 is available ([`f5bd870`](https://github.com/Byron/dua-cli/commit/f5bd870621779711ef8ba03812f097716f52d349))
    - Prepare changelog ([`6ff009a`](https://github.com/Byron/dua-cli/commit/6ff009a3082c2c4dbe90b8188bf943df83f596d0))
    - Thanks clippy ([`0c4d31b`](https://github.com/Byron/dua-cli/commit/0c4d31b406b2c988af3f17fc79b0cf3d7364a910))
    - Skip through single root directory ([`e9fb2fd`](https://github.com/Byron/dua-cli/commit/e9fb2fda3478fefa38bdb9d176380bae5545dbc6))
    - Increase optimization level ([`3d270bd`](https://github.com/Byron/dua-cli/commit/3d270bd2bdd8e83486140c96d7e9c12988c93405))
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

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.20.3 ([`bb41fd5`](https://github.com/Byron/dua-cli/commit/bb41fd55cc476008342be9238568eb8f32dd40c4))
    - Mark-pane now doesn't double-count sizes anymore. ([`7ab0070`](https://github.com/Byron/dua-cli/commit/7ab0070dcfda573cfbdc8451ddba5fcf15067132))
    - Fixes marking parent directory for deletion counts children twice ([`f7086cc`](https://github.com/Byron/dua-cli/commit/f7086cc0836bd091552a83d8faabf937fb4c6cf8))
</details>

## 2.20.2 (2023-11-20)

### Bug Fixes

 - <csr-id-49c3e3d02ad0c14c4123fe1a7fea1f2a5e7a990f/> alignment when in interactive mode and -f binary

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#177](https://github.com/Byron/dua-cli/issues/177)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#177](https://github.com/Byron/dua-cli/issues/177)**
    - Alignment when in interactive mode and -f binary ([`49c3e3d`](https://github.com/Byron/dua-cli/commit/49c3e3d02ad0c14c4123fe1a7fea1f2a5e7a990f))
 * **Uncategorized**
    - Release dua-cli v2.20.2 ([`28fa9dd`](https://github.com/Byron/dua-cli/commit/28fa9ddd7ee30e228d1660dde95ad076e378fba7))
    - Update dependencies ([`12fa54a`](https://github.com/Byron/dua-cli/commit/12fa54a5c6af7720b27cffcc4c89e0728ec4cfda))
    - Upgrade journey-test expectations (doesn't run on CI for some reason) ([`5498040`](https://github.com/Byron/dua-cli/commit/54980406b72e9b633fec632e015da378e5d5a49d))
    - Upgrade `itertools` ([`0c8b6f2`](https://github.com/Byron/dua-cli/commit/0c8b6f2465c48b554faaa06bf0438cfff9df8a7f))
    - Upgrade `open` ([`192420e`](https://github.com/Byron/dua-cli/commit/192420ededd19779754d51d690ecdf7588d62441))
    - Fixes alignment when in interactive mode and -f binary ([`b3bb851`](https://github.com/Byron/dua-cli/commit/b3bb85177d2fc4b299a9d82313832be96b34c3b6))
    - Make clear which windows toolchain is used on CI ([`8c02612`](https://github.com/Byron/dua-cli/commit/8c02612887a49d72244327c9bf7940ff8fa507e6))
    - Update all dependencies ([`1bbee14`](https://github.com/Byron/dua-cli/commit/1bbee14695866294771d9113d9bab4d369b6401b))
</details>

## 2.20.1 (2023-05-05)

### Bug Fixes

 - <csr-id-3e8509d31291cbcaa0501b5ec4abf7f26a226836/> release definition to use a more recent ubuntu, to get linux binaries.
   Otherwise there are no linux runners who can satisfy the request, and they hang forever,
   instead of saying something useful.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#160](https://github.com/Byron/dua-cli/issues/160)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#160](https://github.com/Byron/dua-cli/issues/160)**
    - Release definition to use a more recent ubuntu, to get linux binaries. ([`3e8509d`](https://github.com/Byron/dua-cli/commit/3e8509d31291cbcaa0501b5ec4abf7f26a226836))
 * **Uncategorized**
    - Release dua-cli v2.20.1 ([`bf51db2`](https://github.com/Byron/dua-cli/commit/bf51db20de49dc1c240fcbbe1ddea3a4a6537873))
</details>

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

 - 14 commits contributed to the release.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#157](https://github.com/Byron/dua-cli/issues/157), [#33](https://github.com/Byron/dua-cli/issues/33), [#85](https://github.com/Byron/dua-cli/issues/85)

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#157](https://github.com/Byron/dua-cli/issues/157)**
    - Share `cargo` invocation to help install on more platforms ([`7ef48ad`](https://github.com/Byron/dua-cli/commit/7ef48ad5f326dcdd43f9b5481bf66bfce7d65074))
 * **[#33](https://github.com/Byron/dua-cli/issues/33)**
    - Improve documentation for `Marked Items` pane to make clearer how to delete items. ([`b61ec97`](https://github.com/Byron/dua-cli/commit/b61ec973b7437230183d6dabf361b0848519f5dc))
 * **[#85](https://github.com/Byron/dua-cli/issues/85)**
    - Use metric byte format only on MacOS. ([`22f54dd`](https://github.com/Byron/dua-cli/commit/22f54dd7c0e83b55e0acc2fb1a10ab487bdeb9fb))
 * **Uncategorized**
    - Release dua-cli v2.20.0 ([`9c07301`](https://github.com/Byron/dua-cli/commit/9c07301b5ee7cb290dd1df2297184d29fc43fd67))
    - TUI now shows performance metrics while scanning and after. ([`13bfe45`](https://github.com/Byron/dua-cli/commit/13bfe4582f8cbf6f8f12e7ee8acaae710e8a87d2))
    - Thanks clippy ([`565581f`](https://github.com/Byron/dua-cli/commit/565581fc11faf7512c27fe9095090f482a8d32f0))
    - Simplify GUI refreshes by using a throttle ([`c921dc7`](https://github.com/Byron/dua-cli/commit/c921dc72d3008179e72df9d85f0e0c21c998e199))
    - Generalize the throttle implementation to allow usagein UI ([`e03c560`](https://github.com/Byron/dua-cli/commit/e03c560e8b54e2e231d578e1d5e9dcd206d34216))
    - Added additional clarification for deleting help files. ([`fcc8be9`](https://github.com/Byron/dua-cli/commit/fcc8be93bd8224c01216ed2136cbf7309470ca2f))
    - Dependency update ([`032747d`](https://github.com/Byron/dua-cli/commit/032747d14587161dae1e4508799a232390ddfbae))
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

### Other

 - <csr-id-fe956ca6f244613762bb48de79eac1f6fa399e1b/> don't recurse on cross-device filesystems
   Like it says in the title. Right now, if you pass `-x`, dua doesn't
   count files on other devices, but it still enumerates them. However, a
   good reason to use `-x` is if you have network mounts that are slow, so
   this fixes that.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 17 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.19.2 ([`aa08b81`](https://github.com/Byron/dua-cli/commit/aa08b8100a6c2fcbbbfe1d249d082cfa6161e9d0))
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

 - 7 commits contributed to the release over the course of 52 calendar days.
 - 53 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#143](https://github.com/Byron/dua-cli/issues/143)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#143](https://github.com/Byron/dua-cli/issues/143)**
    - Redraw window while gathering metadata in interactive mode. ([`fb5a39f`](https://github.com/Byron/dua-cli/commit/fb5a39ffb67fad80be0d2090efd34d259d439e98))
 * **Uncategorized**
    - Release dua-cli v2.19.1 ([`e6c10c5`](https://github.com/Byron/dua-cli/commit/e6c10c5b311ad25d010f18d51f150d34bf62b815))
    - Update dependencies ([`662e754`](https://github.com/Byron/dua-cli/commit/662e7541d32f3d5628bfa76e1caa5978caef8bb7))
    - Draw window before processing events, fixes #143 ([`d957a61`](https://github.com/Byron/dua-cli/commit/d957a61ac79b990fa3cf470a9b500b6f390e3a18))
    - Update dependencies ([`f7de1ab`](https://github.com/Byron/dua-cli/commit/f7de1abe9e023abfbc6c757b60085fa8e6ecb538))
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

### Bug Fixes

 - <csr-id-d1cdfa1d682962deea5a0c48b90589becd6e19dc/> resolve stalling issue of previous version.
   This way, this release is the working version of v2.8.1 .

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.19.0 ([`53a7d15`](https://github.com/Byron/dua-cli/commit/53a7d15a1e7bb0efb70264e9bebcb08cccfdb74a))
    - Remove the handbrake on MacOS which can now deliver the expected performance. ([`f073375`](https://github.com/Byron/dua-cli/commit/f073375938f742db3259ec284c3c0d4a56fd0077))
    - Resolve stalling issue of previous version. ([`d1cdfa1`](https://github.com/Byron/dua-cli/commit/d1cdfa1d682962deea5a0c48b90589becd6e19dc))
</details>

## 2.18.2 (2022-12-13)

### Bug Fixes

 - <csr-id-3420dd4e14f3510662b58ae95ae7919ce0a98c95/> downgrade `jwalk` to 0.6 - the latest version does not actually iterate but instead keeps everything in memory.
   That's why `dua` didn't feel responsive anymore, was much slower and
   built up a lot of memory.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.18.2 ([`7633de4`](https://github.com/Byron/dua-cli/commit/7633de4956a5bd770b0db61f1e773b0cb6bfdfa8))
    - Downgrade `jwalk` to 0.6 - the latest version does not actually iterate but instead keeps everything in memory. ([`3420dd4`](https://github.com/Byron/dua-cli/commit/3420dd4e14f3510662b58ae95ae7919ce0a98c95))
    - Adjust maximum package size ([`8a4d8c2`](https://github.com/Byron/dua-cli/commit/8a4d8c27a8c94de6b62fb9d9bd5ba9031f9d6681))
</details>

## 2.18.1 (2022-12-13)

<csr-id-946806e7390799807361562b038fb12eeb2ddf11/>
<csr-id-d3fa946029ef44e5032762ff265180c23a629316/>

Update all dependencies to the latest version. This most notably changes the look of the CLI
to something without color by default thanks to the upgrade to `clap` 4.

### Chore

 - <csr-id-946806e7390799807361562b038fb12eeb2ddf11/> replace `colored` dependency with `owo-colors`.
   The latter provide zero-allocation coloring in the terminal and
   may improve compile times a little.

### Refactor

 - <csr-id-d3fa946029ef44e5032762ff265180c23a629316/> colored path printing; fix size column format

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 2 times to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.18.1 ([`58b53ff`](https://github.com/Byron/dua-cli/commit/58b53ff46274646833a0c07cf89d36301c0eb103))
    - Prepare changelog prior to release ([`4f7915a`](https://github.com/Byron/dua-cli/commit/4f7915ac4226b98ca423cb2010a706379afe4cee))
    - Upgrade to clap 4 ([`dd8b0ef`](https://github.com/Byron/dua-cli/commit/dd8b0ef8e12dfc8d7fb8f359f504c63034d60b9f))
    - Upgrade sysinfo and make thread detection work for all Apple M series for now. ([`bbd5c67`](https://github.com/Byron/dua-cli/commit/bbd5c67342f9e5b509b0ab6e9ca2319c3c7605e2))
    - Upgrade to trash v3.0 ([`bbd3a1c`](https://github.com/Byron/dua-cli/commit/bbd3a1ceaa15fb07c86ad9f4f5fcbaf991cb12af))
    - Thanks clippy ([`82dc467`](https://github.com/Byron/dua-cli/commit/82dc4670bd9b3b93ae949022ecdc58ead79cf905))
    - Update dependencies ([`0057c0d`](https://github.com/Byron/dua-cli/commit/0057c0d2ad0c1491ff1ffabc1aa5fea40539b942))
    - Adjust journey-test expectations according to difference in color encoding. ([`b0bacad`](https://github.com/Byron/dua-cli/commit/b0bacadbeec4ed91bb7a4ab2d783ed0824173274))
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

### Other

 - <csr-id-6a636d542594a76ef8b2faf2ec6347e4c8cb6b38/> switch from colored to owo-colors
   owo-colors is well-maintained, zero-allocation, zero-dependencies crate
   for terminal colors. Also it works on any type that implements Display trait,
   not just on strings.

### New Features

 - <csr-id-28f5ac90cc1ba7d668ae8a83eb5cd899294a8301/> Automatically resize if the terminal changes in size.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 54 calendar days.
 - 68 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#28](https://github.com/Byron/dua-cli/issues/28)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#28](https://github.com/Byron/dua-cli/issues/28)**
    - Automatically resize if the terminal changes in size. ([`28f5ac9`](https://github.com/Byron/dua-cli/commit/28f5ac90cc1ba7d668ae8a83eb5cd899294a8301))
 * **Uncategorized**
    - Release dua-cli v2.18.0 ([`7fe68ea`](https://github.com/Byron/dua-cli/commit/7fe68ead0222467092f67d49855655faf6d61ee4))
    - Update changelog ([`8dc45ab`](https://github.com/Byron/dua-cli/commit/8dc45abd15857cb11ab5567b83d507e4e3474b9e))
    - Make sure the correct version of `crosstermion` is used ([`ed0bfc7`](https://github.com/Byron/dua-cli/commit/ed0bfc750a59d45fad04cb3a39b73e6f70512cee))
    - Prepare changelog ([`50fd287`](https://github.com/Byron/dua-cli/commit/50fd2873257a17baa570d5f9826bcb0c83294059))
    - Merge branch 'dep-upgrade' ([`20b7672`](https://github.com/Byron/dua-cli/commit/20b76721939b77dc6c9a86d3c5f4c22cc7f1cf65))
    - Upgrade dependencies to tui 0.19/crossterm 0.25 ([`e35baea`](https://github.com/Byron/dua-cli/commit/e35baeaf25176e94ccc352b623f5cfd4b6b95b7b))
    - Switch from colored to owo-colors ([`6a636d5`](https://github.com/Byron/dua-cli/commit/6a636d542594a76ef8b2faf2ec6347e4c8cb6b38))
    - Add Apple M2 to default thread derivation ([`b5ec900`](https://github.com/Byron/dua-cli/commit/b5ec90042dec10fef8a35c27c2f7dcdb97b92293))
    - 'Other' -> 'Pre-built binaries' for clarity ([`6bd4338`](https://github.com/Byron/dua-cli/commit/6bd433831850c2032674e440809ac86c0b8c6784))
    - Add Scoop installation for Windows ([`8c67198`](https://github.com/Byron/dua-cli/commit/8c67198f6cc55bbc1a493f7a0e9fb5e9139d5ff3))
</details>

## 2.17.8 (2022-07-05)

### Bug Fixes

 - <csr-id-280d5436979f28c00b267d092e01c9675d3be7be/> remove `chrono` from the dependency tree.
   This assures there is no possibility for undefined behaviour to
   to localtime support used by some of the `trash`-crate code otherwise.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 20 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.17.8 ([`a4de545`](https://github.com/Byron/dua-cli/commit/a4de54582b6eb98795a84b312d701953f28849ce))
    - Speedup CI by adding a rust-cache ([`24df0b0`](https://github.com/Byron/dua-cli/commit/24df0b0fc24f5c6a4fe8a57a7023846309322d3f))
    - Remove `chrono` from the dependency tree. ([`280d543`](https://github.com/Byron/dua-cli/commit/280d5436979f28c00b267d092e01c9675d3be7be))
    - Dependency update ([`533b41d`](https://github.com/Byron/dua-cli/commit/533b41d22280450b883f6a2d340f5bb164e7221b))
</details>

## 2.17.7 (2022-06-14)

### Fixes

- Improve readability of the currently visible path in light terminal color themes [(#129)](https://github.com/Byron/dua-cli/pull/129).

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.17.7 ([`c0e3678`](https://github.com/Byron/dua-cli/commit/c0e3678c9f0eb61885c259b34355b4fd0ae0e3f3))
    - Update changelog prior to release ([`d437ba5`](https://github.com/Byron/dua-cli/commit/d437ba59ea04a264c8f715248969d3f9daef7b87))
    - Merge branch 'show-path' ([`1beb7d7`](https://github.com/Byron/dua-cli/commit/1beb7d7870a308e4829caada7ba3147326ffe0d4))
    - Restyle for compatibility with 'light' color schemes ([`ed28cdb`](https://github.com/Byron/dua-cli/commit/ed28cdbe979cf1fa4a2eccfc3a851fd94f7f2695))
</details>

## 2.17.6 (2022-06-12)

A maintenance release which should make the `ctrl + o` feature open files without blocking on linux
thanks to an upgrade in the `open` crate which powers this feauture.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 29 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.17.6 ([`4105a50`](https://github.com/Byron/dua-cli/commit/4105a50a2ffb38ae37b3e1a49e4d3a345cc59fab))
    - Update changelog prior to release ([`e20e9e0`](https://github.com/Byron/dua-cli/commit/e20e9e0bd3e3cd9525aa60655704df40bb8b448a))
    - Upgrade 'open' to latest version ([`6806241`](https://github.com/Byron/dua-cli/commit/68062418a32e9e69c78c17e1b4c33703752a2a02))
    - Dependency update ([`29a2abc`](https://github.com/Byron/dua-cli/commit/29a2abc33528ba9ddb8b62ad9dd98232bdf75aa9))
</details>

## 2.17.5 (2022-05-13)

### Bug Fixes

 - <csr-id-46a2871dd81fd47dba550ddae2cfeed90d97189d/> update to latest version of `trash` to improve trashing on linux
   See [their respective
   release](https://github.com/Byron/trash-rs/releases/tag/v2.1.2).

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#124](https://github.com/Byron/dua-cli/issues/124), [#127](https://github.com/Byron/dua-cli/issues/127)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#124](https://github.com/Byron/dua-cli/issues/124)**
    - Update to latest version of `trash` to improve trashing on linux ([`46a2871`](https://github.com/Byron/dua-cli/commit/46a2871dd81fd47dba550ddae2cfeed90d97189d))
 * **[#127](https://github.com/Byron/dua-cli/issues/127)**
    - Fix installation instructions, for real this time ([`3c3355e`](https://github.com/Byron/dua-cli/commit/3c3355ed9bc12bbf5ae17b5fea6c58323426cff8))
 * **Uncategorized**
    - Release dua-cli v2.17.5 ([`95fe11d`](https://github.com/Byron/dua-cli/commit/95fe11d8412b07e1143702aa269e90132cdddf99))
</details>

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

 - 6 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#124](https://github.com/Byron/dua-cli/issues/124)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#124](https://github.com/Byron/dua-cli/issues/124)**
    - Update changelog prior to release ([`0348b7d`](https://github.com/Byron/dua-cli/commit/0348b7dedd99e079a73df5765bc3c4db4e1e6f07))
    - Open interactive mode even if one of the input paths can't be read. ([`8742232`](https://github.com/Byron/dua-cli/commit/8742232a15c2bdd608c2e2c731a786c59c7d58dc))
    - Broken or non-existing root path will still print the valid results. ([`75b3eed`](https://github.com/Byron/dua-cli/commit/75b3eed98f14d918f474f73caa3cdedd5af927ad))
    - Record status quo ([`05e61a6`](https://github.com/Byron/dua-cli/commit/05e61a65e318694cfb2b98f9566bff817090d741))
 * **Uncategorized**
    - Release dua-cli v2.17.4 ([`6ce80b1`](https://github.com/Byron/dua-cli/commit/6ce80b14e26d256c3d2936c32b64c1a80b684b6f))
    - Merge branch 'broken-link-handling' ([`157b43c`](https://github.com/Byron/dua-cli/commit/157b43c2cb203c067c66f499a9fd849e5f0e811c))
</details>

## 2.17.3 (2022-05-10)

### Bug Fixes

 - <csr-id-aa2646d5ae4d931ef15787a9723daa007add4a91/> dependency update; upgrade to trash v2.1.1 .
   The trash upgrade makes sure that trashed items on mount points
   on freedesktop are actually restorable.
 - <csr-id-75b3eed98f14d918f474f73caa3cdedd5af927ad/> broken or non-existing root path will still print the valid results.
   Previously it would fail completely without printing anything.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#123](https://github.com/Byron/dua-cli/issues/123)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#123](https://github.com/Byron/dua-cli/issues/123)**
    - Update readme to reflect the changes in install.sh ([`086d0b2`](https://github.com/Byron/dua-cli/commit/086d0b257cc7488132f8c5ea1b550f352e30e828))
 * **Uncategorized**
    - Release dua-cli v2.17.3 ([`1f852ed`](https://github.com/Byron/dua-cli/commit/1f852ed5afd118d1f4804baf0574189f4d1f0b42))
    - Dependency update; upgrade to trash v2.1.1 . ([`aa2646d`](https://github.com/Byron/dua-cli/commit/aa2646d5ae4d931ef15787a9723daa007add4a91))
    - Fix cargo-diet check on CI ([`129c511`](https://github.com/Byron/dua-cli/commit/129c5114b15f1f644fa0c65266f13bed188ac161))
</details>

## 2.17.2 (2022-05-06)

A maintenance release that updates all dependencies. Most notably, `trash-rs` includes a fix for
properly moving files into the trash that required parent directories to be created.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 8 calendar days.
 - 46 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.17.2 ([`dd9f893`](https://github.com/Byron/dua-cli/commit/dd9f8933b75e052dbf3a13a9599061687690fcbe))
    - Update changelog prior to release ([`70581b6`](https://github.com/Byron/dua-cli/commit/70581b6ff384309ddc56d2650c8fef1f41e88d28))
    - Dependency update ([`8f3e157`](https://github.com/Byron/dua-cli/commit/8f3e157b86e7dd7c9669623aea03d7c74340d187))
    - Update dependencies ([`d8eae6e`](https://github.com/Byron/dua-cli/commit/d8eae6e8cf788ea8d69b3e73e83027f2f0e44391))
</details>

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

* The 64-bit atomic counter is replaced with an 8-bit AtomicBool
* All printing is controlled from the main thread
* The first line is cleared prior to printing a report

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

 - 9 commits contributed to the release over the course of 55 calendar days.
 - 57 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.17.1 ([`48c4462`](https://github.com/Byron/dua-cli/commit/48c446294e8ac6b620a2b7fc7c15a4cf9f839452))
    - Prepare changelog ([`fc1e10a`](https://github.com/Byron/dua-cli/commit/fc1e10a77da45d41c8243ddb07d7332ca8e23012))
    - Improve aggregate progress reporting ([`7d83f96`](https://github.com/Byron/dua-cli/commit/7d83f965d620ccebeda9a7451cdbb2e40ed88c24))
    - Update dependencies ([`9a1da6b`](https://github.com/Byron/dua-cli/commit/9a1da6bc4e964912a521b2f0de0bdf6124749ccd))
    - Upgrade sysinfo ([`0b6b52f`](https://github.com/Byron/dua-cli/commit/0b6b52f02b72641a4954838fd9e2ea4fd0447e2d))
    - Adjust to changes in clap ([`f9df024`](https://github.com/Byron/dua-cli/commit/f9df02420d7bd4e492c4a9130833fdf31e739909))
    - Dependency update ([`0d9fbd3`](https://github.com/Byron/dua-cli/commit/0d9fbd386c51be1995aaee70d1a87a1217d9c550))
    - Update clap to official release ([`b029dc5`](https://github.com/Byron/dua-cli/commit/b029dc5d190b23bf3e3fc95a3947f28f868e674e))
    - Upgrade to TUI 0.17 ([`9ce96ac`](https://github.com/Byron/dua-cli/commit/9ce96ac7b89a1ee39cd85a7c18871309d5fe07af))
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

 - 4 commits contributed to the release.
 - 12 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.17.0 ([`4025174`](https://github.com/Byron/dua-cli/commit/4025174e081c7820f8808262e67b96741bd44781))
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

 - 9 commits contributed to the release over the course of 60 calendar days.
 - 74 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#116](https://github.com/Byron/dua-cli/issues/116)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#116](https://github.com/Byron/dua-cli/issues/116)**
    - Add `--ignore-dirs` option, with useful default on linux ([`26d6514`](https://github.com/Byron/dua-cli/commit/26d65145650cc3aac4ad540fdf04e95e139812e3))
 * **Uncategorized**
    - Release dua-cli v2.16.0 ([`a132acb`](https://github.com/Byron/dua-cli/commit/a132acb8fa342e3f16b5f6a4bb31f5962a1f53c2))
    - Update changelog ([`7abddbf`](https://github.com/Byron/dua-cli/commit/7abddbfc74e65ecaf3aa1f2cf7506daf3ddb4bd9))
    - Build on platforms without 64-bit atomics ([`756ca54`](https://github.com/Byron/dua-cli/commit/756ca542a73575df581433fdd84cee8f4bef99b5))
    - Release dua-cli v2.15.0 ([`4b71a56`](https://github.com/Byron/dua-cli/commit/4b71a56bc428663249b2f20dbf19507ad559967d))
    - Update changelog ([`a226d1e`](https://github.com/Byron/dua-cli/commit/a226d1e8e4f0be2d9651950846424dda7e2c63b9))
    - Upgrade clap ([`87d8c45`](https://github.com/Byron/dua-cli/commit/87d8c45b105722352f58b2020aaeaff62f3e00f6))
    - Upgrade and update dependencies ([`269c650`](https://github.com/Byron/dua-cli/commit/269c650872b442e793604391cc5c94dc9fa592fc))
    - Fix link to releases ([`c27da8b`](https://github.com/Byron/dua-cli/commit/c27da8b9bf3d2ea091ff9267d2e96df05a17bf05))
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

 - 5 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#111](https://github.com/Byron/dua-cli/issues/111)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#111](https://github.com/Byron/dua-cli/issues/111)**
    - Cargo install without --locked should work now ([`f26309c`](https://github.com/Byron/dua-cli/commit/f26309c91a271f1c2c32dfb55dbbb8c713f5e97d))
 * **Uncategorized**
    - Release dua-cli v2.14.11 ([`7807c8a`](https://github.com/Byron/dua-cli/commit/7807c8aeef3953e4049f91fcc0597e4ff8018ed9))
    - Adjust changelog ([`bd6a1fd`](https://github.com/Byron/dua-cli/commit/bd6a1fd6202a3d1cb0fd5b17bb858c04fd18235c))
    - Thanks clippy ([`6cff8bc`](https://github.com/Byron/dua-cli/commit/6cff8bc4aea9ac0c93903fcf1357d29a3b9fea0b))
    - Remove superfluous line in release.yml ([`a0625fc`](https://github.com/Byron/dua-cli/commit/a0625fc7070efbca360176aef1a522d2290da086))
</details>

## 2.14.10 (2021-10-26)

### Bug Fixes

 - <csr-id-e220eef3f3fef4abed85807f8606b1c92527f950/> see if releases work now with a different create-release action
   We are only interested in the upload_url, not in actually creating a
   release as smart-release does that already.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.14.10 ([`12e1ad8`](https://github.com/Byron/dua-cli/commit/12e1ad81f8e791b911520343540dfa39bcfc6ef2))
    - See if releases work now with a different create-release action ([`e220eef`](https://github.com/Byron/dua-cli/commit/e220eef3f3fef4abed85807f8606b1c92527f950))
</details>

## 2.14.9 (2021-10-26)

### Bug Fixes

 - <csr-id-d0c2c7cbac9b9dfa18a85a48098f1492c629bfd6/> try to produce release binaries once more
   With smart-release, this is created automatically.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.14.9 ([`ea93ac3`](https://github.com/Byron/dua-cli/commit/ea93ac3efe09e043c6e6711abd0611a5d5af7228))
    - Try to produce release binaries once more ([`d0c2c7c`](https://github.com/Byron/dua-cli/commit/d0c2c7cbac9b9dfa18a85a48098f1492c629bfd6))
    - Update package size to match new changelog ([`9bfc2ea`](https://github.com/Byron/dua-cli/commit/9bfc2ea3040148c3c4e9dd03db3cc9a0b0e7eb0c))
</details>

## 2.14.8 (2021-10-26)

### Changed

 - <csr-id-49193f0506946981bc056b29c3f09c94e30ac457/> auto-config support for Apple M1 Pro and Apple M1 Max

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 38 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.14.8 ([`b9a9b3e`](https://github.com/Byron/dua-cli/commit/b9a9b3ec113430f44982e07c64bfbdde661779b6))
    - Use `cargo changelog` ([`e0b8328`](https://github.com/Byron/dua-cli/commit/e0b8328bde652a02f1f764975a8bf4b2f3619e17))
    - Cleanup changelog ([`c80b1c5`](https://github.com/Byron/dua-cli/commit/c80b1c5017f2679183d1dfc5edc6d379150fbe2a))
    - Auto-config support for Apple M1 Pro and Apple M1 Max ([`49193f0`](https://github.com/Byron/dua-cli/commit/49193f0506946981bc056b29c3f09c94e30ac457))
</details>

## v2.14.7 (2021-09-18)

* Fix deletion which broke with Rust 1.55, for those who are compiling the tool themselves.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 26 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.14.7 ([`07b934f`](https://github.com/Byron/dua-cli/commit/07b934f4e17e0b180d1734a810da3b533a29e43b))
    - Prepare release ([`f5fd8c6`](https://github.com/Byron/dua-cli/commit/f5fd8c6bfa4fb3756b73e29fb53dd553b1c20710))
    - Fix deletion process on Rust 1.55 ([`f45681a`](https://github.com/Byron/dua-cli/commit/f45681aa523fa6cc9d451ef46a8ce62f2ef99bf8))
</details>

## v2.14.6 (2021-08-22)

* Support for arrow keys as well as Home & End. The help pane was updated to reflect these changes.
* More readable information on how to delete or trash files in the mark pane.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 3 calendar days.
 - 5 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.14.6 ([`c148b77`](https://github.com/Byron/dua-cli/commit/c148b779e9eb1ef109fe9276fc378c9d7a553e37))
    - Update change log ([`f48b181`](https://github.com/Byron/dua-cli/commit/f48b181d56d89c3028eb055c80bdf447fe65595d))
    - Merge branch 'style' ([`5904630`](https://github.com/Byron/dua-cli/commit/5904630cfebd4e99bc4ee7a9c23550f85add41d4))
    - Update changelog ([`58bcf90`](https://github.com/Byron/dua-cli/commit/58bcf90ffec21edea8327ba11b6bbc6fcf1440c1))
    - Support Home/End and fix inconsistent help text ([`29017f6`](https://github.com/Byron/dua-cli/commit/29017f6f94003f58118ad7d1fded1d47f79349eb))
    - Improve mark widget tip style ([`019e4cb`](https://github.com/Byron/dua-cli/commit/019e4cb65e6d6302e08692c446bac56fb3beee25))
    - Format correctly ([`8977c17`](https://github.com/Byron/dua-cli/commit/8977c17bcb10373c33d695dd682781fd9590e4e7))
    - Remove unnecessary line ([`d6bbb6d`](https://github.com/Byron/dua-cli/commit/d6bbb6dd91b5367f8bd1f8569d39dbb30b8f89a2))
</details>

## v2.14.5 (2021-08-16)

* Fix installation via `cargo install dua-cli`. Please note that it might break again as it still depends on the unsable `clap-3 beta 4`. Even when pinning it breakage is possible as its dependencies itself aren't pinned.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 11 calendar days.
 - 11 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release dua-cli v2.14.5 ([`b74388c`](https://github.com/Byron/dua-cli/commit/b74388c7f6bc5a759663b98c8fa95db1e0941691))
    - Fix #102, bump patch level ([`3a6c654`](https://github.com/Byron/dua-cli/commit/3a6c654dc2939b5979c47d8fbd14932741f8d1d1))
    - Add NetBSD installation instructions ([`9501d08`](https://github.com/Byron/dua-cli/commit/9501d087d03801568d36df5ebba03515c36e592a))
    - Sysinfo upgrade ([`6827975`](https://github.com/Byron/dua-cli/commit/6827975b74e5cc66ffb7397e5fb3a144d287f1d5))
    - Add aggregate-scan-progress feature to help with #99 ([`7429cb3`](https://github.com/Byron/dua-cli/commit/7429cb3d1139605abdf3efcb8a4d5cceb300be1b))
</details>

## v2.14.4 (2021-08-05)

* upgrade depencies
* upgrade to tui 0.16

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 6 calendar days.
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
    - (cargo-release) version 2.14.4 ([`3987e7c`](https://github.com/Byron/dua-cli/commit/3987e7c51b4b27fd4c95def42ce3e585dc46c7c6))
    - Update dependencies; upgrade to tui-0.16 ([`80a40e5`](https://github.com/Byron/dua-cli/commit/80a40e583791caff575eea257ae7a38fadbc9542))
    - Thanks clippy ([`4598d64`](https://github.com/Byron/dua-cli/commit/4598d64a1150967e48013091e044eae851de62f9))
</details>

## v2.14.3 (2021-07-25)

* upgrade `open` crate to v2

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 11 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.14.3 ([`8222d99`](https://github.com/Byron/dua-cli/commit/8222d993a3afd05e17566b6b30d349b6e4080e0d))
    - Upgrade open to v2 ([`98c859c`](https://github.com/Byron/dua-cli/commit/98c859c71d9ee4be4c19bc436a494f035a241bc1))
</details>

## v2.14.2 (2021-07-14)

* `Ctrl-T` to trash (instead of removal) is now an optional default feature, allowing it to be
  disabled on FreeBSD which isn't currently supported.
* Update dependencies

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 14 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.14.2 ([`64a5589`](https://github.com/Byron/dua-cli/commit/64a5589ef93c03cfb0815e893250918dde6a9ea6))
    - Update changelog ([`e037a96`](https://github.com/Byron/dua-cli/commit/e037a96682b816a1855578cd08bb90dd8e123570))
    - Also run 'make check' on CI now that more feature toggles are added ([`9d2f969`](https://github.com/Byron/dua-cli/commit/9d2f969772306b35eab0b74cb792aac79d1d6af1))
    - Merge branch 'optional-trash' ([`b12b98a`](https://github.com/Byron/dua-cli/commit/b12b98a07935c839a11af08cfa9dc872b5a127e8))
    - Disable test that now starts failing on windows even though… ([`64175e0`](https://github.com/Byron/dua-cli/commit/64175e028965958d0c22f8ffe55cab2fc01f9fc8))
    - Dependency upgrade: petgraph 0.6 ([`b4aeb14`](https://github.com/Byron/dua-cli/commit/b4aeb149cffae440560b54dcae6211eef51e85e4))
    - Dependency update ([`163bd47`](https://github.com/Byron/dua-cli/commit/163bd4764c7b8d35eb8a49af8e96c61430621b20))
    - Refactor ([`6894dd8`](https://github.com/Byron/dua-cli/commit/6894dd8db51cd6fe8a70ad0c906ef351dc0a720c))
    - Add checking and testing of new feature toggle ([`ee680b9`](https://github.com/Byron/dua-cli/commit/ee680b9b82618a1d5ecab1fb2e431fe3ff64d130))
    - Make the trash feature optional ([`1fdded1`](https://github.com/Byron/dua-cli/commit/1fdded129fe766729ac332fa881c0681c9495316))
</details>

## v2.14.1 (2021-06-30)

* Pressing `ctrl+t` in the mark pane now trashes entries instead of deleting them. Not only does that make
  'deletion' reversible but it makes removal of the entry faster in many cases as well.
* updated dependencies

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.14.1 ([`5ecd90f`](https://github.com/Byron/dua-cli/commit/5ecd90fb400c61649826d80c0d1348affd10087e))
    - Upgrade sysinfo ([`e1b8a01`](https://github.com/Byron/dua-cli/commit/e1b8a01579e211c268356ea25c56cfb9391ca717))
    - Prepare patch release ([`0bf969f`](https://github.com/Byron/dua-cli/commit/0bf969f7017f34e626ee892f24e7bacc62e0a5c5))
    - Cargo fmt ([`97a9804`](https://github.com/Byron/dua-cli/commit/97a980436ab46693804ad0a361ab0388f34c8381))
    - Dependency update ([`93cd08d`](https://github.com/Byron/dua-cli/commit/93cd08df930e7f5f5164bc2b9d0979a5794c05be))
</details>

## v2.14.0 (2021-06-30)

<csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/>

### Other

 - <csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/> deduplicate code

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 20 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.14.0 ([`bbc0719`](https://github.com/Byron/dua-cli/commit/bbc0719d489d0484e7f770129fad9839ed2cc5c9))
    - Prep changelog ([`e7de79a`](https://github.com/Byron/dua-cli/commit/e7de79af3304ad9ed70cdf2e9fbe8ad4c765317a))
    - Merge branch 'trash' ([`64d8dc8`](https://github.com/Byron/dua-cli/commit/64d8dc8b9baf0fd2e8942b1391f783fe8a7d4586))
    - Thanks clippy ([`68bbb68`](https://github.com/Byron/dua-cli/commit/68bbb68ffd4887d2023a520e4dfc69b9d8edc736))
    - Add mark pane prompt message for ctrl + t ([`af538bc`](https://github.com/Byron/dua-cli/commit/af538bc545c3b3b7c0a3d5541a1a80b0da536e5b))
    - Deduplicate code ([`02dd1b7`](https://github.com/Byron/dua-cli/commit/02dd1b72c8fe741fb153094fdb08816f7f593c6f))
    - Implement Ctrl+t move to trash ([`00fae90`](https://github.com/Byron/dua-cli/commit/00fae90e0dffc468c75bd362fa4220bc8650fb86))
</details>

## v2.13.1 (2021-06-09)

<csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/>

* Allow usage of the feature introduced in v2.13 by writing the TUI to stderr instead of stdout.
  That way the output can be redirected.

### Other

 - <csr-id-02dd1b72c8fe741fb153094fdb08816f7f593c6f/> deduplicate code

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
    - (cargo-release) version 2.13.1 ([`5534cd7`](https://github.com/Byron/dua-cli/commit/5534cd7126eada8a040f00cd996295dfd42cb4c1))
    - Prepare for version bump ([`d0150a8`](https://github.com/Byron/dua-cli/commit/d0150a8686b8265ca92a930b2d3676e1c89e2402))
    - Show TUI on stderr to enable writing files to stdout ([`a93a642`](https://github.com/Byron/dua-cli/commit/a93a642765540d4010dc2fab90737cd39abaa32d))
</details>

## v2.13.0 (2021-06-08)

* Print remaining marked paths upon exit on stdout. This may help to use `dua i` with other programs
  who want to process the marked paths on their own.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.13.0 ([`1bfcc63`](https://github.com/Byron/dua-cli/commit/1bfcc6306739f4dfbe076acdbe53bf59143e9245))
    - Prepare release ([`140a656`](https://github.com/Byron/dua-cli/commit/140a6560b57aec819ba678e2f9c9a1d975c794af))
    - Print marked items upon exit if these are left in the marked pane ([`017cbd7`](https://github.com/Byron/dua-cli/commit/017cbd7b4c3e57e1a98fbc595159be39bc97c708))
</details>

## v2.12.2 (2021-06-07)

* Prepare for release of new Apple hardware and be more specific when auto-configuring the correct amount of threads.
  Instead an error message will be printed to inform that the given CPU brand isn't configurable yet.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 7 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.12.2 ([`c8d5650`](https://github.com/Byron/dua-cli/commit/c8d5650be77e000801b282c4c0a3861e710de6d8))
    - Prepare new release ([`f45852a`](https://github.com/Byron/dua-cli/commit/f45852a5880fbcd9670f0de3643ea9614ec35de4))
    - Set default processor count on Apple Silicon in a way that won't be totally wrong in future ([`fe9611a`](https://github.com/Byron/dua-cli/commit/fe9611a7fd9a1592cc1a4517948b4a32fba904c9))
    - Refactor ([`c3c103e`](https://github.com/Byron/dua-cli/commit/c3c103eebd82fc729788694a9f3bfd4ded855cf8))
    - Dependency update ([`1fb6bad`](https://github.com/Byron/dua-cli/commit/1fb6badaf653305618c62f7ba4f4332c1c1ab959))
    - Refactor ([`115db26`](https://github.com/Byron/dua-cli/commit/115db26ab86fcb50dd14b12b64240b66bbac53f1))
</details>

## v2.12.1 (2021-05-30)

* Fixed bug that would cause `dua` to unconditionally sleep for 1 second. This sleep was intended for a spawned thread,
  but it slipped into the main thread.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.12.1 ([`06377e5`](https://github.com/Byron/dua-cli/commit/06377e560488e16da185c68c2a0069fd4389fe59))
    - Fix terrible bug causing an unnecessary wait in front of each invocation ([`ac604b3`](https://github.com/Byron/dua-cli/commit/ac604b35c0b80fa6b380cc395a95bf0a5d1d196d))
    - Fix tests ([`dfb40a2`](https://github.com/Byron/dua-cli/commit/dfb40a20d1e697d2f3fc3a159febf9adb3a817b2))
    - Only fetch metadata for files for a speedup ([`d381c6c`](https://github.com/Byron/dua-cli/commit/d381c6caed1fd404d7a11c1f581abdba749b7a20))
    - Mildly optimize progress performance… ([`ffdb0c2`](https://github.com/Byron/dua-cli/commit/ffdb0c270f9c07a3518e2335ee77d7788bfc7793))
</details>

## v2.12.0 (2021-05-29)

YANKED.

* Add minimal progress for when `dua` invocations take longer than 1 second

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 19 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.12.0 ([`939af68`](https://github.com/Byron/dua-cli/commit/939af68f2a50d67e1c85acac49b4047e3dcbe5a9))
    - Only display progress on if stderr is a tty ([`a0d6288`](https://github.com/Byron/dua-cli/commit/a0d628898226e272e9f29137da148991e07f3641))
    - Add simple progress to indicate something is happening on long `dua` runs ([`e68481f`](https://github.com/Byron/dua-cli/commit/e68481f3524d214b76d2895a10febc3a524c3256))
    - Thanks clippy ([`78a68b1`](https://github.com/Byron/dua-cli/commit/78a68b1a9ed5d39d250c5478041e40425a198756))
    - Add similar programs to README ([`60f4324`](https://github.com/Byron/dua-cli/commit/60f432417fe2adbbd54de7293f1c3ffcd45365f7))
</details>

## v2.11.3 (2021-05-09)

* re-add arm builds
* dependency updates (including tui 0.15)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 6 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.11.3 ([`41f0e6d`](https://github.com/Byron/dua-cli/commit/41f0e6d37448535af0bf3ce504e62ec622a2dc74))
    - Prepare releas ([`08eb0e2`](https://github.com/Byron/dua-cli/commit/08eb0e2034779bd0df7899f75cbd30531103cd9c))
    - Dependency updates ([`25f0cb0`](https://github.com/Byron/dua-cli/commit/25f0cb08613be98b84845c49b345921e0a78342b))
    - Re-add arm builds ([`a7db17d`](https://github.com/Byron/dua-cli/commit/a7db17de1528dedd6bcc083a28e575eb9be34885))
</details>

## v2.11.2 (2021-05-03)

* dependency updates (including tui 0.15)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 40 calendar days.
 - 69 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Thanks Clippy

<csr-read-only-do-not-edit/>

[Clippy](https://github.com/rust-lang/rust-clippy) helped 1 time to make code idiomatic. 

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Adjust release workflow to be less specific to ripgrep ([`4becf36`](https://github.com/Byron/dua-cli/commit/4becf36bb16054e9939fb48d45d57e1e7da1e603))
    - Upgrade release workflow file from ripgrep ([`12a01f1`](https://github.com/Byron/dua-cli/commit/12a01f136b04fc633ffe09939343ce1cbc9cc886))
    - (cargo-release) version 2.11.2 ([`1ffc52e`](https://github.com/Byron/dua-cli/commit/1ffc52e0a93150f3d0d488ceb515ce5f4caea816))
    - Fix build (use the latest version of crosstermion, too) ([`b675446`](https://github.com/Byron/dua-cli/commit/b6754461bcb7bfbd1794986e41114f59738fa955))
    - Remove tui-react, it now lives in https://github.com/Byron/tui-crates ([`1ddbeae`](https://github.com/Byron/dua-cli/commit/1ddbeae87dc0c23edf412405d6a08696bc703c1b))
    - Prepare changelog for patch release ([`e16a3e4`](https://github.com/Byron/dua-cli/commit/e16a3e4908cdfed103c0c1d5e54c31f1c90d40df))
    - [dua] actually upgrade to tui 0.15 ([`296b5a7`](https://github.com/Byron/dua-cli/commit/296b5a7172233b030a3995aa72c361873029bc65))
    - [dua] upgrade to tui 0.15 ([`a9ce757`](https://github.com/Byron/dua-cli/commit/a9ce7578bcbc088c8b18e33de83860e10991bf85))
    - [tui-react] upgrade tui to 0.15 ([`27fb521`](https://github.com/Byron/dua-cli/commit/27fb5214e8f2c4669faf093a2ca570da17deca37))
    - Fix help menu typo ([`98d973f`](https://github.com/Byron/dua-cli/commit/98d973fdf1cea099bfe963e9b1736ab2cac08a35))
    - Add installation instructions via homebrew ([`94b8cfb`](https://github.com/Byron/dua-cli/commit/94b8cfb9250da9f77f857b615a1461e748e04a27))
    - Dependency update ([`3f335f0`](https://github.com/Byron/dua-cli/commit/3f335f033a10381a61918bc87c40d461d9c1de8a))
    - Run actions on main ([`7f3c3a4`](https://github.com/Byron/dua-cli/commit/7f3c3a4facebcd6daf2c8532087204904adf38d0))
    - Enable funding ([`6907724`](https://github.com/Byron/dua-cli/commit/6907724b1856466d9603fcab1b59450e6973aadb))
    - New resolver using Rust 1.51 ([`1575ad2`](https://github.com/Byron/dua-cli/commit/1575ad2441c9e4ec034c4256237c7f22908eb875))
    - Thanks clippy ([`59279d4`](https://github.com/Byron/dua-cli/commit/59279d464aac8c3985720d1d46b0a190b4443d2f))
</details>

## v2.11.1 (2021-02-22)

<csr-id-59315b7c63b7328fa70bfe5fc43fdbe9dc5f92e7/>

* The `-x/--stay-on-filesystem` flag is now respected for multiple root paths, as in `dua -x
  path-FS1/ path-FS2/`, as such `dua` will stay in FS1 if the CWD is in FS1.

### Other

 - <csr-id-59315b7c63b7328fa70bfe5fc43fdbe9dc5f92e7/> add MacPorts install instructions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 5 calendar days.
 - 6 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.11.1 ([`2808ff6`](https://github.com/Byron/dua-cli/commit/2808ff645f421aa2b098e3245e76890edad7ce98))
    - Update changelog ([`e5d3752`](https://github.com/Byron/dua-cli/commit/e5d3752c296a859711cf158f1f84a5829bcfa333))
    - Respect 'stay_on_filesystem' when no input files are provided ([`33f81d6`](https://github.com/Byron/dua-cli/commit/33f81d6f56d1c324548a7b6d8a06bac168821516))
    - Update dependencies ([`ae5c9b8`](https://github.com/Byron/dua-cli/commit/ae5c9b896b83b0841069908bc2220312591ed197))
    - Add MacPorts install instructions ([`59315b7`](https://github.com/Byron/dua-cli/commit/59315b7c63b7328fa70bfe5fc43fdbe9dc5f92e7))
</details>

## v2.11.0 (2021-02-15)

### Features

* Add binding capital 'H' to go to the top of any pane/list
* Add binding capital 'G' to go to the bottom of any pane/list

### Fixes
* Without user input during `dua i [<multiple paths>]` the top-most entry will remain selected.
* Avoid stale frame at the end of traversal in interactive sessions when there is no user input.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
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
    - (cargo-release) version 2.11.0 ([`3f4773f`](https://github.com/Byron/dua-cli/commit/3f4773feb937e461b3596fbf13ec28409efb4acc))
    - Adjust changelog prior to release ([`ad7c779`](https://github.com/Byron/dua-cli/commit/ad7c7796ecea46557ab851eb15ed2a20fd1e2447))
    - Enforce drawing once after traversal is done ([`ee73690`](https://github.com/Byron/dua-cli/commit/ee7369022611745ec9c55beddf1b907f13ed3559))
    - Keep selecting the first element during iteration unless… ([`6d7b3cd`](https://github.com/Byron/dua-cli/commit/6d7b3cd062214f2cc66886d49d1a60406204abf3))
    - Thanks clippy ([`6ca9e6c`](https://github.com/Byron/dua-cli/commit/6ca9e6ca52a4d4d32036df2914ee773ab313397b))
    - Add bindings 'H' and 'G' to go to the top/bottom of any pane ([`8b606ac`](https://github.com/Byron/dua-cli/commit/8b606ac464ec5fa3979ab73fef4d29733d389760))
</details>

## v2.10.10 (2021-01-23)

<csr-id-9384cdb5b95e5260f46ccd23e7ca276304190a34/>

Fix --version flag.
It looks like the latest BETAs of clap removed setting the version implicitly.

### Other

 - <csr-id-9384cdb5b95e5260f46ccd23e7ca276304190a34/> fix typo

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 15 calendar days.
 - 15 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.10.10 ([`8cc2f44`](https://github.com/Byron/dua-cli/commit/8cc2f44b4cd89cc046f1748f664d112d0278aa6d))
    - Fix --version ([`1ba3c1c`](https://github.com/Byron/dua-cli/commit/1ba3c1cce9ae9419633f1e197b76c87649e9174a))
    - Dependency update ([`8b602bd`](https://github.com/Byron/dua-cli/commit/8b602bd31fb172fb7f222e68d320787315fbcefb))
    - Fix typo ([`9384cdb`](https://github.com/Byron/dua-cli/commit/9384cdb5b95e5260f46ccd23e7ca276304190a34))
</details>

## v2.10.9 (2021-01-07)

Fix build.

Now that `jwalk` was released in v0.6 with v0.5.2 yanked, `cargo install` will use the previous
version v0.5.1 which does not fit the latest `dua` anymore.

This is now fixed and hopefully permanently so thanks to using `jwalk` v0.6.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.10.9 ([`d5bd682`](https://github.com/Byron/dua-cli/commit/d5bd68259678f48b61608245c1444ffa297131bd))
    - Fix jwalk, the other way around; related to #72 ([`0b0265d`](https://github.com/Byron/dua-cli/commit/0b0265df38adacb86d9b39986c251490eebfb232))
    - Upgrade to tui 14 ([`27e65a2`](https://github.com/Byron/dua-cli/commit/27e65a2fc91b22cb5816864f51d1d3a3ce11a94a))
    - Bump tui version to 0.14 ([`d32ab34`](https://github.com/Byron/dua-cli/commit/d32ab34e2b8521ddbbbaacd08d48b983cb792432))
</details>

## v2.10.8 (2021-01-04)

<csr-id-dc100c8b4a838c92f39d5a67da7eea06e7dec9af/>

Fix build.

A breaking change in jwalk can cause builds to fail. This prevents the issue from spreading at least
with dua-cli.

### Other

 - <csr-id-dc100c8b4a838c92f39d5a67da7eea06e7dec9af/> bump itertools 0.9.0 -> 0.10.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 18 calendar days.
 - 18 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.10.8 ([`523a0c6`](https://github.com/Byron/dua-cli/commit/523a0c6f44f767115da631b85e479d5cedd75674))
    - Update changelog ([`3cb794d`](https://github.com/Byron/dua-cli/commit/3cb794dc89ce13cf10632de13d1f8ec91646c537))
    - Bump itertools 0.9.0 -> 0.10.0 ([`dc100c8`](https://github.com/Byron/dua-cli/commit/dc100c8b4a838c92f39d5a67da7eea06e7dec9af))
    - Dependency update ([`420f1f6`](https://github.com/Byron/dua-cli/commit/420f1f677b77acd73729df19edf2849c65d8d33b))
    - Increase  crate size limit ([`041e218`](https://github.com/Byron/dua-cli/commit/041e218c47f77ea60e982a4e92209e5574cf6336))
</details>

## v2.10.7 (2020-12-16)

Better performance on Apple Silicon (M1).

The IO subsystem on Apple Silicon is different and won't scale nicely just by using all amount of available cores. Instead it seems best to only
use as many threads as performance cores are present on the system - otherwise the performance might actually get worse while using more power.

On all other systems, the default number of threads did not change.

**Please note that for optimial performance** one would need an arm build on MacOS, currently provided is only intel builds.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 9 calendar days.
 - 31 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Dependency update ([`019ec45`](https://github.com/Byron/dua-cli/commit/019ec459b853095aa322a2e297039eea5a5f5939))
    - (cargo-release) version 2.10.7 ([`d1faaac`](https://github.com/Byron/dua-cli/commit/d1faaac20efd8eda07ff8564e834eae8062a5828))
    - Prepare next release ([`20d9094`](https://github.com/Byron/dua-cli/commit/20d9094a6d604badc4e70c9d1f45bca65f35c849))
    - Select better default thread count on Apple Silicon (M1) ([`a1cf012`](https://github.com/Byron/dua-cli/commit/a1cf012f36269d97953baac9288b2fc5551bc6a0))
    - Hopefully fix release pipeline ([`7c40f95`](https://github.com/Byron/dua-cli/commit/7c40f95b4e05eacfbdb0e3267d443f4642c9f80b))
    - Dependency update ([`848c3ed`](https://github.com/Byron/dua-cli/commit/848c3edc45ef645f8403673dfca9764f62ecb51e))
</details>

## v2.10.6

Fix `dua -h` usage string.

## v2.10.5 (2020-11-15)

Dependency update.

* upgrade to TUI v0.13.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`a13e27b`](https://github.com/Byron/dua-cli/commit/a13e27be7d01f226baeabf04c1007f85d3e5b849))
    - Custom usage to fix #71 ([`018b00d`](https://github.com/Byron/dua-cli/commit/018b00db339f9772922007e293567231164b330b))
    - Switch from structup to clap 3 beta.2 ([`5782c4f`](https://github.com/Byron/dua-cli/commit/5782c4ff99b70ea101ed2f36711a456fd4e4e37b))
</details>

## v2.10.4 (2020-11-15)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 19 calendar days.
 - 30 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Patch bump ([`88753aa`](https://github.com/Byron/dua-cli/commit/88753aa6d6a7d23a7d4334b7913655009adfc079))
    - Upgrade to tui 0.13 ([`98da03d`](https://github.com/Byron/dua-cli/commit/98da03d4db2edf8d4ab37d761ec166f467d4cab8))
    - Update tui-react to tui v0.13 ([`2d11a19`](https://github.com/Byron/dua-cli/commit/2d11a191fbdccd3e16b6542743854151d4ebbc5d))
    - Dependency update ([`daad381`](https://github.com/Byron/dua-cli/commit/daad3817e314b972294730c880536142521dee30))
    - Show 'scanning' note even without entering a directory ([`8992625`](https://github.com/Byron/dua-cli/commit/8992625fe2bfc8ceb371a86733bb3900e4caf3d9))
    - Update README to reflect only working installation methods ([`9a38f1f`](https://github.com/Byron/dua-cli/commit/9a38f1fc12a3326646e053b4700dd0a593ffbde8))
    - Disable release-build test mode in preparation for merge ([`24f040a`](https://github.com/Byron/dua-cli/commit/24f040a27a3afbab63b439439afd65d53602dd5e))
    - See if ARM works again ([`db47b37`](https://github.com/Byron/dua-cli/commit/db47b375db9ee8a94aec40d6c0ac430085f6bab1))
</details>

## v2.10.3 (2020-10-15)

Dependency update.
Should fix [this issue](https://github.com/Byron/dua-cli/issues/66)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 42 calendar days.
 - 79 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 2.10.3 ([`c32322f`](https://github.com/Byron/dua-cli/commit/c32322f29a120712a75f95585e4d3a3d700c538b))
    - Dependency update ([`6cb8209`](https://github.com/Byron/dua-cli/commit/6cb8209d48f0832b99f497c011c81d1e1a7c6a95))
    - Dependency update ([`c7cdf36`](https://github.com/Byron/dua-cli/commit/c7cdf368a06797e8ca73a3c621a3e451883c0937))
    - Provide alternative installation instructions for linux ([`53d31a7`](https://github.com/Byron/dua-cli/commit/53d31a76242dcf4b2395526beadbb34a48164c7e))
    - Upgrade to latest version of tui ([`872bbbc`](https://github.com/Byron/dua-cli/commit/872bbbc0d630ce5ccf17a6847c6b12846f745997))
    - Update to tui 0.12 ([`3e1b8c2`](https://github.com/Byron/dua-cli/commit/3e1b8c202638b5067f794f8d3687834eb3d4b450))
    - Dependency update ([`9a877e2`](https://github.com/Byron/dua-cli/commit/9a877e2401b1d5f5751047867a7067fd7fdc473c))
    - Dependency update ([`56a365b`](https://github.com/Byron/dua-cli/commit/56a365b5ee21f09bb80afb32d0184b150f16f4c2))
    - Dependency update ([`dadb3fe`](https://github.com/Byron/dua-cli/commit/dadb3fe70d3bb15a5cc1f2e5d8d0307faaa9d702))
</details>

## v2.10.2 (2020-07-27)

Change light-grey color in command-line mode to Cyan to fix disappearing text.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`b38d234`](https://github.com/Byron/dua-cli/commit/b38d23483973595940d500310a89ec3f525895be))
    - Refactor ([`cdc5ee3`](https://github.com/Byron/dua-cli/commit/cdc5ee36d2c7c6bc6ecc9676ebaa408066a9eb5a))
    - Src, aggregate: fix colors for aggregate mode ([`4d2e839`](https://github.com/Byron/dua-cli/commit/4d2e83904fd66a3d480b5f50ad6fa2192d113a3f))
</details>

## v2.10.1 (2020-07-24)

Change light-grey color in interactive mode to Cyan to fix disappearing text.

See [this PR](https://github.com/Byron/dua-cli/pull/62) for reference.

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
    - (cargo-release) version 2.10.1 ([`31c588e`](https://github.com/Byron/dua-cli/commit/31c588eaf30f34f2df23c3cc28ee8aebe5a01ca0))
    - Update changelog ([`c939b2c`](https://github.com/Byron/dua-cli/commit/c939b2c9a1405a9f364a10c2f692267f0879e1df))
    - Fix styling for folders (cyan=folders, not chagned - regular files) ([`2cc6916`](https://github.com/Byron/dua-cli/commit/2cc69169282a07a485992bf95969cf6f81981b08))
    - Fix clippy warnings ([`292c4d3`](https://github.com/Byron/dua-cli/commit/292c4d30722592b3e5ab1d779b5502cb0d129999))
</details>

## v2.10.0 (2020-07-22)

Minor improvements of looks; improved windows support.

* previously in interactive mode on Windows, directory sizes would appear as 0 bytes in size. This is now fixed!

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 33 commits contributed to the release over the course of 14 calendar days.
 - 15 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Arm also has a problem now - ignore it for now ([`04b9e52`](https://github.com/Byron/dua-cli/commit/04b9e52b9edf5f1b0490e7a55ec99891cf404b46))
    - And one more… ([`601eee2`](https://github.com/Byron/dua-cli/commit/601eee2219f6f135e65b5783e3180a82d8f316c0))
    - Nigthly is definitely required for windows builds, let's hope that works ([`5b7696c`](https://github.com/Byron/dua-cli/commit/5b7696cb2c40a14a5deb26193d8d74212a01c141))
    - Seems nightly is broken right now - stable it is everywhere ([`d7a7f9c`](https://github.com/Byron/dua-cli/commit/d7a7f9ccf810f90e80b4b04c0252ed8eab2b17e7))
    - Try again to make things build on linux, argh! ([`f520072`](https://github.com/Byron/dua-cli/commit/f5200723e8cda73045be4a65c4bf11ad9a4a023d))
    - Try to build on stable on arm (which fails otherwise now) ([`8efa046`](https://github.com/Byron/dua-cli/commit/8efa04659864c9260deca6515a6b0428cc4278ae))
    - Minor style improvements to handle special case ([`69a2490`](https://github.com/Byron/dua-cli/commit/69a2490844d87c09cd5cc51da49e3cd87a03c35a))
    - Avoid jump when cycling through byte visualization ([`4f91292`](https://github.com/Byron/dua-cli/commit/4f912929f213c00f6721995bfc5ee0b8879d80e9))
    - (cargo-release) version 0.10.1 ([`b5d1a21`](https://github.com/Byron/dua-cli/commit/b5d1a21e50f2d64abeb79c9c108839c1fb27bb0e))
    - Fix incorrect render area of tui-react list ([`3715b71`](https://github.com/Byron/dua-cli/commit/3715b714c83cdbbe7230d85ae87e5f93c07160e0))
    - Fix mark pane ([`b4476ba`](https://github.com/Byron/dua-cli/commit/b4476bac270e2d1cdeb0f28bf7528d95b770a7e3))
    - Help is back to normal ([`8c2a174`](https://github.com/Byron/dua-cli/commit/8c2a174ed31cfc6e7095cf1cf4dbc24bf38ea975))
    - Help looks better now, but is far from 'normal' ([`29ee421`](https://github.com/Byron/dua-cli/commit/29ee421dd40666c53f659692a9a55cf8874cee1a))
    - Switch to crosstermion 0.3 for tui 0.10 support ([`fd8c441`](https://github.com/Byron/dua-cli/commit/fd8c441af3739027b7959a21b530ddb4da455f73))
    - Merge remote-tracking branch 'origin/master' ([`4812206`](https://github.com/Byron/dua-cli/commit/4812206eab68ea5588d93f9ea0589f9e772ee5ad))
    - Use published version of tui-react ([`ed1f91b`](https://github.com/Byron/dua-cli/commit/ed1f91b42890998b255567f32e8049a842552937))
    - Upgrade to tui 0.10 step one… ([`839b932`](https://github.com/Byron/dua-cli/commit/839b9323d93b9f562f6414cd66504b6d686c0224))
    - Fix path construction of 'sample_02_tree' for test ([`5a36cd1`](https://github.com/Byron/dua-cli/commit/5a36cd18a31ca1fbdc62d4e594933a6327fe4e7d))
    - Fix platform size difference of 'sample_01_tree' for test ([`62c5833`](https://github.com/Byron/dua-cli/commit/62c58330b41cb19adde1c7d2b08a5db251be3580))
    - Tui-react now works with tui 10.0; tracks tui's version number now ([`773497c`](https://github.com/Byron/dua-cli/commit/773497cc48a406a069be84e14194d51484fdbec2))
    - Re-enable test, disabled accidentally ([`48cbe09`](https://github.com/Byron/dua-cli/commit/48cbe0919da1dd6aa8c933b5d156e7f0ce5997a8))
    - Update to colored 2.0 ([`72e776d`](https://github.com/Byron/dua-cli/commit/72e776d9a3668a81a9502e9560c06a2e500a37c8))
    - Fix test on windows - it's breaking now since #53 is fixed ([`1207bdd`](https://github.com/Byron/dua-cli/commit/1207bdd582c75895354b639fb81006d97076da83))
    - Dependency update ([`f7f2118`](https://github.com/Byron/dua-cli/commit/f7f211802edeff5c1981ab8bfe01517639f79e19))
    - Don't pay extra on linux for helping with #53 ([`d18191d`](https://github.com/Byron/dua-cli/commit/d18191d8b19471eabc34526070bcc440edd72626))
    - Use full path for obtaining the 'real size on disk' ([`22a13fb`](https://github.com/Byron/dua-cli/commit/22a13fbea06199151d5cdf2f3a0533984111e0b3))
    - Speedup build times by not optimizing build dependencies ([`16e00de`](https://github.com/Byron/dua-cli/commit/16e00de6821675f8c4a0ed8500c2abfaa3af3bb0))
    - Replace flume with just std::sync::mpsc ([`ba78ae4`](https://github.com/Byron/dua-cli/commit/ba78ae433d1ea905bf1efd751cec34901e509caa))
    - Update dependencies ([`901d29d`](https://github.com/Byron/dua-cli/commit/901d29df066e8974b272c742ca4f9a9c7aa49dbc))
    - Update dependencies ([`78448e6`](https://github.com/Byron/dua-cli/commit/78448e62bb50284a85fdf03b289049eecc1ee265))
    - Patch bump tui-react ([`7fbd933`](https://github.com/Byron/dua-cli/commit/7fbd93302566b19427e2b9432abd2cd131651983))
    - Calculate block width without going through graphemes ([`9702296`](https://github.com/Byron/dua-cli/commit/97022961a0d7f65c605f71f764b766b29866c4c7))
    - Update dependencies ([`69edd7c`](https://github.com/Byron/dua-cli/commit/69edd7c1b109a443565c6fd9d2e23d2e030031dd))
</details>

## v2.9.1 (2020-07-07)

Globs for Windows; fixed handling of colors.
* On widnows, `dua` will now expand glob patterns by itself as this capability is not implemented by shells `dua` can now run in.
* A bug was discovered that could cause `dua a` invocation to now show paths behind their size in an incorrect attempt to not print with color.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`42a5067`](https://github.com/Byron/dua-cli/commit/42a5067eacf10cfdca7b1d5df92748c9855fefa3))
    - Merge branch 'rivy-fix.win' ([`edd0d74`](https://github.com/Byron/dua-cli/commit/edd0d74a12096f83c4b75ffd021c31dcbc269a46))
    - Fix color handling (causing the text to disappear); fix tty detection ([`82d005b`](https://github.com/Byron/dua-cli/commit/82d005b9e3ed9ce8d4441c607ec160f2f0a48b1c))
    - Add windows wildcard argument support (using `wild`) ([`2c73b4d`](https://github.com/Byron/dua-cli/commit/2c73b4d59603c12d31ded1a2f2ca9ef97a5ff0b3))
    - Fix windows compiler warnings (unused_variables) ([`5a11216`](https://github.com/Byron/dua-cli/commit/5a11216b53af2644100fcfebe44b0b6eea2dbb78))
</details>

## v2.9.0 (2020-07-06)

Full windows support!

* On Windows, we will now build using `crossterm`, which was greatly facilitated by `crosstermion`.
* On Unix systems, the backend is still `termion`.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 20 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Cut new release: 2.9 ([`becae48`](https://github.com/Byron/dua-cli/commit/becae48c29aa2db036f097516959d60cc219bc03))
    - Releases are working as expected ([`230bd1d`](https://github.com/Byron/dua-cli/commit/230bd1d338cae861f1390b4db0dc58c8ea1491d4))
    - Skip one test on windows ([`fece423`](https://github.com/Byron/dua-cli/commit/fece4231cd24409b0772a820cee18c2922d45e5b))
    - Fix release.yml ([`eac0702`](https://github.com/Byron/dua-cli/commit/eac07027c3e9baff2d73ebfa7cc3ce752c0a8303))
    - Windows is nightly only right now ([`034c7ec`](https://github.com/Byron/dua-cli/commit/034c7ec6abbed58688d82a4e703fdb10864af58f))
    - Setup main branch for release build testing ([`50eb08b`](https://github.com/Byron/dua-cli/commit/50eb08b1b23714ab43e9457b92ec799440a0bc37))
    - Don't implicitly pull in termion! Kills windows build reliably… ([`d57cdca`](https://github.com/Byron/dua-cli/commit/d57cdca7e57c40e51fdaec760e92b111dc69ad0f))
    - Inform about a certain decision related to tui backend support ([`676c6a9`](https://github.com/Byron/dua-cli/commit/676c6a99be6a604fa0508a8335e3a2f9dad206e7))
    - Make interactive mode optional, allow selection of backend for windows, unix ([`464829e`](https://github.com/Byron/dua-cli/commit/464829e11f5d6d63019ec167e2e1b1b7c0061f0a))
    - Add preliminary windows test for building ([`d0c362a`](https://github.com/Byron/dua-cli/commit/d0c362ae0f0f7ff4d49d899591c6cbb205e6b191))
    - Completely rid ourselves of Termion to make backend selection possible ([`0e760d7`](https://github.com/Byron/dua-cli/commit/0e760d733108a7e3a2153b4cee03f33ef13e5cd4))
    - Replace termion::color with colored ([`40e9eb1`](https://github.com/Byron/dua-cli/commit/40e9eb1d0e548dac3ec896d293291d1e439ba976))
    - Termcolor spends 1200 lines on handlings buffers, and it's not liking plain io::Write ([`e867e58`](https://github.com/Byron/dua-cli/commit/e867e58ebd2febc66342f0337f08b75574b24e02))
    - For a moment I thought 'colored' could be used, but… ([`86f16c3`](https://github.com/Byron/dua-cli/commit/86f16c3042d9f8ba400512c8f2916c3a40e2d1f8))
    - Always use crossterm for now just to test if it works and… ([`3e0d4b0`](https://github.com/Byron/dua-cli/commit/3e0d4b022ff8d6ce5115894f3b6ad68f01ff370f))
    - Use crosstermion to create a terminal with the corresponding backend ([`98f850a`](https://github.com/Byron/dua-cli/commit/98f850a1ccd30618620a7d78999899c24463238a))
    - Allow case-insensitivity with byte format variants ([`4b59c36`](https://github.com/Byron/dua-cli/commit/4b59c36ca8c53e63dd74fc0b3179a4ed9de2f60d))
    - Convert input handling to crosstermion ([`388a134`](https://github.com/Byron/dua-cli/commit/388a1347580df120cead11f98516ceb911373316))
    - Show possible variants of byte formats ([`fddc8cb`](https://github.com/Byron/dua-cli/commit/fddc8cbcadb50a6ad2bf06e883fe751f3bca55b3))
    - Put Freaky into the changelog :) ([`b46cd3a`](https://github.com/Byron/dua-cli/commit/b46cd3a4920155cffbaecaf1ec8efe0ec245c531))
</details>

## v2.8.2 (2020-07-02)

* Switch back to `clap` from `argh` to support non-UTF-8 encoded paths to be passed to dua

I hope that `argh` or an alternative will one day consider supporting os-strings, as it would in theory be an issue
for anyone who passes paths to their command-line tool.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`4b965d7`](https://github.com/Byron/dua-cli/commit/4b965d76f096815b75759064bbf635d35b701560))
    - Make aliases visible in generated docs ([`531fbf1`](https://github.com/Byron/dua-cli/commit/531fbf1d5b4107cc54a426559e552d818e1d5735))
    - Bring structopt back, argh doesn't support OsStrings ([`e32778b`](https://github.com/Byron/dua-cli/commit/e32778b00dd38bc2053d325453ec19f498b68a29))
</details>

## v2.8.1 (2020-07-02)

* Switch from deprecated `failure` to `anyhow` to reduce compile times a little and binary size by 130kb.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`10aecc0`](https://github.com/Byron/dua-cli/commit/10aecc0ce7d33afc1fdbe8ce88b1aa871f055cf8))
    - Use 'anyhow' instead of 'failure' to simplify code and reduce bloat ([`af7a09c`](https://github.com/Byron/dua-cli/commit/af7a09c53faf9ebeeb8c0a15278b510738d1f34f))
</details>

## v2.8.0 (2020-07-02)

* Switched from `clap` to `argh` for a 300kb reduction in binary size and 1 minute smaller compile times.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump minor version ([`9ac025f`](https://github.com/Byron/dua-cli/commit/9ac025f7e546514581aaa96f96b8af476988d384))
    - All tests work with argh (which really needs aliases) ([`03e9a2a`](https://github.com/Byron/dua-cli/commit/03e9a2ac143c269d2c44a6bd13a0da10ede8bf38))
    - First version of options struct based on Argh ([`d787a9c`](https://github.com/Byron/dua-cli/commit/d787a9c5b8ccadae678c985b05ecc328d62df8f3))
</details>

## v2.7.0 (2020-07-02)

* [Support for extremely large][issue-58], zeta byte scale, files or filesystem traversals.
* [Fix possibly incorrect handling of hard links][pr-57] in traversals spanning multiple devices.

Both changes were enabled by [@Freaky](https://github.com/Freaky) whom I hereby thank wholeheartedly :).

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 29 calendar days.
 - 31 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade byte-unit to version 4 ([`8040d5c`](https://github.com/Byron/dua-cli/commit/8040d5c50df32b6b19b775a88bdc9616fbfe8980))
    - Update dependencies ([`1d61587`](https://github.com/Byron/dua-cli/commit/1d61587ce0e783019e5f3cb2a8acdd8c5eb93cca))
    - Fix unittests, at least to work locally on MacOS ([`1ce39f9`](https://github.com/Byron/dua-cli/commit/1ce39f9427b30adccf3e62751625b2296a333ca0))
    - Cut a new minor release: 2.7 ([`841a9d5`](https://github.com/Byron/dua-cli/commit/841a9d55fe1c4d76276616eab17274a45391bdcb))
    - Use u128 for byte sizes ([`1d8ba52`](https://github.com/Byron/dua-cli/commit/1d8ba524ac83a0c3b5e4146cf937ed75650f1e97))
    - Fix inode filtering with multiple devices ([`c37ee44`](https://github.com/Byron/dua-cli/commit/c37ee449f32ed3af0fc222f669ae3f40859d8a39))
    - Add more information about what it means to 'quit more quickly' ([`0ee7e06`](https://github.com/Byron/dua-cli/commit/0ee7e06589baace8fd453e67ac78db5ca3e1553d))
</details>

## v2.6.1 (2020-05-31)

* quit without delay from interactive mode after `dua` was opened on huge directories trees. 
  See [this commit](https://github.com/Byron/dua-cli/commit/91aade36c71e4e14167030b6ec8c3c13dcdc1b2b) for details.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 27 commits contributed to the release over the course of 11 calendar days.
 - 26 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`5eae4e3`](https://github.com/Byron/dua-cli/commit/5eae4e32dd6c2e7d0714605cddda81bef32347c6))
    - Avoid deallocation a potentially big hashmap ([`91aade3`](https://github.com/Byron/dua-cli/commit/91aade36c71e4e14167030b6ec8c3c13dcdc1b2b))
    - Abort on panic for smaller binaries; update dependencies ([`31778d7`](https://github.com/Byron/dua-cli/commit/31778d7517cf27f5a5effccc7373b71833546098))
    - Check package size limit in CI using cargo-diet ([`4dfb18f`](https://github.com/Byron/dua-cli/commit/4dfb18fe86cbe881b71de2db2faa43e8206e9a4f))
    - Fix install script instructions ([`6d15037`](https://github.com/Byron/dua-cli/commit/6d1503759774510ca9509175efd5785b41b9482d))
    - Optimize crate size with `cargo diet -r` ([`ca2dc43`](https://github.com/Byron/dua-cli/commit/ca2dc43b5aa1c0a2f025a697c9956f29d1bf0fe4))
    - Remove unused files ([`bb40674`](https://github.com/Byron/dua-cli/commit/bb406748e2b7e6cc047ebb4f9262c2f5d51f8dbb))
    - Add information about Windows installations ([`f0f20af`](https://github.com/Byron/dua-cli/commit/f0f20af237d7acddf4de3ae13673f44617728cf4))
    - Disable test mode ([`8dbf7e6`](https://github.com/Byron/dua-cli/commit/8dbf7e6a8e512378949939c9613fef5417a602c8))
    - See if all targets work! ([`002678e`](https://github.com/Byron/dua-cli/commit/002678e0a369802e8e245fa3ddacd2e2d7cc8eeb))
    - Add windows-by-handle feature to lib.rs, where it probably has to be ([`cc1930a`](https://github.com/Byron/dua-cli/commit/cc1930ab6c387628cd1f2ba3499d64b7a523ad5f))
    - Remove now unneeded specialized code to try checking out the repo ([`7318d07`](https://github.com/Byron/dua-cli/commit/7318d0774322a9ecfd958cafc6e2bfe48e1cfa79))
    - Remove paths windows chokes on ([`82d2d51`](https://github.com/Byron/dua-cli/commit/82d2d51e5bf3398808d2dbce6c3964ce6c53660e))
    - Try with manual sparse checkout :D ([`9935b3f`](https://github.com/Byron/dua-cli/commit/9935b3fdb9d901302019d7dbeb9d4c2060325359))
    - No clone needed, can just checkout sparsely ([`62e6c3e`](https://github.com/Byron/dua-cli/commit/62e6c3ed2e9f45afe229872eafa7937617329840))
    - Better checkout code, based on what the checkout action does ([`67ca691`](https://github.com/Byron/dua-cli/commit/67ca691b5a6afa0608a4dd3d5042229a18508ad8))
    - Need debug info :D ([`cb3b636`](https://github.com/Byron/dua-cli/commit/cb3b636b249dd20ea216e601d7ca21adce36dfbe))
    - Let's see what we actually checkout ([`20d194f`](https://github.com/Byron/dua-cli/commit/20d194f408a04fc21e9c58c38d22a577d87f594a))
    - Job shouldn't fail if checkout fails - looks like sparse checkout works! ([`93ffeb1`](https://github.com/Byron/dua-cli/commit/93ffeb1bb70683f60d10eae9e0dd91fb4e4c8748))
    - Try to get it cloned one more time ([`ff8482a`](https://github.com/Byron/dua-cli/commit/ff8482aea09a13ff24921b24e0849f4df858b429))
    - Maybe continue-on-error makes failures successes? ([`cab78dd`](https://github.com/Byron/dua-cli/commit/cab78dd0fa0df3aa9f17915832a04f8b4ac44a33))
    - Fix 'append file to other file' for windows; try again to trigger sparse checkout ([`904c484`](https://github.com/Byron/dua-cli/commit/904c48434befab6c54cc5e4c1d81c52f29988a82))
    - Right, leading exclamation marks in yaml! ([`b351b1d`](https://github.com/Byron/dua-cli/commit/b351b1d776cc68859737d0380302abce86b3e003))
    - Bump artifact version ([`b7220a8`](https://github.com/Byron/dua-cli/commit/b7220a8cb38c05a71b3e2f35e98b6b672c8d9479))
    - Try to use a sparse checkeout if standard checkout fails ([`a37a66a`](https://github.com/Byron/dua-cli/commit/a37a66a51f12ace6f4aa5be0e04bfdf6246cffb4))
    - Try windows release binaries ([`15b0b0b`](https://github.com/Byron/dua-cli/commit/15b0b0bfe33af3b74be69be631b22df666883922))
    - Fix crossdev to support windows (as originally intended) ([`3884ea6`](https://github.com/Byron/dua-cli/commit/3884ea66d74a0a04beb24e7c12144ac8245d4b95))
</details>

## v2.6.0 (2020-05-04)

* Use `x` to only mark entries for deletion, instead of toggling them.
* Add `-x` | `--stay-on-filesystem` flag to force staying on the file system the root is on, similar to `-x` in the venerable `du` tool.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 22 calendar days.
 - 29 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade to tui 0.9 ([`42c541a`](https://github.com/Byron/dua-cli/commit/42c541ac1977cef5169981c5996820214da9c937))
    - Update dependencies ([`a078086`](https://github.com/Byron/dua-cli/commit/a078086ce7fad108929afc7c8f24ab7c05b1be46))
    - Add '-x' flag to not cross filesystems ([`9156cf7`](https://github.com/Byron/dua-cli/commit/9156cf7cac8f91a496f7383940f3ce6140ffe54c))
    - Fix cargo fmt ([`a5988d0`](https://github.com/Byron/dua-cli/commit/a5988d091b437315a91accd21f6f1b61d21e2e9a))
    - Add 'x' key to mark for deletion, without toggling ([`5cedded`](https://github.com/Byron/dua-cli/commit/5cedded25d10800805d6717381bf2981e270e23d))
    - Mild refactor ([`5c1a04b`](https://github.com/Byron/dua-cli/commit/5c1a04bb108eefdb6e10294fef0681cf92ecbaad))
    - Fix clippy lints ([`83804ad`](https://github.com/Byron/dua-cli/commit/83804adf605c2d1264b0fcafcdbf5f77023570ab))
    - Link Rust badge to actions ([`9b3de55`](https://github.com/Byron/dua-cli/commit/9b3de5547d418697e7f094513e80dee4d00c21ff))
    - Add fmt and clippy lints ([`bc4fe3a`](https://github.com/Byron/dua-cli/commit/bc4fe3aebf5a728a30dcd31c6b06d883c3c2a745))
    - Bye bye travis, we had a really good time… ([`6d91259`](https://github.com/Byron/dua-cli/commit/6d91259c03591eb65c26a709d5906d98ea42b1ed))
    - Update badges ([`66f2bf7`](https://github.com/Byron/dua-cli/commit/66f2bf7a223dbd80457df730a7f282b793a2f10e))
</details>

## v2.5.0 (2020-04-05)

Much more nuanced percentage bars for a more precise visualization of space consumption.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 5 calendar days.
 - 6 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump minor ([`1027e9d`](https://github.com/Byron/dua-cli/commit/1027e9da425fda430b4be054a085d32972ef3c2d))
    - Fix compile errors after porting commit ([`26b9569`](https://github.com/Byron/dua-cli/commit/26b9569472ffb300d7019dbed5524fdbf688c6b8))
    - Add eighth sections to bar ([`82333ac`](https://github.com/Byron/dua-cli/commit/82333ac619e95a0635c20e9bc16b364b5f520e2d))
    - Update asciinema video ([`6821adc`](https://github.com/Byron/dua-cli/commit/6821adca0f351411120c0c7f1c2b9f99f03040b8))
    - Bump tui-react version to 0.3 ([`cad0beb`](https://github.com/Byron/dua-cli/commit/cad0beb5cf8735af20e74764eae6b9d120093b22))
    - Minor bump for tui default features = false ([`b42a81d`](https://github.com/Byron/dua-cli/commit/b42a81dba70f272374a6683f0c430c3e1ab5ed5d))
    - Disable default features for tui in tui-react ([`8467a49`](https://github.com/Byron/dua-cli/commit/8467a49796e56a874837dc810dc2e534ec03f0a3))
    - Clippy ([`70b043a`](https://github.com/Byron/dua-cli/commit/70b043abfd4a5765b4966cff65a7b67c518528ef))
</details>

## v2.4.1 (2020-03-30)

Bugfix: Update currently visible entries when scanning.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch ([`f3505ec`](https://github.com/Byron/dua-cli/commit/f3505ec9f67abd9d4ce51c3b91d3d1edc6003ee0))
    - Update currently visible entries whenever we get the chance during scanning ([`8b3a32f`](https://github.com/Byron/dua-cli/commit/8b3a32f9d99a26ac62e150ae6a2cb5fa835a8055))
    - Revert attempt to use tui-react's drawing… ([`fc0b814`](https://github.com/Byron/dua-cli/commit/fc0b814eab5d4157b3c09b34957c8a68e39d46d3))
    - Revert "use tui-react to draw text…" ([`dff2c86`](https://github.com/Byron/dua-cli/commit/dff2c8637198f1b695d3ccf25a49566e55e38249))
    - Cleanup ([`12fd993`](https://github.com/Byron/dua-cli/commit/12fd9936abfce74df3b5e3b005d7eff7e7d8204d))
    - Use tui-react to draw text… ([`e8c00b7`](https://github.com/Byron/dua-cli/commit/e8c00b709fe1d4470d80e086ba615febba0dfd24))
    - Remove roadmap, development is a bit more 'fluid' these days ([`0838d9e`](https://github.com/Byron/dua-cli/commit/0838d9ed97f6be0a5a080170c15605581e0088bb))
</details>

## v2.4.0 (2020-03-29)

Full interaction during scanning phase; add inline-help for better UX.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 29 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump minor version ([`4bea206`](https://github.com/Byron/dua-cli/commit/4bea206639aecd7c28bb399bc93ec9350b5da142))
    - Don't try to shutdown keyinput thread to not lose input events ([`80979a1`](https://github.com/Byron/dua-cli/commit/80979a179f924af87a33fc81ccca055ce6df5636))
    - First step towards support aync/channel based input events ([`e811eff`](https://github.com/Byron/dua-cli/commit/e811effe6424cd691260b07d1187d7c2d34ad4f1))
    - Toggle help for entries and mark pane ([`7689016`](https://github.com/Byron/dua-cli/commit/7689016c537d054a519e4e61c577e30645537213))
    - Navigation help for 'help' pane :D ([`d5ed498`](https://github.com/Byron/dua-cli/commit/d5ed498b592ff2b7f725163cae0c8426930c005c))
    - Auto-help which follows through the panes ([`ac04d9e`](https://github.com/Byron/dua-cli/commit/ac04d9ed9992090cfaf0002c2da954fefd542241))
    - Crossbeam channel is actually not needed in this case ([`a3cf6d6`](https://github.com/Byron/dua-cli/commit/a3cf6d6f3ea68d4cc91a433b4e3701e698f27009))
    - Import plenty of utilities from prodash into tui-react ([`584cc98`](https://github.com/Byron/dua-cli/commit/584cc989cfdf37cd11a2e885e42ddabaccda7dec))
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
    - Fix tests by regenerating them - issue is that sym-links are not shown anymore. ([`6b90258`](https://github.com/Byron/dua-cli/commit/6b90258662810ce740f7f9ad44234e10f3367fc3))
    - Add ArchLinux to README.md ([`a4abfd1`](https://github.com/Byron/dua-cli/commit/a4abfd11f679a479d9668d833cecfee0425bd22f))
    - Merge remote-tracking branch 'origin/master' ([`f5a1ff2`](https://github.com/Byron/dua-cli/commit/f5a1ff2fbb3aeaf6a9afb730a39a8c8abea454c4))
</details>

## v2.3.9 (2020-03-27)

Do not follow symlinks unless it's the only root path to follow.

This brutally fixes an issue where symbolics links are honored when they are placed in the current working directory, as internally `dua` will 
treat each cwd directory entry as individual root path.

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
    - Truly don't follow symlinks unless they are the only top-level path. ([`768cbce`](https://github.com/Byron/dua-cli/commit/768cbce3963be7d6ece448d56289223810d678ac))
    - Update README.md ([`ac2fe84`](https://github.com/Byron/dua-cli/commit/ac2fe840b510c4f15a63135f124fb140db271848))
</details>

## v2.3.8 (2020-03-26)

`dua interactive` (`dua i`) is now about twice as fast due to using all logical cores, not just physical ones.
This is also the first release with github releases: https://github.com/Byron/dua-cli/releases/tag/v2.3.8

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release.
 - 2 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - And don't forget to create a directory for artifacts… ([`2bbbb0b`](https://github.com/Byron/dua-cli/commit/2bbbb0b42371e0701af3b927fee129cd8be5a852))
    - Revert "Azure repository is super instable, and often unavailable making this fail" ([`25ae12d`](https://github.com/Byron/dua-cli/commit/25ae12d5ac0a1dde4709cff6be948ab56fdf00d3))
    - Azure repository is super instable, and often unavailable making this fail ([`e94f97d`](https://github.com/Byron/dua-cli/commit/e94f97d91f1021ef06b307c72ea6f6600cb1d375))
    - Generalize release setup for easier copy-paste ([`ea05566`](https://github.com/Byron/dua-cli/commit/ea05566e9cf0f6248c32f304a5282a5d7a551ef4))
    - Bump patch level ([`65ac16b`](https://github.com/Byron/dua-cli/commit/65ac16b377aa33d1064de2ebfaba51d6f95acb55))
    - Adjust releases for master: run on tags only ([`e843eda`](https://github.com/Byron/dua-cli/commit/e843eda0266950bde0d39c9f1b1b8a08d16d9a44))
    - Github releases! ([`8e8e011`](https://github.com/Byron/dua-cli/commit/8e8e0119441518062cc7612b360eca1beaf7143c))
    - Considerably speed up dua interactive by allowing to use all (logical) cores ([`085ae37`](https://github.com/Byron/dua-cli/commit/085ae37d70bbd4328e046a47bc41c13e669eb562))
    - Fix build instruction ([`b39f773`](https://github.com/Byron/dua-cli/commit/b39f7738d45b2627cddd4e026bde6342a7535ccf))
    - Journey tests still fail, newline issues, ignore for now ([`49f3cb9`](https://github.com/Byron/dua-cli/commit/49f3cb9f161ac6898a0d4ad52501d2159421e68c))
    - Adjust release.yml to hopefully suit dua ([`e3481bd`](https://github.com/Byron/dua-cli/commit/e3481bd3a4775898ca6233486fafaae599c51e6d))
    - Use CHANGELOG instead of a huge section in README ([`4254d39`](https://github.com/Byron/dua-cli/commit/4254d3953654a60102ed2bc6e3e0fd57138038f1))
    - Update journey tests hoping they yield the same results on CI ([`fefc52a`](https://github.com/Byron/dua-cli/commit/fefc52ab97cc19ccd85a9dc46175f4c3b3b1c91d))
    - Now with the actual, unaltered release.yml, previous one was ci.yml ([`c32e65a`](https://github.com/Byron/dua-cli/commit/c32e65a4562f3e3c9ce7b39ebbe4bd54ba31da93))
    - Oriignal release.yml from ripgrep, no alterations ([`17170fb`](https://github.com/Byron/dua-cli/commit/17170fb41c2962a468fde7c97cf863ea3e5a85a2))
    - Create rust.yml ([`64d9524`](https://github.com/Byron/dua-cli/commit/64d95247edbd69bb6bf5dd976d2b43364535c107))
</details>

## v2.3.7 (2020-03-24)

<csr-id-45d1ef31181cd9b430d855a4fe23550ea97e685e/>

Upgrade to filesize 0.2.0 from 0.1.0; update dependency versions

### Other

 - <csr-id-45d1ef31181cd9b430d855a4fe23550ea97e685e/> Update Fedora instructions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 6 calendar days.
 - 8 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Updaet dependencies; bump version ([`7c8e387`](https://github.com/Byron/dua-cli/commit/7c8e3875018fc61b86588212d3812a81546b664e))
    - Update to filesize v0.2 ([`cf902db`](https://github.com/Byron/dua-cli/commit/cf902dbc2cc7b80b2657cf2429db708cc71b6253))
    - Update Fedora instructions ([`45d1ef3`](https://github.com/Byron/dua-cli/commit/45d1ef31181cd9b430d855a4fe23550ea97e685e))
</details>

## v2.3.6 (2020-03-16)

Upgrade to jwalk 0.5 bringing better threading control and no symlink following during traversal

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`aa9e326`](https://github.com/Byron/dua-cli/commit/aa9e326d595ea83c3e22a3972a5f068937c47ba3))
    - Potentially faster release binaries; smaller release binaries ([`4f468f4`](https://github.com/Byron/dua-cli/commit/4f468f4349c245d79f4da90e55649d9551af8da7))
    - Now we are truly single-threaded when threads = 1 ([`b7ed2bb`](https://github.com/Byron/dua-cli/commit/b7ed2bbc957c416e8af08983bba46a4fe2a9553c))
    - Add marker for future improvement : parallel deletion ([`394e261`](https://github.com/Byron/dua-cli/commit/394e2615d5fb2cbde9ddb076f1e4867a4161e05a))
    - Jwalk 0.5 has landed - now we don't follow symlinks during traversal! ([`0d6116e`](https://github.com/Byron/dua-cli/commit/0d6116eea1e741bc8bc1fc6d04536c8242c5aa42))
</details>

## v2.3.5 (2020-03-15)

Fast exit from interactive mode for a responsive exit; dependency updates (except jwalk)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`5b696d4`](https://github.com/Byron/dua-cli/commit/5b696d46bf923f5eb0c7d7b3935e35695dc16318))
    - Revert "Upgrade to jwalk 0.5; stop following symlinks during traversal" ([`d2fda42`](https://github.com/Byron/dua-cli/commit/d2fda42dca410a9319f3f08b24545cbd8b8f1f59))
</details>

## v2.3.4 (2020-03-15)

YANKED - jwalk 0.5.0 wasn't used correctly which led to a performance regression

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Upgrade to jwalk 0.5; stop following symlinks during traversal ([`4990fa4`](https://github.com/Byron/dua-cli/commit/4990fa4202f2b687ee2476efe0a406fdfe23fd96))
    - Minor update: itertools ([`e873656`](https://github.com/Byron/dua-cli/commit/e873656d53d4071f70e73514a96eaa4cbfd23fc4))
    - Updated dependencies, again ([`80b43ca`](https://github.com/Byron/dua-cli/commit/80b43caf3bf46f6afea3deaf1b36f985a7025c19))
    - Remove 32bit apple target, it's now unsupported ([`79cc463`](https://github.com/Byron/dua-cli/commit/79cc46322ff29130ab8b1f0061c805c7780119c3))
    - Bump patch level; update dependencies ([`8241b80`](https://github.com/Byron/dua-cli/commit/8241b808988485e651d8336c812f8d3b5376934d))
    - Adapt journey tests to changed signature ([`b26f8ff`](https://github.com/Byron/dua-cli/commit/b26f8ff07730c6d0ba21cd2db398539a1252bf7a))
</details>

## v2.3.3 (2020-03-14)

YANKED - journey tests failed to changed method signature.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 18 calendar days.
 - 18 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump versio ([`d53fd50`](https://github.com/Byron/dua-cli/commit/d53fd5067daecd6e2e7affec917f594fd4e951c6))
    - Exit the program directly to avoid latency ([`175de56`](https://github.com/Byron/dua-cli/commit/175de56ebe0aff01f7e67de9862d98ba0970feea))
    - Add Fedora installation instructions ([`821a456`](https://github.com/Byron/dua-cli/commit/821a45642036597002db798238dc719849be6f56))
    - Prevent continuous unit tests from triggering themselves ([`832e5cd`](https://github.com/Byron/dua-cli/commit/832e5cd99d2d08b9a504612b6af4aaf007c22f14))
</details>

## v2.3.2 (2020-02-25)

Incude the license file in crate.

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
    - Add license file to TUI-react; include it in dua, update dependencies ([`96ff5ab`](https://github.com/Byron/dua-cli/commit/96ff5ab74a70dd908f5dd218077cd2382e08d9f1))
</details>

## v2.3.1 (2020-02-23)

Include .md files in Crate, update dependencies.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Dependency update, version bump ([`a37e68d`](https://github.com/Byron/dua-cli/commit/a37e68d7cd1c1884a0803bb05e1a333fec259ce3))
    - (cargo-release) start next development iteration 2.3.1-alpha.0 ([`4298271`](https://github.com/Byron/dua-cli/commit/4298271100197a2dec7b6bee296f4395ba7fcdcd))
</details>

## v2.3.0 (2020-02-22)

Show size on disk by default; Dependency Update.

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/37), hard links are now not counted anymore.
The `-l` flag will count hard links as it did before. 

And of course, this has no noticable performance impact.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update readme in preparation for new release ([`2f6bb76`](https://github.com/Byron/dua-cli/commit/2f6bb76452b37b47f1f465d8c09ee72c4ed61f14))
    - Rename 'count-links' to more descriptive 'count-hard-links' ([`db514fe`](https://github.com/Byron/dua-cli/commit/db514fe58c234ad312156814ba6f5ee7b7af0b60))
    - Merge branch 'Freaky-hardlink-tracking' ([`a6a4cf3`](https://github.com/Byron/dua-cli/commit/a6a4cf3705ba764ca0862fd3faaf0f7df31ac28d))
    - Remove short-comings from README, as they are not present anymore ([`93b9e12`](https://github.com/Byron/dua-cli/commit/93b9e12a1de090d1c07968144f6d21061e6de50a))
    - Cargo fmt ([`ba7b071`](https://github.com/Byron/dua-cli/commit/ba7b071af53444cf33ed6a11aae02b34bc26c82b))
    - Add hardlink tracking, and an option to disable it ([`5b52294`](https://github.com/Byron/dua-cli/commit/5b522946adb5bb71dd51068eee5f1136e6403b31))
    - (cargo-release) start next development iteration 2.2.1-alpha.0 ([`0c86b89`](https://github.com/Byron/dua-cli/commit/0c86b894caf99d3bee319c5af6f1dcf754b44011))
</details>

## v2.2.0 (2020-02-22)

Show size on disk by default; Dependency Update.

Thanks to [this PR](https://github.com/Byron/dua-cli/pull/35), the old apparent size can be displayed with the
`-A` flag, and the much more useful 'size on disk' is now shown by default.

To my pleasant surprise, this does not seem to affect performance at all - everything stays speedy.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 20 calendar days.
 - 20 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Dependency update, cut release ([`f2793b9`](https://github.com/Byron/dua-cli/commit/f2793b913b80744b4696024cb5e90e7f4f4f4627))
    - Merge branch 'Freaky-apparent-size' ([`4db48ce`](https://github.com/Byron/dua-cli/commit/4db48ce218f12e11bbf6727fab6fb58c142b1a33))
    - Add support for real/apparent size ([`d86e1e0`](https://github.com/Byron/dua-cli/commit/d86e1e0f66ac8bd031233a6a54e2a1694acf1142))
    - Upgrade tui-react ([`2495390`](https://github.com/Byron/dua-cli/commit/249539045e4dfb813723dff342c52a1ca92184ce))
    - New release of tui-react ([`8aec8c7`](https://github.com/Byron/dua-cli/commit/8aec8c7c9879c0bf29e82b89aab9202e2d117698))
    - Cargo update - will a better lock file fix this issue? ([`c1203ee`](https://github.com/Byron/dua-cli/commit/c1203ee8bede4ad7cd7daaf245d2bfc4ff11cae1))
    - Fix installation instructions ([`e773e33`](https://github.com/Byron/dua-cli/commit/e773e339363e0855474b34c57044872931bd73a0))
    - For now, only run unit-tests on CI ([`8809700`](https://github.com/Byron/dua-cli/commit/8809700d0902888b7ad012c183f9a6229d52a3b8))
</details>

## v2.1.13 (2020-02-01)

Dependency Update; Github Releases.
Binaries for Linux and MacOS are now available on GitHub Releases.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 87 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix script paths; add badge ([`5bd7766`](https://github.com/Byron/dua-cli/commit/5bd77660635eb699a385ebc4fb483e8bb8a9ca22))
    - Add installation note ([`30e7eeb`](https://github.com/Byron/dua-cli/commit/30e7eeb1965694508e8bffae4e3ea47c3cc7118b))
    - Add travis support including releases ([`421072f`](https://github.com/Byron/dua-cli/commit/421072f09738756c1796809accf3d5e1890f807c))
    - Update tui to 0.8 ([`d871bc0`](https://github.com/Byron/dua-cli/commit/d871bc044028edf6e1cdb4cdcb1c59176648c129))
    - Update petgraph ([`4b2e72f`](https://github.com/Byron/dua-cli/commit/4b2e72f0a89b9f0930a894ef9ebf3e4af94464a0))
    - Cargo-update + new Cargo.lock format ([`ecded30`](https://github.com/Byron/dua-cli/commit/ecded309bc695fa6f5596366694371f0e661d8e9))
    - Nicer and leaner makefile ([`673975a`](https://github.com/Byron/dua-cli/commit/673975aba4f24d3cf6bb6f76863273c62bc4121c))
    - Fix version in README ([`0fef32f`](https://github.com/Byron/dua-cli/commit/0fef32fc22a78bad0a4a1062249f2e54a2008e6f))
    - Update all dependencies to latest version ([`543f7f3`](https://github.com/Byron/dua-cli/commit/543f7f3948c26250a8fc6ebf79a49f3ddfa3cb63))
</details>

## v2.1.12 (2019-10-23)

More obvious highlighting of active panel.

Depending on the terminal used, it might not have been obvious which panel was active. This might be
confusing to new and current users.
Now the color of the widget frame is changed to light gray, instead of remaining gray.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version ([`0627932`](https://github.com/Byron/dua-cli/commit/0627932c3c908b1d7ec48e728687a6eac7f291b7))
    - Make sure borders are drawn more priminently on focus ([`70c8d44`](https://github.com/Byron/dua-cli/commit/70c8d44b8ac42170989aa2e892cf44f79b9ab4c2))
</details>

## v2.1.11 (2019-07-26)

Finally fix symlink handling.

`dua` will not follow symbolic links when deleting directories. Thank a ton, @vks!

_Technical Notes_: Handling symbolic links properly is impossible without usage of `symlink_metadata()`.

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
    - Bump version to 2.2.0 ([`d614b47`](https://github.com/Byron/dua-cli/commit/d614b475dcb02690286218accec28c8b6ee5167c))
    - Update dependencies ([`f205cec`](https://github.com/Byron/dua-cli/commit/f205cec7a6415ad85cefd69026c0f236839c9690))
    - Don't follow symlinks when calculating size interactively ([`6b235de`](https://github.com/Byron/dua-cli/commit/6b235de6f43af0f7573275c2b205741f326fd4cf))
    - Don't follow symlinks when deleting files recursively ([`e01f157`](https://github.com/Byron/dua-cli/commit/e01f157d708eb1cf5cdef0daff843eda98c5db76))
</details>

## v2.1.10 (2019-07-25)

Compatibility with light terminals.
 
* the TUI is now usable on light terminals, and highlighting is more consistent. Thank you, @vks!
* Fixes misaligned columns when displaying '100.00%' alongside other rows by displaying `100.0%` instead. Thanks, @vks, for pointing it out.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 3 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Prepare next release ([`4e500be`](https://github.com/Byron/dua-cli/commit/4e500beb8444f6d9fa31ab984551716fb480d7f5))
    - A single decimal slot for percentages; Fixes #26 ([`44aa899`](https://github.com/Byron/dua-cli/commit/44aa8997e3b18214f7177f7c6cc36a25daafbf24))
    - Update README for upcoming release ([`abefc91`](https://github.com/Byron/dua-cli/commit/abefc91fdfe2d7a168dce4b9bda8c9d0cc98e0dd))
    - Run rustfmt; use debug_assert; rename function ([`fa7daf1`](https://github.com/Byron/dua-cli/commit/fa7daf1be9b67d70c3cde64cecdd4a76d2e8082b))
    - Use same colors in mark pane as in entries pane ([`3baf7f3`](https://github.com/Byron/dua-cli/commit/3baf7f31b91c71ba0acb2be886a47ccbd2b295fb))
    - Fix color scheme for light terminals ([`977e69f`](https://github.com/Byron/dua-cli/commit/977e69f9aafc54f9b2ed9ddb2eee5164e30b213c))
    - Forbid unsafe everywhere ([`f4028ba`](https://github.com/Byron/dua-cli/commit/f4028baf655e2994459e55d62435de4456fee80f))
</details>

## v2.1.9 (2019-07-21)

Improved handling of broken symlinks.

* during symlink deletion, now broken symlinks will be deleted as expected.
* always return to the previous terminal screen so the TUI doesn't stick to the current one.
* display broken symlinks on the first level of iteration.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version ([`387cc1f`](https://github.com/Byron/dua-cli/commit/387cc1f86e5aec8a20a25ea71f74e948b110d2c6))
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

 - 2 commits contributed to the release.
 - 10 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch level ([`22c7eb5`](https://github.com/Byron/dua-cli/commit/22c7eb5e34372b2883276bb7fc207df891f7df8e))
    - Do not follow symbolic links when iterating directories! ([`560a76d`](https://github.com/Byron/dua-cli/commit/560a76d43fa44c4ebf9bdc51087647bb800bbe68))
</details>

## v2.1.7 (2019-07-03)

Use latest version of open-rs.

That way, pressing `shift + O` to open the currently selected file won't possibly spam the terminal
with messages caused by the program used to find the system program to open the file.

Fixes [#14](https://github.com/Byron/dua-cli/issues/14)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix Cargo.lock... again. 2.1.7 is 2.1.6 effectively ([`dd12ca7`](https://github.com/Byron/dua-cli/commit/dd12ca765b7c7726e718b64035dedd0c9b3d50a0))
</details>

## v2.1.6 (2019-07-03)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump patch; fixes #14 ([`473ac20`](https://github.com/Byron/dua-cli/commit/473ac20f5a03e95ed5fe02ced97231806282c09c))
</details>

## v2.1.5 (2019-07-03)

- re-release with Cargo.lock

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix inconsistent cargo.lock file; update all deps ([`03628c8`](https://github.com/Byron/dua-cli/commit/03628c86778c29ee27e78608401766fe92a7c683))
</details>

## v2.1.4 (2019-07-02)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 15 calendar days.
 - 15 days passed between releases.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Prep for re-release ([`22947b7`](https://github.com/Byron/dua-cli/commit/22947b76ed438ca0282f8d8bf4edc54096f43df7))
    - Add `Cargo.lock` because this is a binary ([`ebc9c6b`](https://github.com/Byron/dua-cli/commit/ebc9c6b4cebc4ced23707e0d6aab4b5fa70511fc))
    - Add install instructions for voidlinux ([`d039285`](https://github.com/Byron/dua-cli/commit/d0392854ce811b559e4acaf0ea654c1922e9cd6a))
    - Additional limitations related to symlinks and hardlinkes ([`532457e`](https://github.com/Byron/dua-cli/commit/532457e58b2b15439558bbf5bc2062c94d9bcdf7))
</details>

## v2.1.3 (2019-06-16)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Patch release to get a working github release - no changes to code ([`fc9f3a1`](https://github.com/Byron/dua-cli/commit/fc9f3a167622fe5fd0ea2c9a9eb0c2630d6fd244))
    - Make filename smaller; related to #10 ([`868499e`](https://github.com/Byron/dua-cli/commit/868499e0d5459ddc1b9dfb6edfa6cf41948b93a5))
    - Inform about the dark-mode limitation ([`bb2162c`](https://github.com/Byron/dua-cli/commit/bb2162cc3e6fd189592028246acc48610c93f1c1))
</details>

## v2.1.2 (2019-06-16)

Bug fixes and improvements.

* Performance fix when showing folders with large amounts of files
* Display of amount of entries per directory

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

* Better information about deletion progress
* removal of windows support

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Reopen #2; removal of windows support ([`81dc53b`](https://github.com/Byron/dua-cli/commit/81dc53b0e6d7c292909610fba6fd030ed6b01917))
    - Better progress display when deleting multiple items ([`d586703`](https://github.com/Byron/dua-cli/commit/d5867038aa8d1d216c146fe8d0a919352dce4855))
</details>

## v2.1.0 (2019-06-16)

Bug fixes and improvements.

* windows support (never actually worked), usage of crossterm is difficult thanks to completely
  different input handling.
* additional key-bindings
* auto-restore previous selection in each visited directory

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version to 2.1.0 ([`a8f595f`](https://github.com/Byron/dua-cli/commit/a8f595f576f164fd13e59230370b310119599f43))
    - Fix tests... really need CI if PRs keep coming ([`6578aa8`](https://github.com/Byron/dua-cli/commit/6578aa8ded3089e09f731115777413824dbc7f74))
    - Auto-restore previously selected entries; quality of life! ([`52f40ca`](https://github.com/Byron/dua-cli/commit/52f40caf557c4dfdae169b39984dd6fda1f77474))
    - Add 'h' and 'l' as alternative keybindings ([`251ea53`](https://github.com/Byron/dua-cli/commit/251ea53bbd5072a7e7315c610cbb59540f93c7a9))
    - Fixes #2 - use crossterm instead of Termion ([`34274b1`](https://github.com/Byron/dua-cli/commit/34274b108957e8819395d4bc38a9456be5372a2a))
    - One more limitation ([`b68900b`](https://github.com/Byron/dua-cli/commit/b68900b0d20ef5cf5b6302a5165a7ba0f9653540))
</details>

## v2.0.1 (2019-06-16)

Bug fixes and improvements.

* fix typo in title 
* better display of IO-Errors in aggregate mode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix typo in title ([`9526241`](https://github.com/Byron/dua-cli/commit/952624118bf3c293f23064e21828af00df9d132c))
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

 - <csr-id-c67abaec3c573dbfaf31be22693220a49a67b262/> first test to fully verify deletion
 - <csr-id-a128eb4a6e675f148a203ac66de075ee0c0def1c/> Move parts of the tests into their own files
 - <csr-id-ef8cf5636f782024372f044af80f06ed030168b0/> recursive deletion - tests can begin
 - <csr-id-dacb897405c06f9468faa860e27f47d1d0e548bb/> simple recursive copy - deletion would like depth-first though ;)
 - <csr-id-51ce1ed159d59c6e221af4df9a3f7da41b1820cb/> Basic for test with writable directory
   Would have loved to use a crate with basic utilities, but there is
   no internet here :(
 - <csr-id-6cbd4866b18de91d3702a55c45650615d67f5f30/> Make marker selection feel right
 - <csr-id-7ad2130bada27098e2d24f06650873a53b159f87/> Nicer colors for warn window in selection
 - <csr-id-49edb7654ce3380bcde28630645af3740cf1a07a/> Warning window follows user selection
 - <csr-id-984bf4fcce05cd5d495511123c2c3b6906b96f6d/> Fix handling of deleting the first index in the mark list
 - <csr-id-b4a2e0ee8f267ee50f92433e826fa9e42ff618db/> more prominent selection in mark pane
 - <csr-id-b4669c0214a1bc858cf437a65583af7e4b9ec277/> Rustic way of handling the mark panes disappearance

 - <csr-id-fcde45752a9b86ed606b78f522f6b6dd0de25457/> don't show warning if nothing is marked anymore
   this can happen if the user removes all entries. The pane stays open
   in this case, which is a little inconsistent, but not worth fixing
   as it's certainly not the common case.
   
   If it should be fixed, the 'key()' function should become consuming
   to possible delete the pane.
 - <csr-id-01dd8e284224e42b59f317cd922d388f23def829/> Actually hook up spacebar in mark pane
 - <csr-id-d42573e63a120c8c5a253b7be52f9c68fb72274b/> Make help window pretty again
 - <csr-id-c0aa567e81b54913df464c9b500fe7a20ada0ea5/> Better handling of what is selected after removing a marked entry
 - <csr-id-f9a9cdf9f827a5e08b1bcc6035f908fdb971c9fd/> Don't try to go down as marked items are removed

### Other Features

 * Single Unit Mode, see [reddit](https://www.reddit.com/r/rust/comments/bvjtan/introducing_dua_a_parallel_du_for_humans/epsroxg/)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 234 commits contributed to the release.
 - 16 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Improve readme ([`5a6571e`](https://github.com/Byron/dua-cli/commit/5a6571e6563411b9803be31b292a13bc6ca62b58))
    - Update to the latest asciinema recording ([`2748500`](https://github.com/Byron/dua-cli/commit/2748500e395c4845488d332a83b4c5eeec1c64cb))
    - Handle symlinks in a rather brutal way. ([`209eecf`](https://github.com/Byron/dua-cli/commit/209eecf042761eba35be809ca22bc98af472acad))
    - Fix journey-tests ([`854dc46`](https://github.com/Byron/dua-cli/commit/854dc46e1d99ce5c089369820351b9354707a300))
    - Prepare 2.0 release ([`d18db06`](https://github.com/Byron/dua-cli/commit/d18db061b3da35e98eaf7d9f642a84c7df74233f))
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
    - Mention a limitation I chose to forego ([`88ec5d5`](https://github.com/Byron/dua-cli/commit/88ec5d51980533a4942cf18fb60f525924dfb2bd))
    - Add more unicode samples, along with a new limitations ([`f1cc234`](https://github.com/Byron/dua-cli/commit/f1cc234c3aa77f97e2b9281beed61ddb6b6e170b))
    - Add difficult graphemes from... ([`07727c6`](https://github.com/Byron/dua-cli/commit/07727c6abd83d2f58cccf92d7cf85eebb96a1524))
    - Add grapheme  ladden files ([`3e8dad3`](https://github.com/Byron/dua-cli/commit/3e8dad38085c060d6bfbf298a989739a9f9159ab))
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
    - Remove obsolete annotations ([`982446a`](https://github.com/Byron/dua-cli/commit/982446ad0ef9a475274c9a0f05a32147fcafd061))
    - Version bump ([`64e4068`](https://github.com/Byron/dua-cli/commit/64e4068308c9f314fdc881b40c218a5b41c7686b))
    - More hotkeys ([`eec9803`](https://github.com/Byron/dua-cli/commit/eec980374f7ada8c002d7f8d1663307552f801ab))
    - Fix sorting; add some alternate keys ([`f2e4504`](https://github.com/Byron/dua-cli/commit/f2e45047015ec2c08777513a366db92af0ae3586))
    - Clear screen at initialization ([`37ce7fe`](https://github.com/Byron/dua-cli/commit/37ce7fe923ad76e9c6b24a462b3cb258eef88607))
    - Refactor ([`c33ae7c`](https://github.com/Byron/dua-cli/commit/c33ae7c7d9f538490346a8532e27c3dd6c4aa21d))
    - Bump version ([`f512974`](https://github.com/Byron/dua-cli/commit/f512974d55577265f40dbf58053203a4b12152ad))
    - Assure we see something while scanning - entries are now manually provided ([`2c1cb19`](https://github.com/Byron/dua-cli/commit/2c1cb19aeb89d25977bd9fa76b8572d7e7d942a7))
    - Adjust release notes ([`9e6f62e`](https://github.com/Byron/dua-cli/commit/9e6f62e32259aa9be67402980b38f3c6133efa19))
    - The block is now not needed anymore - we can just own simple props ([`42fb0cc`](https://github.com/Byron/dua-cli/commit/42fb0cccb10ce1084267b63b07a5a0a8bf84de99))
    - Updated readmes ([`f59b32d`](https://github.com/Byron/dua-cli/commit/f59b32d344875bbfc584f259c2c2e74dbb254b08))
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
    - Add caveats of tui-react - they seem to be grave!! ([`bdec24f`](https://github.com/Byron/dua-cli/commit/bdec24f2d708baddd9602c3b9c841419425062c9))
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
    - Fix tests - column width changes ([`c7ee6b5`](https://github.com/Byron/dua-cli/commit/c7ee6b53b49a8c9489aa07bd7d262ec1d2b76349))
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
    - Another limitation in readme ([`cab0ec2`](https://github.com/Byron/dua-cli/commit/cab0ec257356aea1cfc947cfed35b6ee6b9b8024))
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
    - Add 'make test' target, fix unit tests ([`2338e75`](https://github.com/Byron/dua-cli/commit/2338e751c40284fe49643767dd33be3230f63440))
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
    - Ignore ds-store ([`1ff799e`](https://github.com/Byron/dua-cli/commit/1ff799e725c5cbdea33f952495211708482e2b73))
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
    - Fix unit tests ([`bc80db2`](https://github.com/Byron/dua-cli/commit/bc80db2b3f026cc10f9a06f0db624d32c1bd807f))
    - Improve title display, deal with relative paths ([`5b4d44c`](https://github.com/Byron/dua-cli/commit/5b4d44c0121db981a61a838db18a5e6ccf4660bf))
    - Better title for entries, based on the paths your are in ([`74870ba`](https://github.com/Byron/dua-cli/commit/74870bae69ed9bfe34e75ef82e3d76bc6f98e160))
    - Move 'traverse' module out of 'interactive' - it's unrelated ([`fb57ebd`](https://github.com/Byron/dua-cli/commit/fb57ebd0423775c4c9b757a2fad588f8baa5beec))
    - Add 'u' key to go up one level ([`84b6f8c`](https://github.com/Byron/dua-cli/commit/84b6f8ce829e7a57604b4e983c91bc52a7299ac4))
    - Show directories very similar to ncdu ([`74e5116`](https://github.com/Byron/dua-cli/commit/74e511631a7f05143e487584a4325fe65c774ba5))
    - Add 'o' navigation ([`25ceae2`](https://github.com/Byron/dua-cli/commit/25ceae2779e3e20b4ff4ac3d6149410e5f851775))
    - Add 'k' navigation key ([`748dfc3`](https://github.com/Byron/dua-cli/commit/748dfc353a7d8c7bbb6bbfb097bacec18b80e32a))
    - Add 'j' key functionality for basic navigation ([`a76ad50`](https://github.com/Byron/dua-cli/commit/a76ad5009ac9177e1efb37130d1dcedb5df1e5de))
    - Make working with nodes less cumbersome in unit tests ([`1cfb627`](https://github.com/Byron/dua-cli/commit/1cfb62722d25ee17109fd0073e3cd0ac6a022ffa))
    - Compute percentage (at all), non-graphical for now ([`df0fe62`](https://github.com/Byron/dua-cli/commit/df0fe6279065ba060803e236a73336bdcf8fe4dd))
    - Preliminary styling for selected entries ([`90f94f7`](https://github.com/Byron/dua-cli/commit/90f94f79ac54689c4af47ad31e1080da725cd7ed))
    - Unify sorting to start dealing with selections ([`0b3e158`](https://github.com/Byron/dua-cli/commit/0b3e158085d68ba43dc3ac034ce4f0b5df9d61e8))
    - Smaller release binaries! ([`b3dc836`](https://github.com/Byron/dua-cli/commit/b3dc836baa00e36c56f823e9e5b3e9118fdd8b30))
    - Test for handling the root correctly in interactive mode ([`59a3001`](https://github.com/Byron/dua-cli/commit/59a3001012d5ff40d050a1abfc370aaa248d8f66))
    - The first test for user input, yeah! ([`05c8ec1`](https://github.com/Byron/dua-cli/commit/05c8ec1a6e2ce9af3f55d75cb761cf3b66244bb8))
    - Prepare for mutable state in application, even more :D ([`11147d8`](https://github.com/Byron/dua-cli/commit/11147d8e344435b95adaca68e5125836c0bf2ed9))
    - Prepare for handling mutable application state ([`e48898b`](https://github.com/Byron/dua-cli/commit/e48898ba98312be9e77b2d5cc8a64a127ac59688))
    - Sorting by size, descending, for entries ([`e8cb9dc`](https://github.com/Byron/dua-cli/commit/e8cb9dcda01d5dc073dfb8093f66bd13d5699105))
    - Don't display '0' for total bytes while traversing ([`9720931`](https://github.com/Byron/dua-cli/commit/9720931800fd8e189c99cbf0cb01a31f23663744))
    - Assure root size is properly computed ([`dcf3a26`](https://github.com/Byron/dua-cli/commit/dcf3a2651b79493964feb16d8a2148e851a7b4ca))
    - Refactor ([`1f482aa`](https://github.com/Byron/dua-cli/commit/1f482aab49a9094234d422b3599858e909c3f164))
    - Document reasoning for using termion everywhere ([`0cc49f5`](https://github.com/Byron/dua-cli/commit/0cc49f5cbbd383f57a2f628711cabf36a38de2c0))
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
    - Tree building works - next: sizes ([`5a7ee1b`](https://github.com/Byron/dua-cli/commit/5a7ee1bf881518b6cd33a1880fabd12aa53553bf))
    - One step closer to the actual tree-building implementation ([`7c3743d`](https://github.com/Byron/dua-cli/commit/7c3743d601cce407024e65570d108867a6196893))
    - First failing attempt to build a graph on demand ([`0774ecc`](https://github.com/Byron/dua-cli/commit/0774eccb72abfd800880cbc8490cb9899f1ab140))
    - First failing test - even though just a guess :D ([`68569c6`](https://github.com/Byron/dua-cli/commit/68569c69f5fdeedddd45635e8eb6d0c255de53f4))
    - Some more inspiration ([`396ab0b`](https://github.com/Byron/dua-cli/commit/396ab0b5adbb0a29c6b4db77b30893978752329e))
    - First infrastructure for unit-level tests ([`1c53865`](https://github.com/Byron/dua-cli/commit/1c538654fba3caf7f7d601d6bf8a4af24faf19c8))
    - Basic frame to support new interactive mode ([`6d82a72`](https://github.com/Byron/dua-cli/commit/6d82a724b0452e417e20cbe8a02e3bed647e9674))
    - Highlight files with a different color ([`495ccbd`](https://github.com/Byron/dua-cli/commit/495ccbda25cb27dc912c07fbdb29651b83f32c68))
</details>

## 1.2.0

The first usable, read-only interactive terminal user interface.
That's that. We also use `tui-react`, something that makes it much more pleasant to handle the
application and GUI state.

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

 - 27 commits contributed to the release over the course of 3 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Add description to Cargo.toml ([`a53c2ac`](https://github.com/Byron/dua-cli/commit/a53c2acb65457df740f3605124b9e42d363897de))
    - Better readme ([`e8a83e7`](https://github.com/Byron/dua-cli/commit/e8a83e779f694a8ba2a264a5def7add6d65b191c))
    - Add asciicast ([`a66cf95`](https://github.com/Byron/dua-cli/commit/a66cf95bf57f477eae7a8ef307fd62a4df0da76d))
    - Now with colored help ([`3798be8`](https://github.com/Byron/dua-cli/commit/3798be8a31902a74f4c0280d0d1def8d6bb74d10))
    - Prepare for release ([`28079ec`](https://github.com/Byron/dua-cli/commit/28079ec7d976aef0eacd88e0090f05ad87219558))
    - Create LICENSE ([`0678400`](https://github.com/Byron/dua-cli/commit/06784008779ceace1fabd55f271996e406f6502b))
    - Udpate readme ([`0ae156e`](https://github.com/Byron/dua-cli/commit/0ae156e5a1b6e3f7be2c61cba2a882d8a8a933c4))
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
    - Mission statement and first tasks, to get started ([`247a3b9`](https://github.com/Byron/dua-cli/commit/247a3b9dc851237288fd243a9029afcec6453e5d))
    - First instantiation of template ([`e9a4472`](https://github.com/Byron/dua-cli/commit/e9a447250ba9ffd10f94f6f7d970c6da141c185d))
</details>

## v0.14.0 (2021-01-04)

## v0.13.0 (2020-11-15)

## v0.12.0 (2020-09-28)

## v0.10.1 (2020-07-22)

## v0.10.0 (2020-07-22)

## v0.4.1 (2020-07-10)

## v0.3.0 (2020-04-03)

## v0.2.2 (2020-03-29)

## v0.0.1 (2020-10-26)

