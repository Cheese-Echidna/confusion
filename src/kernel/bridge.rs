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
    struct ModifyStep {
        kind: u32,
        target: u32,
        tool: i32,
        face: u32,
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        copy: bool,
        mode: u32,
    }
    struct CreateStep {
        kind: u32,
        edge_start: u32,
        edge_count: u32,
        second_start: u32,
        second_count: u32,
        target: i32,
        second_target: i32,
        values: Vec<f64>,
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
    struct PlaneFrame {
        ox: f64,
        oy: f64,
        oz: f64,
        xx: f64,
        xy: f64,
        xz: f64,
        yx: f64,
        yy: f64,
        yz: f64,
        nx: f64,
        ny: f64,
        nz: f64,
    }
    struct BodyFace {
        face: u32,
        body: u32,
        part: u32,
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
    struct FaceInspection {
        face: u32,
        area: f64,
        min_curvature: f64,
        max_curvature: f64,
        min_draft: f64,
        max_draft: f64,
        samples: u32,
    }
    struct Interference {
        first: u32,
        second: u32,
        volume: f64,
    }
    struct Inspection {
        area: f64,
        cx: f64,
        cy: f64,
        cz: f64,
        valid: bool,
        solids: u32,
        faces: Vec<FaceInspection>,
        interference: Vec<Interference>,
        error: String,
    }
    struct Mesh {
        inspection: Inspection,
        planes: Vec<PlaneFrame>,
        bodies: Vec<BodyFace>,
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
        fn evaluate_modified_model(
            edges: &[ProfileEdge],
            steps: &[ModelStep],
            planes: &[FaceRequest],
            edits: &[ModifyStep],
        ) -> Result<Mesh>;
        fn evaluate_complete_model(
            edges: &[ProfileEdge],
            steps: &[ModelStep],
            planes: &[FaceRequest],
            edits: &[ModifyStep],
            creates: &[CreateStep],
        ) -> Result<Mesh>;
        fn evaluate_create_model(
            edges: &[ProfileEdge],
            steps: &[ModelStep],
            planes: &[FaceRequest],
            creates: &[CreateStep],
        ) -> Result<Mesh>;
        fn extrude_region(edges: &[ProfileEdge], depth: f64) -> Result<Mesh>;
        fn extrude(profile: &[Point2], depth: f64) -> Result<Mesh>;
    }
}
