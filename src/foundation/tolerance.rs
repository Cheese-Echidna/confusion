//! Separate modeling, coincidence, solver, tessellation and display tolerances with scale-aware
//! policies.
//!
//! Planned public API (not implemented): TolerancePolicy, ModelTolerance, SolveTolerance,
//! MeshTolerance.
//!
//! Connections: kernel/validation, solver/nonlinear, sketch/profiles.
//!
//! Invariant: Never use screen-space snap distance as a geometric tolerance.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
