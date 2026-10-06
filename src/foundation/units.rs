//! Implement dimensional quantities and conversion for length, angle, mass, time and derived units.
//!
//! Planned public API (not implemented): Quantity, Dimension, Unit, UnitSystem, UnitError.
//!
//! Connections: parameters/expressions, settings/schema, exchange/step.
//!
//! Invariant: Canonical SI values internally; preserve preferred display units separately and
//! reject dimensionally invalid arithmetic.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
