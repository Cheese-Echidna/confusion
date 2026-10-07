//! Build a Fusion-inspired GPUI shell with Zed-style panels and command search; views dispatch intent and display immutable results.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod accessibility;
pub mod actions;
pub mod analysis;
pub mod browser;
pub mod command_palette;
pub mod components;
pub mod diagnostics;
pub mod dialogs;
pub mod inspector;
pub mod parameter_table;
pub mod settings;
pub mod status_bar;
pub mod theme;
pub mod toolbar;
pub mod viewport;
pub mod workspace;

pub mod text_input;

pub mod assets;

pub mod sketch_canvas;
pub mod view_cube;
