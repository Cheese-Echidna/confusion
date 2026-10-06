//! Kernel-independent vocabulary used by every domain; never import GPUI, FFI, persistence or application services.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod errors;
pub mod ids;
pub mod math;
pub mod revision;
pub mod tolerance;
pub mod units;
