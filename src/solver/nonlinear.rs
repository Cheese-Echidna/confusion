//! Implement damped least squares/trust-region iteration, scaling, bounds, warm starts and branch
//! continuity.
//!
//! Planned public API (not implemented): ConstraintSolver trait, RustConstraintSolver,
//! SolveOptions, SolveTermination.
//!
//! Connections: solver/jacobian, solver/solution, solver/parallel.
//!
//! Invariant: Bound iterations and time, honor cancellation between iterations, and never report
//! convergence using step size alone.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
