//! Read/write binary and ASCII STL using explicit units, tessellation tolerance and manifold
//! validation.
//!
//! Planned public API (not implemented): StlReader, StlWriter, StlOptions, StlValidationReport.
//!
//! Connections: kernel/tessellation, exchange/import, exchange/export.
//!
//! Invariant: STL has no standard units or constraints; export scale and watertightness must be
//! checked.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
