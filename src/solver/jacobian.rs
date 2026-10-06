//! Assemble sparse Jacobian blocks, reuse symbolic sparsity and supply rank-revealing linear solves
//! with faer.
//!
//! Planned public API (not implemented): JacobianAssembler, JacobianPattern, LinearSolveBackend
//! trait.
//!
//! Connections: solver/residuals, solver/nonlinear, solver/diagnostics.
//!
//! Invariant: Avoid forming normal equations for rank diagnosis; QR/SVD are required for
//! ill-conditioned systems.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
