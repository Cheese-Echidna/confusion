//! Explain grounded parts, remaining motion DOF, inconsistent joints and broken attachment
//! references.
//!
//! Planned public API (not implemented): AssemblyDiagnostic, JointConflict, MotionDegreesOfFreedom.
//!
//! Connections: solver/diagnostics, assembly/joints, ui/diagnostics.
//!
//! Invariant: Map solver coordinates to user-understandable translation/rotation freedoms.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
