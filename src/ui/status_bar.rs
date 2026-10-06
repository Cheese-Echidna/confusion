//! Display mode, solver status/DOF, units, snap hints, selection information and job progress.
//!
//! Planned public API (not implemented): StatusBar, StatusModel.
//!
//! Connections: application/session, runtime/progress, sketch/diagnostics.
//!
//! Invariant: Throttle updates and avoid layout jitter during dragging.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
