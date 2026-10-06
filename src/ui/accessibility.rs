//! Provide keyboard traversal, accessible names, focus recovery and non-color-only diagnostic
//! states.
//!
//! Planned public API (not implemented): AccessibilityPolicy, FocusCoordinator.
//!
//! Connections: ui/components, ui/workspace, settings/schema.
//!
//! Invariant: Validate platform assistive-technology support against the selected GPUI revision
//! rather than assuming it exists.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
