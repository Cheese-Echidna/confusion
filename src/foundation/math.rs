//! Define double-precision points, vectors, rigid transforms, bounds and coordinate frames using
//! nalgebra.
//!
//! Planned public API (not implemented): Point2, Point3, Vector3, Transform3, PlaneFrame, Bounds3.
//!
//! Connections: sketch/workplane, kernel/queries, render/camera.
//!
//! Invariant: Right-handed Z-up coordinates; convert to camera-relative f32 only at the GPU
//! boundary.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
