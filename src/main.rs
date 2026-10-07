//! Confusion desktop entry point. Connections: application/bootstrap creates the
//! parametric sketch/model workspace when desktop is enabled; domain logic remains in the library.

fn main() {
    #[cfg(feature = "desktop")]
    confusion::application::bootstrap::run_desktop();
    #[cfg(not(feature = "desktop"))]
    println!("Run Confusion with: cargo run --features desktop");
}
