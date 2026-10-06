//! Apply structural edits atomically with preconditions, validation and invertible patches.
//!
//! Planned public API (not implemented): DocumentTransaction, DocumentPatch, PatchOperation,
//! TransactionError.
//!
//! Connections: commands/dispatch, document/store, document/validation.
//!
//! Invariant: Failed edits leave state unchanged; previews do not mutate committed state.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
