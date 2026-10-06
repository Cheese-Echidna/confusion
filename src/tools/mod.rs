//! Own modal CAD tool state machines: activation, picking, numeric entry, preview, commit and cancel.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod assembly;
pub mod constraints;
pub mod model;
pub mod sketch;
pub mod state;
