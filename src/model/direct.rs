//! Represent move face, offset face, replace/delete face and imported-body edits as declarative
//! modifier nodes.
//!
//! Planned public API (not implemented): DirectEditDefinition, DirectEditEvaluator.
//!
//! Connections: model/feature, kernel/api, model/naming.
//!
//! Invariant: Direct manipulation must produce editable parameters and semantic references; no
//! baked mesh replaces parametric intent.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
