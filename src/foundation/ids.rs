//! Define UUID-backed durable IDs and distinct typed IDs for document, component, occurrence, body,
//! feature, sketch, entity, constraint, parameter and material.
//!
//! Planned public API (not implemented): DocumentId, ComponentId, OccurrenceId, BodyId, FeatureId,
//! SketchId, EntityId, ConstraintId, ParameterId, MaterialId.
//!
//! Connections: document/schema, sketch/entities, model/feature, assembly/components.
//!
//! Invariant: Do not serialize arena offsets or kernel pointer addresses as identity.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
