//! Implement geometry-specific residuals and analytic derivatives, with a small dual-number
//! fallback for complex constraints.
//!
//! Planned public API (not implemented): ConstraintResidual trait, ResidualEvaluation,
//! DerivativeScalar.
//!
//! Connections: solver/problem, solver/jacobian, foundation/math.
//!
//! Invariant: Separate hard equations from drag objectives; unit-normalize residuals and handle
//! degenerate geometry explicitly.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
