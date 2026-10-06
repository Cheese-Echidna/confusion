//! Define defaults for appearance, navigation, units, rendering, solving, autosave, panels,
//! accessibility and keymaps.
//!
//! Planned public API (not implemented): ApplicationSettings, SettingsVersion, SolverSettings,
//! RenderSettings.
//!
//! Connections: settings/store, application/bootstrap, settings/keymap.
//!
//! Invariant: Document intent stays in .con; all application preferences and shortcut overrides
//! stay in settings.json.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
