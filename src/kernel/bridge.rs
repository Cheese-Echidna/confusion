//! Declare private cxx bridges for shapes, algorithms, provenance, tessellation, XDE and exact
//! queries.
//!
//! Planned public API (not implemented): Crate-private ffi module and bridge value types.
//!
//! Connections: kernel/occt, kernel/ownership, native OCCT adapter to be added.
//!
//! Invariant: Catch C++ exceptions inside the native boundary; check algorithm IsDone/status before
//! extracting outputs. No FFI code is implemented yet.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
