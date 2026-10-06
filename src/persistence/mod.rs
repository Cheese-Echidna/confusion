//! Serialize only current declarative state and required embedded assets in a versioned .con container.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod assets;
pub mod atomic;
pub mod container;
pub mod migration;
pub mod reader;
pub mod recovery;
pub mod schema;
pub mod writer;
