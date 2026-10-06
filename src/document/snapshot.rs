//! Create immutable Send + Sync snapshots for solving, evaluation and export.
//!
//! Planned public API (not implemented): DocumentSnapshot, SnapshotBuilder.
//!
//! Connections: document/store, runtime/jobs, evaluation/engine.
//!
//! Invariant: Contain only domain data and revision stamps; no GPUI entities or native shape
//! pointers.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
