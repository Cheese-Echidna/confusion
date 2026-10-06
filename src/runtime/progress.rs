//! Aggregate worker progress into throttled, revision-stamped UI events.
//!
//! Planned public API (not implemented): ProgressEvent, JobProgress, ProgressSink trait.
//!
//! Connections: runtime/jobs, application/events, ui/status_bar.
//!
//! Invariant: Do not emit an event per residual or triangle; unknown total work is supported.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
