//! Load container data, verify asset hashes, migrate schemas, validate references and construct a
//! current design.
//!
//! Planned public API (not implemented): ConReader, LoadOptions, LoadResult.
//!
//! Connections: persistence/container, document/validation, persistence/schema.
//!
//! Invariant: Never invoke geometry computation while parsing; reject unsupported future formats
//! without rewriting them.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
