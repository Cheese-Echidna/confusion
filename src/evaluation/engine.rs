//! Evaluate dirty dependency layers: parameters, planes/projections, sketch solves/profiles, exact
//! features, assemblies and downstream analyses.
//!
//! Planned public API (not implemented): EvaluationEngine, EvaluationRequest, EvaluationResult.
//!
//! Connections: document/snapshot, solver/nonlinear, kernel/session, runtime/jobs.
//!
//! Invariant: Publish consistent snapshots; failed nodes block dependents while retaining
//! explicitly stale last-good previews.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
