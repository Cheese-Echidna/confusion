//! Define the two primary modes and transition rules between model and sketch editing.
//!
//! Planned public API (not implemented): ApplicationMode, ModeTransition, SketchEditContext.
//!
//! Connections: tools/state, settings/keymap, ui/toolbar.
//!
//! Invariant: Components, assemblies, drawings and geometric inspection are contextual model-mode
//! workbenches rather than additional primary modes.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
