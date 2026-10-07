//! Own the exact-geometry boundary and all OCCT FFI; expose domain values or opaque local handles, never raw native pointers.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod api;
pub mod bridge;
pub mod curves;
pub mod occt;
pub mod ownership;
pub mod queries;
pub mod session;
pub mod tessellation;
pub mod validation;

#[cfg(feature = "kernel")]
pub mod cancellation;
