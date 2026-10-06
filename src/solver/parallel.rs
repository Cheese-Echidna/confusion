//! Schedule independent islands and parallel residual/Jacobian evaluation with a dedicated Rayon
//! pool.
//!
//! Planned public API (not implemented): ParallelSolveScheduler, SolverBudget.
//!
//! Connections: solver/decomposition, runtime/scheduler, solver/nonlinear.
//!
//! Invariant: Coordinate faer and Rayon thread budgets to avoid nested oversubscription; merge
//! results deterministically.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
