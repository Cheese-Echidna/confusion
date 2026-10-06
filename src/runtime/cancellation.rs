//! Provide cancellation tokens and cooperative checkpoints with latest-request replacement.
//!
//! Planned public API (not implemented): CancellationToken, CancellationSource, Cancelled.
//!
//! Connections: solver/nonlinear, evaluation/engine, exchange/import.
//!
//! Invariant: Native algorithms may not stop immediately; canceled outputs must never publish even
//! if native work finishes.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
