//! Evaluate transient tool overlays against a committed snapshot without committing model changes.
//!
//! Planned public API (not implemented): PreviewRequest, PreviewResult, PreviewToken.
//!
//! Connections: tools/state, commands/preview, evaluation/engine.
//!
//! Invariant: Newest request wins; accepting a preview validates and commits exactly one
//! transaction.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
