//! Own native shape caches and serialize unaudited operations on a dedicated kernel worker.
//!
//! Planned public API (not implemented): KernelSession, KernelJob, KernelResponse.
//!
//! Connections: runtime/jobs, kernel/ownership, evaluation/engine.
//!
//! Invariant: Parallelize only independent, audited native operations; never share mutable shapes
//! or process-global exchange settings across workers.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
