//! Render stable object/face/edge/entity IDs and perform bounded asynchronous hit readback.
//!
//! Planned public API (not implemented): GpuPicker, PickRequest, PickResult, PickIdMap.
//!
//! Connections: interaction/picking, render/scene, render/renderer.
//!
//! Invariant: Discard picks for outdated scenes; exact refinement belongs to kernel queries, not
//! GPU ID buffers.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
