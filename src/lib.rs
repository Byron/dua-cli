//! Public library API for `dua` core traversal and aggregation functionality.
//!
//! This crate powers the `dua` binary and can also be used as a library.
#![cfg_attr(windows, feature(windows_by_handle))]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod aggregate;
mod common;
mod config;
pub use config::Config;
mod crossdev;
mod inodefilter;

/// Filesystem traversal, in-memory tree representation, and traversal events.
pub mod traverse;

pub use aggregate::aggregate;
pub use common::*;
pub(crate) use inodefilter::InodeFilter;
