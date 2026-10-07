//! Provide import/export adapters and reports; STEP/STL outputs are geometry interchange rather than parametric .con models.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod dxf;
pub mod export;
pub mod import;
pub mod step;
pub mod stl;
pub mod svg;
pub mod three_mf;
