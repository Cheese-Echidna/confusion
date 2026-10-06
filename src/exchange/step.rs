//! Read/write STEP through OCCT XDE including assembly hierarchy, names, colors and unit metadata.
//!
//! Planned public API (not implemented): StepReader, StepWriter, StepOptions, StepTransferReport.
//!
//! Connections: kernel/bridge, assembly/components, materials/appearance.
//!
//! Invariant: AP242 support must be verified for the pinned OCCT build; do not promise parametric
//! intent transfer.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
