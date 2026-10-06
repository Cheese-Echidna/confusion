//! Implement same-directory temporary writes, flush, atomic replacement and platform-specific
//! durability handling.
//!
//! Planned public API (not implemented): AtomicFileWriter, AtomicWriteOptions, AtomicWriteError.
//!
//! Connections: persistence/writer, settings/store, platform/files.
//!
//! Invariant: Handle overwrite races and permission errors; filesystem rename guarantees differ by
//! platform.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
