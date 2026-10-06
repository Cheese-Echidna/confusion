//! Own all user-configurable application settings in one versioned JSON file, including context-dependent keybindings.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod fusion_defaults;
pub mod keymap;
pub mod migration;
pub mod schema;
pub mod store;
