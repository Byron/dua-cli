//! Core traversal and aggregation functionality used by the `dua` binary.
//!
//! **Unstable:** The entire Rust library API may change in minor releases.
//!
//! Library consumers should use a tilde requirement, such as `~3.0.0`, to receive
//! compatible patch updates. The `dua` command-line interface follows semantic versioning.
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub(crate) use dua_core as walk;
pub use dua_core::Options as TraversalOptions;

mod aggregate;
/// Discovery of directories suitable for interactive cleanup.
pub mod clean;
mod common;
mod config;
pub use config::{Config, KeyBinding, KeyBindings, KeysConfig};
mod crossdev;
mod diff;
pub use diff::diff_snapshots;
mod inodefilter;
/// Reading and writing dua traversal snapshots.
pub mod snapshot;
mod tree;

mod stacks;
/// Filesystem traversal, in-memory tree representation, and traversal events.
pub mod traverse;

#[cfg(any(windows, target_os = "macos"))]
pub use aggregate::aggregate_entries;
pub use aggregate::{aggregate, aggregate_replay, aggregate_snapshot};
pub use common::*;
pub(crate) use inodefilter::InodeFilter;
pub use stacks::{stacks, stacks_from_replay, stacks_from_traversal};
pub use tree::{aggregate_tree, aggregate_tree_from_replay, aggregate_tree_from_traversal};
