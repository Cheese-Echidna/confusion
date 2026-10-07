//! Implement a wgpu 3D renderer consuming immutable scenes; GPUI owns UI layout and the window compositor.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod camera;
pub mod device;
pub mod gpui_bridge;
pub mod materials;
pub mod overlays;
pub mod passes;
pub mod picking;
pub mod renderer;
pub mod scene;

pub mod grid;
