//! Orchestrate depth, shaded bodies, edges, transparency, selection, overlays and presentation
//! passes.
//!
//! Planned public API (not implemented): ViewportRenderer, RenderFrame, RenderOptions.
//!
//! Connections: render/device, render/passes, render/gpui_bridge.
//!
//! Invariant: No solving or geometry evaluation on the render thread; avoid per-frame allocation
//! and mesh uploads.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
