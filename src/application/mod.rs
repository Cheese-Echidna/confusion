//! Compose services and own window/document sessions; this is the only layer that coordinates every subsystem.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod bootstrap;
pub mod events;
pub mod modes;
pub mod services;
pub mod session;
pub mod workspace;
