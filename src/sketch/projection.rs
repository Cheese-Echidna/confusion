//! Project or intersect external geometry into linked sketch reference entities.
//!
//! Planned public API (not implemented): ProjectionDefinition, ProjectionResult, ProjectionService
//! trait.
//!
//! Connections: kernel/queries, model/naming, sketch/entities.
//!
//! Invariant: Keep source associations; avoid implicit self-dependency and expose broken
//! projections.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
