//! Aggregate parameters, sketches, solid features, components, materials and associative drawing
//! definitions.
//!
//! Planned public API (not implemented): DesignDocument, DocumentMetadata, DocumentSchemaVersion.
//!
//! Connections: persistence/schema, parameters/table, sketch/entities, model/feature,
//! assembly/components.
//!
//! Invariant: The schema is current design intent; exclude undo stacks, chronological command logs
//! and derived GPU/solver/kernel state.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
