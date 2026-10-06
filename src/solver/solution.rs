//! Return candidate values, residual norms, source IDs, convergence state and revision stamp.
//!
//! Planned public API (not implemented): SolveResult, SolvedSketch, SolvedAssembly,
//! SolveStatistics.
//!
//! Connections: evaluation/engine, application/session, sketch/diagnostics.
//!
//! Invariant: Workers return owned results; session accepts only results matching the current
//! revision and preview request.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
