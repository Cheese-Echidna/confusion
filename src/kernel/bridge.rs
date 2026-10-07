//! Audited CXX boundary: opaque OCCT caches remain confined to their owning worker.
//! Exports owned Mesh/Vertex through extrude. Coordinates and exact volume are SI;
//! meshing is derived display data. Native exceptions become Rust Result errors.
#[cfg(feature = "kernel")]
use crate::kernel::cancellation::{EvaluationCancellation, evaluation_cancelled};
#[cfg(feature = "kernel")]
#[cxx::bridge(namespace = "confusion")]
pub mod ffi {
    struct CurveIntersection {
        first: f64,
        second: f64,
    }
    struct ProfilePoint {
        x: f64,
        y: f64,
    }
    struct ProfileEdge {
        wire: u32,
        sx: f64,
        sy: f64,
        ex: f64,
        ey: f64,
        cx: f64,
        cy: f64,
        sweep: f64,
        kind: u32,
        poles: Vec<ProfilePoint>,
        identity: String,
        from: f64,
        to: f64,
    }
    struct ModelStep {
        identity: String,
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
        identity: String,
        face_reference: String,
        tool_reference: String,
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
        identity: String,
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
    #[derive(Clone)]
    struct FaceAnchor {
        face: u32,
        support: u32,
        producer: u32,
        role: u32,
        ambiguous: bool,
    }
    #[derive(Clone)]
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
    #[derive(Clone)]
    struct BodyFace {
        face: u32,
        body: u32,
        part: u32,
    }
    struct Point2 {
        x: f64,
        y: f64,
    }
    #[derive(Clone)]
    struct Vertex {
        x: f64,
        y: f64,
        z: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        face: u32,
    }
    #[derive(Clone)]
    struct FaceInspection {
        face: u32,
        area: f64,
        min_curvature: f64,
        max_curvature: f64,
        min_draft: f64,
        max_draft: f64,
        samples: u32,
    }
    #[derive(Clone)]
    struct Interference {
        first: u32,
        second: u32,
        volume: f64,
    }
    #[derive(Clone)]
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
    #[derive(Clone)]
    struct BoundFaceReference {
        edit: String,
        face: String,
        tool: String,
    }
    #[derive(Clone)]
    struct NamedFace {
        face: u32,
        key: String,
        ambiguous: bool,
    }
    #[derive(Clone)]
    struct Mesh {
        names: Vec<NamedFace>,
        references: Vec<BoundFaceReference>,
        inspection: Inspection,
        planes: Vec<PlaneFrame>,
        bodies: Vec<BodyFace>,
        anchors: Vec<FaceAnchor>,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        volume: f64,
        faces: u32,
    }
    extern "Rust" {
        type EvaluationCancellation;
        fn evaluation_cancelled(token: &EvaluationCancellation) -> bool;
    }
    unsafe extern "C++" {
        include!("src/kernel/native/occt.hpp");
        type ModelCache;
        fn new_model_cache() -> UniquePtr<ModelCache>;
        fn evaluate_cached_model(
            cache: Pin<&mut ModelCache>,
            edges: &[ProfileEdge],
            steps: &[ModelStep],
            planes: &[FaceRequest],
            edits: &[ModifyStep],
            creates: &[CreateStep],
            keys: &[String],
        ) -> Result<Mesh>;
        fn evaluate_cancellable_model(
            cache: Pin<&mut ModelCache>,
            edges: &[ProfileEdge],
            steps: &[ModelStep],
            planes: &[FaceRequest],
            edits: &[ModifyStep],
            creates: &[CreateStep],
            keys: &[String],
            token: &EvaluationCancellation,
        ) -> Result<Mesh>;
        fn reused_features(self: &ModelCache) -> usize;
        fn write_step(cache: &ModelCache, path: &str) -> Result<Inspection>;
        fn inspect_step(path: &str) -> Result<Mesh>;
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
        fn profile_intersections(
            first: &ProfileEdge,
            second: &ProfileEdge,
            same: bool,
        ) -> Result<Vec<CurveIntersection>>;
        fn extrude_region(edges: &[ProfileEdge], depth: f64) -> Result<Mesh>;
        fn extrude(profile: &[Point2], depth: f64) -> Result<Mesh>;
    }
}
