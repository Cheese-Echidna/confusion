//! Isolate native paths, dialogs, filesystem semantics, GPU presentation and lifecycle from domain code.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod dialogs;
pub mod files;
pub mod gpu_surface;
pub mod lifecycle;
pub mod paths;
