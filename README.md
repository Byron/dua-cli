**dua** (-> _Disk Usage Analyzer_) is a tool to conveniently learn about the usage of disk space of a given directory. It's parallel by default and will max out your SSD, providing relevant information as fast as possible.

[![asciicast](https://asciinema.org/a/au3neIHDGtYYj4blyTXR8VkJz.svg)](https://asciinema.org/a/au3neIHDGtYYj4blyTXR8VkJz)

### Installation

Via `cargo`, which can be obtained using [rustup][rustup]

```
cargo install dua-cli
```

### Usage

```bash
# count the space used in the current working directory
dua
# count the space used in all directories that are not hidden
dua *
# learn about additional functionality
dua aggregate --help
```

### Roadmap

#### 🚧v2.0 (_wip_) - interactive visualization of directory sizes with an option to queue their deletion

A sub-command bringing up a terminal user interface to allow drilling into directories, and clearing them out, all just using the keyboard.

##### Other Features

 * [ ] Single Unit Mode, see [reddit](https://www.reddit.com/r/rust/comments/bvjtan/introducing_dua_a_parallel_du_for_humans/epsroxg/)
 * [ ] Evaluate unit coloring

#### ✅v1.0 (_released_) - aggregate directories, fast

Simple CLI to list top-level directories similar to sn-sort, but faster and more tailored to getting an idea of where most space is used.

### Development

#### Run tests

```bash
make journey-tests
```

#### Learn about other targets

```
make
```

### Acknowledgements

Thanks to [jwalk][jwalk], all there was left to do is to write a command-line interface. As `jwalk` matures, **dua** should benefit instantly.

### Tradeoffs

* Dedication to `termion`
  * we use [`termion`][termion] exlusively, and even though [`tui`][tui] supports multiple backends, we only support its termion backend. _Reason_: `tui` is only used for parts of the program, and in all other parts `termion` is used for coloring the output. Thus we wouldn't support changing to a different backend anyway unless everything is done with TUI, which is really not what it is made for.


[rustup]: https://rustup.rs/
[jwalk]: https://crates.io/crates/jwalk
[termion]: https://crates.io/crates/termion
[tui]: https://github.com/fdehau/tui-rs
