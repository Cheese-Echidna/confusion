//! Check B-rep validity, orientation, closure, self-intersections and tolerance growth; implement
//! bounded healing.
//!
//! Planned public API (not implemented): ShapeValidation, HealingOptions, HealingReport.
//!
//! Connections: evaluation/diagnostics, exchange/import, kernel/occt.
//!
//! Invariant: Healing must report modified topology and provenance; no silent destructive repair.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
