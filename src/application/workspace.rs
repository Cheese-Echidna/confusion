//! Manage documents, tabs, windows, dirty-close handling and active component/workbench context.
//!
//! Planned public API (not implemented): WorkspaceController, WorkspaceState, WorkspaceContext.
//!
//! Connections: application/session, platform/lifecycle, ui/workspace.
//!
//! Invariant: Window layout preferences live in settings.json; documents hold only model intent and
//! optional named views.
//!
//! Architecture scaffold only. Implement the contract above; do not add placeholder
//! behavior that reports success. See docs/architecture.md and docs/libraries.md.
