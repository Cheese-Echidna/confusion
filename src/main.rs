//! Future desktop entry point for Confusion.
//!
//! Planned exports: none; main should parse launch/open-file options and delegate to
//! confusion::application::bootstrap::run_desktop, then report startup errors.
//!
//! Connections: application/bootstrap owns GPUI, services, settings and worker startup.
//! Keep model, solver, native bridge and UI implementation out of this binary entry point.
//! This executable currently identifies the architecture scaffold; it is not a CAD app.

fn main() {
    println!("Confusion architecture scaffold; see docs/architecture.md.");
}
