//! Display constraint/evaluation failures and navigate to affected geometry with repair actions.
//!
//! Planned public API (not implemented): DiagnosticsPanel, DiagnosticItem, RepairActionView.
//!
//! Connections: sketch/diagnostics, evaluation/diagnostics, commands/registry.
//!
//! Invariant: Explain status in plain CAD terms and distinguish stale previews from valid current
//! results.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
