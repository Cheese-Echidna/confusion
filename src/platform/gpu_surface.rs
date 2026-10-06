//! Implement Metal/IOSurface, Vulkan/shared-device and D3D compositor adapters with explicit
//! synchronization.
//!
//! Planned public API (not implemented): NativeViewportSurface, SurfaceLease, PresentationBackend
//! trait.
//!
//! Connections: render/gpui_bridge, render/device, GPUI platform backend.
//!
//! Invariant: Pinned GPUI 0.2.2 uses blade/native backends; wgpu integration requires verified
//! interop or an owned compositor patch, not a second swapchain on the same window.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
