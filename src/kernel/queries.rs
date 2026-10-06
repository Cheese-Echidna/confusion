//! Provide mass properties, distance, closest points, ray hits, curvature, interference and section
//! queries.
//!
//! Planned public API (not implemented): GeometryQuery trait, MassProperties, ExactHit,
//! SectionResult.
//!
//! Connections: interaction/picking, assembly/contact, model/measure, drawing/views.
//!
//! Invariant: Exact B-rep is authoritative; render meshes are broad-phase approximations.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
