//! Map solver rank, residuals, conflicts and free directions onto sketch entities for actionable
//! feedback.
//!
//! Planned public API (not implemented): SketchStatus, SketchDiagnostic, ConstraintConflict.
//!
//! Connections: solver/diagnostics, ui/diagnostics, render/overlays.
//!
//! Invariant: Differentiate underconstrained, redundant, inconsistent, degenerate and
//! failed-to-converge states.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
