//! Implement WGSL pipelines for PBR shading, hidden/visible edges, x-ray, clipping, shadows and
//! selection outlines.
//!
//! Planned public API (not implemented): RenderPassSet, PipelineCache, ViewStyle.
//!
//! Connections: render/renderer, render/materials, render/overlays.
//!
//! Invariant: Linear-light shading with explicit output color space; screen-space edge widths must
//! scale with DPI.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
