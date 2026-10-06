//! Define coincidence, horizontal/vertical, parallel, perpendicular, tangent, equal, concentric,
//! collinear, symmetry, midpoint, fix and smoothness constraints.
//!
//! Planned public API (not implemented): SketchConstraint, ConstraintKind, ConstraintTarget.
//!
//! Connections: solver/residuals, tools/constraints, sketch/diagnostics.
//!
//! Invariant: Driving constraints belong to the model; inferred suggestions become constraints only
//! on user acceptance.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
