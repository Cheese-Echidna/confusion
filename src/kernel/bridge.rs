//! Audited value-only CXX boundary: OCCT shapes never escape their worker invocation.
//! Exports owned Mesh/Vertex through extrude. Coordinates and exact volume are SI;
//! meshing is derived display data. Native exceptions become Rust Result errors.
#[cfg(feature = "kernel")]
#[cxx::bridge(namespace = "confusion")]
pub mod ffi {
    struct Point2 {
        x: f64,
        y: f64,
    }
    struct Vertex {
        x: f64,
        y: f64,
        z: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        face: u32,
    }
    struct Mesh {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        volume: f64,
        faces: u32,
    }
    unsafe extern "C++" {
        include!("src/kernel/native/occt.hpp");
        fn extrude(profile: &[Point2], depth: f64) -> Result<Mesh>;
    }
}
