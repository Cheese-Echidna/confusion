//! Store source B-reps/meshes, textures, fonts used for geometry, images and linked-part snapshots
//! by digest.
//!
//! Planned public API (not implemented): AssetStore, AssetId, AssetManifest, EmbeddedAsset.
//!
//! Connections: persistence/container, materials/appearance, exchange/import.
//!
//! Invariant: Imported geometry is authoritative source data; pin font/texture/linked-part content
//! for offline reproducibility.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
