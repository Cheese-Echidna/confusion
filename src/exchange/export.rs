//! Dispatch exports against a successfully evaluated immutable revision with progress and atomic
//! writes.
//!
//! Planned public API (not implemented): ExportService, ExportRequest, ExportResult, ExportFormat.
//!
//! Connections: evaluation/engine, persistence/atomic, exchange/step, exchange/stl.
//!
//! Invariant: Reject stale or failed geometry unless an explicit partial-export choice is made.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
