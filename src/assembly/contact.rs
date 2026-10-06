//! Perform broad-phase collision and exact interference/contact queries for assembly motion.
//!
//! Planned public API (not implemented): ContactSet, InterferenceResult, AssemblyCollisionService.
//!
//! Connections: kernel/queries, assembly/kinematics, runtime/jobs.
//!
//! Invariant: Define contact support and friction assumptions explicitly; collision checking alone
//! is not a physical simulator.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
