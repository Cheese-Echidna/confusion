//! Coordinate parameters, solving and exact feature evaluation over snapshots, emitting immutable derived results.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod cache;
pub mod diagnostics;
pub mod engine;
pub mod invalidation;
pub mod preview;

pub mod solid;
