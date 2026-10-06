//! Maintain generated/modified/deleted subshape provenance and resolve stable semantic selectors
//! across reevaluation.
//!
//! Planned public API (not implemented): TopologyNamingService, ProvenanceMap, NamedTopology,
//! NamingConflict.
//!
//! Connections: document/references, kernel/api, sketch/projection, assembly/joints.
//!
//! Invariant: Combine producer role, ancestry and geometry signatures; never silently select an
//! arbitrary face after a split/merge.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
