//! Evaluate motion links, rigid groups and driven joint motion using shared constraint equations.
//!
//! Planned public API (not implemented): KinematicSystem, MotionLink, MotionDriver, MotionResult.
//!
//! Connections: solver/continuation, assembly/joints, render/scene.
//!
//! Invariant: Time samples are derived; persist drivers and constraints, not playback history.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
