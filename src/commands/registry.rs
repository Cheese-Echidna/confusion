//! Register stable action IDs, contextual availability, argument schemas, labels and help.
//!
//! Planned public API (not implemented): CommandRegistry, CommandDescriptor, CommandId,
//! CommandContext.
//!
//! Connections: ui/toolbar, ui/command_palette, settings/keymap.
//!
//! Invariant: Action IDs are stable serialized shortcut targets; registry contains no hard-coded
//! input chords.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
