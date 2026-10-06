//! Implement trim, extend, offset, mirror, rectangular/circular patterns, break and spline control
//! edits as declarative patches.
//!
//! Planned public API (not implemented): SketchEdit, SketchEditService.
//!
//! Connections: document/transaction, kernel/curves, tools/sketch.
//!
//! Invariant: Preserve unaffected IDs and repair constraint references atomically.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
