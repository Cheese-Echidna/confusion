//! Define structured domain errors with stable codes, entity context and actionable diagnostics.
//!
//! Planned public API (not implemented): DomainError, Diagnostic, DiagnosticCode, Severity.
//!
//! Connections: evaluation/diagnostics, application/events, ui/diagnostics.
//!
//! Invariant: No GPUI types or user-visible raw C++ exception strings in domain errors.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
