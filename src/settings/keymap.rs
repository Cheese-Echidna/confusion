//! Define platform-aware chords and context predicates for sketch/model modes, tool stages,
//! viewport and text focus.
//!
//! Planned public API (not implemented): Keymap, KeyBinding, KeyContext, KeyChord, KeymapResolver.
//!
//! Connections: commands/registry, ui/actions, tools/state.
//!
//! Invariant: Deterministic precedence: modal tool, focused control, mode, global; text entry
//! prevents accidental modeling actions.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
