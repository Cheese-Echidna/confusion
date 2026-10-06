//! Represent semantic references to profiles, planes and generated faces/edges using producer IDs
//! and selectors.
//!
//! Planned public API (not implemented): GeometryReference, TopologySelector, ReferenceResolution.
//!
//! Connections: model/naming, sketch/projection, assembly/joints, drawing/views.
//!
//! Invariant: Never depend on face enumeration indexes; an ambiguous match must require repair.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
