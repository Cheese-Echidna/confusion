//! Own canonical current design state and atomic mutations; no UI, kernel handles or runtime caches in persistent types.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod dependency;
pub mod references;
pub mod schema;
pub mod snapshot;
pub mod store;
pub mod transaction;
pub mod validation;

pub mod model;

pub mod dirty;
