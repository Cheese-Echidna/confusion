//! Translate input into navigation, selection and tool intentions without directly changing model state.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod gizmos;
pub mod navigation;
pub mod picking;
pub mod selection;
pub mod snapping;
