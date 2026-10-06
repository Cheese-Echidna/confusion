//! Own named values and safe dimensional expression evaluation; depend only on foundation and generic libraries.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod configurations;
pub mod evaluate;
pub mod expressions;
pub mod table;
