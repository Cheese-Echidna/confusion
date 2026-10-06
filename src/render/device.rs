//! Create or adopt GPU device/queue, negotiate features, recover from device loss and manage
//! resource lifetimes.
//!
//! Planned public API (not implemented): RenderDevice, DeviceCapabilities, DeviceRecovery.
//!
//! Connections: render/renderer, render/gpui_bridge, runtime/scheduler.
//!
//! Invariant: Prefer a shared compositor device; imported texture ownership and fences must be
//! explicit.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
