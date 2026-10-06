//! Store typed user and feature parameters with names, scope, expressions, bounds and descriptions.
//!
//! Planned public API (not implemented): ParameterTable, Parameter, ParameterScope.
//!
//! Connections: document/schema, model/feature, sketch/dimensions.
//!
//! Invariant: Parameter IDs survive rename; reject duplicate names within a scope.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
