//! Implement GeometryKernel using a narrow owned CXX adapter around Open CASCADE.
//!
//! Planned public API (not implemented): OcctKernel, OcctVersion, OcctOptions.
//!
//! Connections: kernel/api, kernel/bridge, kernel/session.
//!
//! Invariant: OCCT supplies geometry, not the application's parametric semantics; pin native
//! version and build options.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
