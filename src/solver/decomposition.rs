//! Partition disconnected systems and identify strongly coupled groups, elimination opportunities
//! and gauge freedom.
//!
//! Planned public API (not implemented): SolvePartition, ConstraintDecomposer, GaugeFreedom.
//!
//! Connections: solver/problem, solver/parallel, solver/diagnostics.
//!
//! Invariant: Constraint graphs can be cyclic; decomposition must preserve their equations.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
