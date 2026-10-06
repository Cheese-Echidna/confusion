//! Store document material definitions and built-in catalogs with physical properties and units.
//!
//! Planned public API (not implemented): MaterialDefinition, MaterialLibrary, MaterialProperties.
//!
//! Connections: document/schema, assembly/bom, model/measure.
//!
//! Invariant: Copy used definitions into the document; immutable built-ins can have versioned
//! catalog IDs.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
