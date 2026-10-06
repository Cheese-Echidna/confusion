//! Own the live document, apply validated patches, increment revisions and publish change sets.
//!
//! Planned public API (not implemented): DocumentStore, DocumentChangeSet, DocumentAccess.
//!
//! Connections: application/session, document/transaction, evaluation/invalidation.
//!
//! Invariant: Use one writer and immutable worker snapshots; never hold a store lock during
//! expensive work.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
