//! Define point, line, arc, circle, ellipse, conic, spline and text-outline entities, plus
//! construction/reference flags.
//!
//! Planned public API (not implemented): Sketch, SketchEntity, SketchGeometry, SplineDefinition,
//! EntityFlags.
//!
//! Connections: document/schema, solver/problem, sketch/profiles, tools/sketch.
//!
//! Invariant: Persist stable entity IDs and numeric seed geometry for deterministic branch
//! selection.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
