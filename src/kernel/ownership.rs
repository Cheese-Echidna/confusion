//! Manage opaque shape handles and audited destruction with worker confinement and explicit
//! cloning.
//!
//! Planned public API (not implemented): ShapeHandle, ShapeId, KernelShapeStore.
//!
//! Connections: kernel/session, kernel/occt, evaluation/cache.
//!
//! Invariant: Do not mark OCCT wrappers Send/Sync without an audit; mutable native objects remain
//! worker-local.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
