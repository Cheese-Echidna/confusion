//! Construct concrete adapters and provide shared service handles for commands and sessions.
//!
//! Planned public API (not implemented): ApplicationServices, ServiceBuilder.
//!
//! Connections: runtime/scheduler, kernel/occt, evaluation/engine, persistence/reader.
//!
//! Invariant: Inject interfaces for headless tests; domain modules must not depend back on
//! application services.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
