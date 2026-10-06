//! Coordinate cancellable STEP/STL/IGES/DXF/SVG imports with explicit units and healing policies.
//!
//! Planned public API (not implemented): ImportService, ImportRequest, ImportResult, ImportReport.
//!
//! Connections: kernel/validation, persistence/assets, document/transaction.
//!
//! Invariant: Imported exact bodies and reference triangle meshes are embedded source nodes; Fusion
//! proprietary feature intent cannot be inferred from STEP.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
