//! Define the shared tool lifecycle and manage active tool, nested numeric entry and preview
//! tokens.
//!
//! Planned public API (not implemented): Tool trait, ToolController, ToolState, ToolEvent,
//! ToolResponse.
//!
//! Connections: commands/dispatch, interaction/selection, evaluation/preview.
//!
//! Invariant: Tool states are transient; cancel restores the committed document and releases
//! temporary selections.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
