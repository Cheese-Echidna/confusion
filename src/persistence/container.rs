//! Define a ZIP-based .con container with manifest.json, design.json and content-addressed assets.
//!
//! Planned public API (not implemented): ConContainer, ContainerManifest, ContainerLimits.
//!
//! Connections: persistence/reader, persistence/writer, persistence/assets.
//!
//! Invariant: Bound entry counts and expansion sizes; disallow traversal, duplicate names and
//! external-path writes.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
