//! Serialize deterministic current-state snapshots and referenced assets to an atomic .con save.
//!
//! Planned public API (not implemented): ConWriter, SaveOptions, SaveResult.
//!
//! Connections: document/snapshot, persistence/atomic, persistence/assets.
//!
//! Invariant: Exclude undo/redo, command logs, edit timestamps, kernel caches and render meshes;
//! imported source geometry is required data.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
