//! Build immutable draw packets, component instances, topology IDs and incremental scene deltas
//! from evaluation.
//!
//! Planned public API (not implemented): RenderScene, SceneDelta, DrawInstance, SceneRevision.
//!
//! Connections: evaluation/engine, kernel/tessellation, render/renderer.
//!
//! Invariant: GPU data uses camera-relative f32 while source geometry remains f64; scene and pick
//! IDs share revision stamps.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
