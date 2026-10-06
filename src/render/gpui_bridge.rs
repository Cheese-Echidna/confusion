//! Present a GPU viewport inside a GPUI element using an audited shared-device or native texture
//! compositor adapter.
//!
//! Planned public API (not implemented): ViewportSurface trait, GpuiViewportElement, FrameLease,
//! SurfaceCapabilities.
//!
//! Connections: ui/viewport, render/device, platform/gpu_surface.
//!
//! Invariant: GPUI 0.2.2 only exposes native surface() on macOS; a cross-platform fork/bridge must
//! be proven before interactive UI work. No per-frame CPU readback in the production path.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
