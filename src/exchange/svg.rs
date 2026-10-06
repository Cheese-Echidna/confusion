//! Translate planar vector paths and sketch/drawing projections to/from SVG.
//!
//! Planned public API (not implemented): SvgReader, SvgWriter, SvgOptions.
//!
//! Connections: sketch/entities, drawing/views, exchange/import.
//!
//! Invariant: Explicitly handle transforms, units and unsupported effects; raster image traces
//! require separate algorithms.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
