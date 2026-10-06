//! Implement axis/plane/rotation handles, dimension drag and joint manipulator interactions.
//!
//! Planned public API (not implemented): Gizmo, GizmoState, ManipulationIntent.
//!
//! Connections: render/overlays, tools/model, tools/assembly.
//!
//! Invariant: Gizmos edit preview parameters; acceptance uses a command transaction.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
