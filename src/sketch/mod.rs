//! Define sketch intent and topology independently of solver implementation and UI gestures.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod constraints;
pub mod diagnostics;
pub mod dimensions;
pub mod edit;
pub mod entities;
pub mod inference;
pub mod profiles;
pub mod projection;
pub mod workplane;
