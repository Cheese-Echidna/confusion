//! Load, validate, atomically save and hot-reload one settings.json file, preserving unknown future
//! fields.
//!
//! Planned public API (not implemented): SettingsStore, SettingsChangeSet, SettingsDiagnostic.
//!
//! Connections: platform/paths, persistence/atomic, settings/migration.
//!
//! Invariant: Malformed reload keeps last valid settings; debounce own writes and file watcher
//! events.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
