//! Maintain bounded inverse/forward patch stacks and coalesce drag/text edits into logical
//! transactions.
//!
//! Planned public API (not implemented): UndoStack, UndoEntry, UndoGroup.
//!
//! Connections: document/transaction, application/session, commands/dispatch.
//!
//! Invariant: Never serialize this stack, redo chain or command chronology in .con; save
//! checkpoints are session metadata.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
