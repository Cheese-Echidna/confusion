//! Define exact geometry operations, capabilities, modeling requests and typed results independent
//! of OCCT classes.
//!
//! Planned public API (not implemented): GeometryKernel trait, KernelCapabilities, ModelingRequest,
//! ModelingResult.
//!
//! Connections: model/operations, evaluation/engine, kernel/occt.
//!
//! Invariant: Operations return provenance maps and validity diagnostics as well as shapes.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
