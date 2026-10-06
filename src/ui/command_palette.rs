//! Implement Zed-style command search and contextual availability with keybinding hints.
//!
//! Planned public API (not implemented): CommandPalette, CommandSearchResult.
//!
//! Connections: commands/registry, settings/keymap, ui/actions.
//!
//! Invariant: Use a single command catalog; search covers tools, settings and relevant repair
//! actions.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
