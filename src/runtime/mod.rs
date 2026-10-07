//! Provide bounded job scheduling and cancellation; application owns lifecycle while workers operate only on snapshots.
//!
//! Connections: parent lib.rs exposes this namespace; sibling connections are
//! specified per child file and constrained by docs/architecture.md.
//!
//! Planned exports live in the child modules documented below. This module currently
//! exports only module namespaces. Follow the dependency rules in docs/architecture.md.

pub mod cancellation;
pub mod jobs;
pub mod progress;
pub mod scheduler;
pub mod telemetry;

pub mod worker;

pub mod candidate;
