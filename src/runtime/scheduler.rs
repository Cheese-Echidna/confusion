//! Manage bounded queues, interactive/background priorities and dedicated solver/kernel/IO
//! executors.
//!
//! Planned public API (not implemented): JobScheduler, WorkerBudget, SchedulerMetrics.
//!
//! Connections: solver/parallel, kernel/session, runtime/jobs.
//!
//! Invariant: Reserve capacity for interactive dragging; apply backpressure and avoid unbounded
//! spawning.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
