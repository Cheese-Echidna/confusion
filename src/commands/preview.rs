//! Coordinate command argument previews, cancellation and final acceptance.
//!
//! Planned public API (not implemented): CommandPreview, PreviewController, PreviewCommit.
//!
//! Connections: evaluation/preview, tools/state, document/transaction.
//!
//! Invariant: Commit only current valid preview parameters and discard stale jobs on cancel.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
