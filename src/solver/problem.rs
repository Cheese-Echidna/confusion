//! Compile sketch and assembly constraints into variable blocks, equations, bounds, scales and
//! fixed parameters.
//!
//! Planned public API (not implemented): ConstraintProblem, VariableBlock, ResidualBlock,
//! ProblemCompiler.
//!
//! Connections: sketch/constraints, assembly/joints, parameters/evaluate.
//!
//! Invariant: All variables use f64; record equation-to-source IDs for diagnostics.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
