//! Generate tolerance-controlled render/export meshes with normals, edge polylines, face IDs and
//! cancellation.
//!
//! Planned public API (not implemented): TessellationRequest, TessellatedShape, MeshProvenance.
//!
//! Connections: render/scene, exchange/stl, evaluation/cache.
//!
//! Invariant: Reuse exact shape revisions and distinguish display LOD from STL export quality.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
