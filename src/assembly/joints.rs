//! Define rigid, revolute, slider, cylindrical, pin-slot, planar and ball joints with limits and
//! alignment references.
//!
//! Planned public API (not implemented): JointDefinition, JointKind, JointLimit, JointOrigin.
//!
//! Connections: solver/problem, document/references, tools/assembly.
//!
//! Invariant: Joints constrain occurrence transforms; as-built and positioned joint creation share
//! declarative intent.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
