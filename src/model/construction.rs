//! Define axes, points and offset/angled/tangent/midplanes with stable references.
//!
//! Planned public API (not implemented): ConstructionDefinition, ConstructionGeometry,
//! ConstructionEvaluator.
//!
//! Connections: sketch/workplane, document/references, kernel/queries.
//!
//! Invariant: Changing support geometry invalidates construction and dependent features through the
//! DAG.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
