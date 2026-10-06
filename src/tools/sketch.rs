//! Implement line/polyline, rectangle, circle, arc, ellipse, spline, text, trim and pattern tool
//! interactions.
//!
//! Planned public API (not implemented): SketchTool, SketchToolKind, SketchToolParameters.
//!
//! Connections: sketch/entities, sketch/edit, sketch/inference, tools/state.
//!
//! Invariant: Fusion defaults map to tools through settings, while tool-specific prompts drive
//! contextual shortcuts.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
