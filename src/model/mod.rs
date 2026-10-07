//! Own declarative modeling features and stable output identity; editing a feature replaces intent rather than appending history.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod analysis;
pub mod construction;
pub mod direct;
pub mod feature;
pub mod measure;
pub mod naming;
pub mod operations;
pub mod patterns;
pub mod registry;

pub mod modify;

pub mod solid_create;
