//! Adapt semantic command IDs and key contexts to GPUI actions and key bindings.
//!
//! Planned public API (not implemented): GpuiActionAdapter, ActionBindings, KeyContextAdapter.
//!
//! Connections: settings/keymap, commands/dispatch, application/modes.
//!
//! Invariant: Rebuild bindings atomically on settings change; preserve focus and modal precedence.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
