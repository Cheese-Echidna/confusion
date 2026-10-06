//! Define typed application events and subscriptions for changes, progress and user-facing
//! failures.
//!
//! Planned public API (not implemented): ApplicationEvent, EventSubscription, Notification.
//!
//! Connections: document/store, runtime/progress, ui/status_bar.
//!
//! Invariant: Prefer bounded/coalesced delivery; services expose domain data rather than GPUI
//! callbacks.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
