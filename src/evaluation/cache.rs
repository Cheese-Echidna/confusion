//! Cache parameter/solve/shape/mesh results with semantic input hashes, backend versions and
//! tolerances.
//!
//! Planned public API (not implemented): EvaluationCache, CacheKey, CacheEntry.
//!
//! Connections: evaluation/engine, kernel/ownership, runtime/scheduler.
//!
//! Invariant: Caches are disposable and excluded from canonical model state; no hash key relies on
//! edit chronology.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
