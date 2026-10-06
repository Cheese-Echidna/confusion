//! Export vector PDF/SVG/DXF and plot sheets with embedded fonts and explicit scale.
//!
//! Planned public API (not implemented): DrawingExporter trait, DrawingExportOptions, PlotResult.
//!
//! Connections: exchange/dxf, exchange/svg, drawing/views.
//!
//! Invariant: A PDF library handles encoding only; line weights, standards and associative layout
//! are owned implementation work.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
