//! Model reusable components, occurrences and joint constraints within model mode, independent of UI.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod bom;
pub mod components;
pub mod contact;
pub mod diagnostics;
pub mod joints;
pub mod kinematics;
