//! Solve interactive drag targets with warm starts, temporary objectives and continuation across
//! small increments.
//!
//! Planned public API (not implemented): DragSolveSession, DragTarget, ContinuationState.
//!
//! Connections: tools/sketch, tools/assembly, solver/nonlinear.
//!
//! Invariant: Preserve existing hard constraints and solution branch; final accepted values commit
//! through transactions.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
