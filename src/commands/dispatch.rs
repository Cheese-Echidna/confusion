//! Validate and execute actions against application services and document transactions.
//!
//! Planned public API (not implemented): CommandDispatcher, CommandInvocation, CommandOutcome.
//!
//! Connections: document/transaction, application/services, tools/state.
//!
//! Invariant: Menus and shortcuts call the same dispatcher; disabled context actions cannot be
//! invoked indirectly.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
