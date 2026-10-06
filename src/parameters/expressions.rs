//! Parse a purpose-built expression AST with literals, units, references, arithmetic and approved
//! mathematical functions.
//!
//! Planned public API (not implemented): Expression, ExprNode, ExpressionParser, ExpressionError.
//!
//! Connections: parameters/table, parameters/evaluate, ui/parameter_table.
//!
//! Invariant: Resolve names to stable IDs; no arbitrary code execution or untyped eval library.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
