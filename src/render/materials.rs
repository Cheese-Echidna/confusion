//! Upload PBR parameters/textures, manage samplers, environments and appearance resource caches.
//!
//! Planned public API (not implemented): GpuMaterial, TextureCache, EnvironmentLighting.
//!
//! Connections: materials/appearance, render/passes, persistence/assets.
//!
//! Invariant: GPU assets are derived from document references and are never authoritative model
//! data.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
