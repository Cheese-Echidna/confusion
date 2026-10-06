//! Provide technical drawing workbenches in model mode; persist associative drawing intent, derive geometry and output.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod annotations;
pub mod document;
pub mod export;
pub mod tables;
pub mod views;
