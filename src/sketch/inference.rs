//! Generate temporary snap and constraint candidates from cursor position, existing entities and
//! mode.
//!
//! Planned public API (not implemented): InferenceEngine, SnapCandidate, ConstraintSuggestion.
//!
//! Connections: tools/sketch, interaction/snapping, sketch/constraints.
//!
//! Invariant: Screen-space ranking is transient; do not silently commit constraints.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
