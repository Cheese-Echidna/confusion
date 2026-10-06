//! Upgrade settings schemas and renamed action IDs with reversible backups and clear diagnostics.
//!
//! Planned public API (not implemented): SettingsMigrator, SettingsMigration, MigrationResult.
//!
//! Connections: settings/store, commands/registry, settings/schema.
//!
//! Invariant: Never create separate keymap or theme preference files; preserve user overrides on
//! default updates.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
