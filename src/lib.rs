//! Public library API for `dua` core traversal and aggregation functionality.
//!
//! This crate powers the `dua` binary and can also be used as a library.
#![deny(unsafe_code)]
#![deny(missing_docs)]

pub(crate) use dua_core as walk;
pub use dua_core::Options as TraversalOptions;

mod aggregate;
mod common;
mod config;
pub use config::{Config, KeyBindings, KeysConfig};
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
