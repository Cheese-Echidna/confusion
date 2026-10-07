//! Audited value-only CXX boundary: OCCT shapes never escape their worker invocation.
//! Exports owned Mesh/Vertex through extrude. Coordinates and exact volume are SI;
//! meshing is derived display data. Native exceptions become Rust Result errors.
#[cfg(feature = "kernel")]
#[cxx::bridge(namespace = "confusion")]
pub mod ffi {
    struct ProfileEdge {
        wire: u32,
        sx: f64,
        sy: f64,
        ex: f64,
        ey: f64,
        cx: f64,
        cy: f64,
        sweep: f64,
    }
    struct ModelStep {
        edge_start: u32,
        edge_count: u32,
        depth: f64,
        operation: u32,
        target: i32,
        support: i32,
        producer: i32,
        role: u32,
    }
    struct FaceRequest {
        support: i32,
        producer: i32,
        role: u32,
    }
    struct FaceAnchor {
        face: u32,
        support: u32,
        producer: u32,
        role: u32,
        ambiguous: bool,
    }
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
        anchors: Vec<FaceAnchor>,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        volume: f64,
        faces: u32,
    }
    unsafe extern "C++" {
        include!("src/kernel/native/occt.hpp");
        fn evaluate_model(
            edges: &[ProfileEdge],
            steps: &[ModelStep],
            planes: &[FaceRequest],
        ) -> Result<Mesh>;
        fn extrude_region(edges: &[ProfileEdge], depth: f64) -> Result<Mesh>;
        fn extrude(profile: &[Point2], depth: f64) -> Result<Mesh>;
    }
}
