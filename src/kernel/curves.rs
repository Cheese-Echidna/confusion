//! Build exact sketch wires and analytic/NURBS curves; expose intersections, splitting and offset
//! operations.
//!
//! Planned public API (not implemented): CurveDefinition, WireBuilder, CurveQuery.
//!
//! Connections: sketch/profiles, sketch/edit, kernel/occt.
//!
//! Invariant: Preserve entity-to-edge provenance and plane transforms.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
