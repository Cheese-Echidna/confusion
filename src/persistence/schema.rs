//! Define stable serde DTOs and explicit conversion between disk format and domain state.
//!
//! Planned public API (not implemented): ConDocumentDto, ConSchemaVersion, SchemaConversion.
//!
//! Connections: document/schema, persistence/migration, persistence/container.
//!
//! Invariant: Do not automatically serialize runtime structs; field evolution must remain
//! deliberate and validated.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
