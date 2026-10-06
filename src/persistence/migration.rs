//! Upgrade older declarative schemas while preserving semantic IDs and validating feature
//! interpretation.
//!
//! Planned public API (not implemented): ConMigrator, SchemaMigration, MigrationReport.
//!
//! Connections: persistence/schema, persistence/reader, model/registry.
//!
//! Invariant: Keep old-format fixtures; never reconstruct edit history as a migration mechanism.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
