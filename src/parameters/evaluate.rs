//! Type-check and evaluate expressions in topological dependency order, returning dirty parameters
//! and errors.
//!
//! Planned public API (not implemented): ParameterEvaluator, EvaluatedParameters,
//! ParameterDependencies.
//!
//! Connections: document/dependency, evaluation/engine, solver/problem.
//!
//! Invariant: Reject cycles and non-finite values; distinguish degree display from radian
//! computation.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
