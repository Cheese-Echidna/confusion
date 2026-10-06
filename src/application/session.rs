//! Own one document store, undo stack, selection, mode, active tool, jobs and last-good evaluation.
//!
//! Planned public API (not implemented): DocumentSession, SessionState, SessionEvent.
//!
//! Connections: document/store, commands/undo, tools/state, ui/workspace.
//!
//! Invariant: Accept only current results; distinguish committed, preview, pending and stale views
//! explicitly.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
