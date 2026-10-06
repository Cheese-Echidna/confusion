//! Associate evaluation failures, broken references and blocked nodes with repair actions.
//!
//! Planned public API (not implemented): EvaluationDiagnostic, BlockedDependency, RepairSuggestion.
//!
//! Connections: model/naming, kernel/validation, ui/diagnostics.
//!
//! Invariant: Expose stale geometry clearly; exports require a successful evaluation of the
//! requested revision.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
