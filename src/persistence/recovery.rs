//! Write autosave snapshots and recover unsaved documents after crashes.
//!
//! Planned public API (not implemented): RecoveryManager, RecoverySnapshot, RecoveryCandidate.
//!
//! Connections: application/session, persistence/writer, platform/paths.
//!
//! Invariant: Recovery files are full current-state snapshots, not event journals; settings remain
//! in the one JSON settings file.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
