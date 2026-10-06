//! Merge GPU broad-phase hits, exact refinement, sketch proximity and selection priorities.
//!
//! Planned public API (not implemented): PickService, HitCandidate, PickContext.
//!
//! Connections: render/picking, kernel/queries, interaction/selection.
//!
//! Invariant: Do not block pointer event handling on exact kernel queries; ignore stale
//! asynchronous hits.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
