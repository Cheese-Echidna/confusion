//! Confusion architecture scaffold: declarative parametric CAD with sketch/model modes.
//!
//! Planned public API: headless document, evaluation, solver and exchange services;
//! application composition and GPUI views live in separate upper-layer modules.
//!
//! Connections: main.rs will call application::bootstrap after runtime implementation.
//! All child APIs named in file comments are proposed contracts, not implemented types.
//! See docs/architecture.md for dependency direction and docs/file-map.md for each file.
//!
//! Canonical .con state excludes edit history. Domain code uses f64 geometry; GPU
//! rendering uses derived scene data. Native unsafe code belongs only in kernel/bridge
//! and platform/gpu_surface after auditing ownership and synchronization.

pub mod application;
pub mod assembly;
pub mod commands;
pub mod document;
pub mod drawing;
pub mod evaluation;
pub mod exchange;
pub mod foundation;
pub mod interaction;
pub mod kernel;
pub mod materials;
pub mod model;
pub mod parameters;
pub mod persistence;
pub mod platform;
pub mod render;
pub mod runtime;
pub mod settings;
pub mod sketch;
pub mod solver;
pub mod tools;
pub mod ui;
