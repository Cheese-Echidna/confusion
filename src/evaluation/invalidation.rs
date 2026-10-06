//! Compute transitive dirty sets from semantic change sets and configuration changes.
//!
//! Planned public API (not implemented): InvalidationPlanner, DirtySet, DependencyDelta.
//!
//! Connections: document/dependency, document/store, evaluation/cache.
//!
//! Invariant: Camera, selection and UI settings must not trigger geometry reevaluation.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
