//! Implement extrude/revolve/sweep/loft, Boolean join/cut/intersect, hole, fillet/chamfer, shell,
//! draft, rib/web, emboss and thread evaluators.
//!
//! Planned public API (not implemented): ExtrudeDefinition, RevolveDefinition, SweepDefinition,
//! LoftDefinition, ModifyDefinition, SolidFeatureEvaluator.
//!
//! Connections: kernel/api, sketch/profiles, parameters/evaluate.
//!
//! Invariant: Specify distance/to-object extents, taper, thin walls, variable radii and Boolean
//! operation explicitly; split this file by operation as implementations grow.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
