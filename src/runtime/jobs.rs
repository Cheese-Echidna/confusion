//! Define typed solve/evaluate/tessellate/import/export/render-analysis jobs and stamped completion
//! messages.
//!
//! Planned public API (not implemented): JobId, JobRequest, JobResult, JobPriority, JobStamp.
//!
//! Connections: application/session, document/snapshot, runtime/scheduler.
//!
//! Invariant: Every result includes revision and request identity; no job captures GPUI entities
//! across threads.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
