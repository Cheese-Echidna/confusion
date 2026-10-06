//! Initialize settings, logging, registries, workers, kernel and GPUI in dependency order.
//!
//! Planned public API (not implemented): ApplicationBootstrap, BootstrapOptions, run_desktop
//! function.
//!
//! Connections: application/services, settings/store, platform/lifecycle, ui/workspace.
//!
//! Invariant: The scaffold does not implement startup yet; capability failures must be surfaced
//! before enabling unsupported tools.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
