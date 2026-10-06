//! Wrap GPUI/native open/save dialogs and file association requests.
//!
//! Planned public API (not implemented): FileDialogService trait, NativeFileDialogs,
//! FileTypeFilter.
//!
//! Connections: ui/dialogs, application/workspace.
//!
//! Invariant: Handle portal/sandbox permissions and asynchronous window lifetime.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
