//! Provide file/import/export dialogs, confirmations for destructive model actions and
//! progress/cancel views.
//!
//! Planned public API (not implemented): DialogHost, ImportDialog, ExportDialog, ErrorDialog.
//!
//! Connections: platform/dialogs, exchange/export, persistence/reader.
//!
//! Invariant: Persist dialog preferences in settings.json; errors explain units, formats and failed
//! operations.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
