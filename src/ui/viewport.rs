//! Implement GPUI layout/focus/input around the 3D viewport and overlay status.
//!
//! Planned public API (not implemented): ViewportView, ViewportState.
//!
//! Connections: render/gpui_bridge, interaction/navigation, tools/state.
//!
//! Invariant: Only presentation and event routing here; do not invoke solving during GPUI
//! render/layout.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
