//! Maintain typed selection sets, filters, occurrence paths and selection cycling.
//!
//! Planned public API (not implemented): SelectionModel, SelectionItem, SelectionFilter.
//!
//! Connections: tools/state, ui/browser, interaction/picking.
//!
//! Invariant: Selection is session state; topology selections use semantic references when
//! committed to features.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
