#![cfg(all(feature = "solver", feature = "kernel"))]
use confusion::{
    document::{
        model::{ExtrudeFeature, ExtrudeOperation, SketchPlane},
        schema::{Design, Extrusion},
    },
    evaluation::{cache::EvaluationCache, solid},
    kernel::bridge::ffi,
    model::{
        modify::{ModifyKind, SolidEdit},
        solid_create::{CreateFeature, CreateKind},
    },
};
use uuid::Uuid;
fn mesh(d: &Design) -> ffi::Mesh {
    solid::evaluate(d, || false).unwrap().mesh.unwrap()
}
fn base() -> (Design, Uuid) {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.02, 0.02]);
    let depth = d.parameter("thickness", "10 mm".into());
    let id = Uuid::new_v4();
    d.extrusion = Some(Extrusion {
        id,
        depth,
        boundary: vec![],
    });
    d.sync_construction();
    (d, id)
}
fn attach(
    d: &mut Design,
    support: Uuid,
    m: &ffi::Mesh,
    face: u32,
    operation: ExtrudeOperation,
    depth: &str,
) {
    let reference = m
        .names
        .iter()
        .find(|n| n.face == face && !n.ambiguous)
        .unwrap()
        .key
        .clone();
    let frame = &m
        .planar_faces
        .iter()
        .find(|f| f.face == face)
        .unwrap()
        .frame;
    let vertices: Vec<_> = m.vertices.iter().filter(|v| v.face == face).collect();
    let center = vertices
        .iter()
        .fold([0.; 3], |mut a, v| {
            a[0] += v.x;
            a[1] += v.y;
            a[2] += v.z;
            a
        })
        .map(|x| x / vertices.len() as f64);
    let delta = [
        center[0] - frame.ox,
        center[1] - frame.oy,
        center[2] - frame.oz,
    ];
    let x = delta[0] * frame.xx + delta[1] * frame.xy + delta[2] * frame.xz;
    let y = delta[0] * frame.yx + delta[1] * frame.yy + delta[2] * frame.yz;
    let sketch = d
        .create_sketch(SketchPlane::NamedFace { support, reference })
        .unwrap();
    d.rectangle([x - 0.001, y - 0.001], [x + 0.001, y + 0.001]);
    let depth = d.parameter("newDepth", depth.into());
    d.features.push(ExtrudeFeature {
        id: Uuid::new_v4(),
        name: "Face extrusion".into(),
        sketch,
        boundary: vec![],
        depth,
        operation,
        target: if operation == ExtrudeOperation::NewBody {
            None
        } else {
            Some(support)
        },
    });
    d.sync_construction();
    d.validate().unwrap();
}
#[test]
fn every_flat_extrusion_face_can_support_a_cut() {
    let (d, id) = base();
    let original = mesh(&d);
    assert_eq!(original.planar_faces.len(), 6);
    let mut cache = EvaluationCache::default();
    solid::evaluate_cached(&d, || false, &mut cache).unwrap();
    for plane in &original.planar_faces {
        let mut candidate = d.clone();
        attach(
            &mut candidate,
            id,
            &original,
            plane.face,
            ExtrudeOperation::Cut,
            "2 mm",
        );
        assert!((mesh(&candidate).volume - (original.volume - 0.002f64.powi(3))).abs() < 1e-12);
        let cached = solid::evaluate_cached(&candidate, || false, &mut cache)
            .unwrap()
            .mesh
            .unwrap();
        assert!((cached.volume - (original.volume - 0.002f64.powi(3))).abs() < 1e-12);
    }
}
#[test]
fn create_box_face_sketch_regenerates_in_construction_order_and_cache() {
    let mut d = Design::default();
    let id = Uuid::new_v4();
    let parameters = CreateKind::Box
        .fields()
        .iter()
        .enumerate()
        .map(|(i, f)| d.parameter(&format!("box{i}"), f.default.into()))
        .collect();
    d.create_features.push(CreateFeature {
        id,
        name: "Box".into(),
        kind: CreateKind::Box,
        parameters,
        sketch: None,
        second_sketch: None,
        boundary: vec![],
        target: None,
        second_target: None,
    });
    d.sync_construction();
    let original = mesh(&d);
    let top = original
        .planar_faces
        .iter()
        .find(|f| f.frame.nz > 0.9)
        .unwrap()
        .face;
    attach(&mut d, id, &original, top, ExtrudeOperation::Cut, "2 mm");
    let expected = original.volume - 0.002f64.powi(3);
    let mut cache = EvaluationCache::default();
    for _ in 0..2 {
        assert!(
            (solid::evaluate_cached(&d, || false, &mut cache)
                .unwrap()
                .mesh
                .unwrap()
                .volume
                - expected)
                .abs()
                < 1e-12
        );
    }
    let reopened: Design = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert!((mesh(&reopened).volume - expected).abs() < 1e-12);
    let height = d.create_features[0].parameters[2];
    d.parameters
        .iter_mut()
        .find(|p| p.id == height)
        .unwrap()
        .expression = "30 mm".into();
    let changed = solid::evaluate_cached(&d, || false, &mut cache)
        .unwrap()
        .mesh
        .unwrap();
    assert!((changed.volume - (expected + 0.02 * 0.02 * 0.01)).abs() < 1e-12);
    assert!((changed.planes.last().unwrap().oz - 0.03).abs() < 1e-12);
    assert!((changed.volume - mesh(&d).volume).abs() < 1e-12);
}
#[test]
fn sketch_on_modified_flat_face_uses_the_modified_plane() {
    let (mut d, id) = base();
    d.solid_edits.push(SolidEdit {
        id: Uuid::new_v4(),
        kind: ModifyKind::PressPull,
        target: id,
        tool: None,
        face: 1,
        edge_points: vec![],
        face_reference: Some(format!("{id}/cap/end")),
        tool_reference: None,
        values: [0.002, 0., 0., 0.],
        parameters: [None; 4],
        copy: false,
        mode: 0,
    });
    d.sync_construction();
    let original = mesh(&d);
    let top = original
        .planar_faces
        .iter()
        .find(|f| f.frame.nz > 0.9)
        .unwrap()
        .face;
    attach(&mut d, id, &original, top, ExtrudeOperation::Cut, "2 mm");
    let result = mesh(&d);
    assert!((result.volume - (original.volume - 0.002f64.powi(3))).abs() < 1e-12);
    let active = result.planes.last().unwrap();
    assert!((active.oz - 0.012).abs() < 1e-12);
    let mut cache = EvaluationCache::default();
    for _ in 0..2 {
        assert!(
            (solid::evaluate_cached(&d, || false, &mut cache)
                .unwrap()
                .mesh
                .unwrap()
                .volume
                - result.volume)
                .abs()
                < 1e-12
        );
    }
}
#[test]
fn negative_depth_reverses_extrusion_and_zero_is_rejected() {
    let (mut d, _) = base();
    d.parameters
        .iter_mut()
        .find(|p| p.name == "thickness")
        .unwrap()
        .expression = "-10 mm".into();
    let m = mesh(&d);
    assert!((m.volume - 0.02 * 0.02 * 0.01).abs() < 1e-12);
    assert!(m.vertices.iter().all(|v| v.z <= 1e-12));
    assert!(m.vertices.iter().any(|v| v.z < -0.009));
    d.parameters
        .iter_mut()
        .find(|p| p.name == "thickness")
        .unwrap()
        .expression = "0 mm".into();
    assert!(solid::evaluate(&d, || false).is_err());
}
#[test]
fn forward_face_dependencies_are_rejected() {
    let (mut d, id) = base();
    let m = mesh(&d);
    let face = m.planar_faces[0].face;
    attach(&mut d, id, &m, face, ExtrudeOperation::NewBody, "2 mm");
    let downstream = d.features[0].id;
    if let SketchPlane::NamedFace { support, .. } = &mut d.active_plane {
        *support = downstream;
    }
    assert!(d.validate().is_err());
}

#[test]
fn negative_cut_from_xy_cuts_upward() {
    let (mut d, id) = base();
    let before = mesh(&d).volume;
    let sketch = d.create_sketch(SketchPlane::Xy).unwrap();
    d.rectangle([0.005, 0.005], [0.007, 0.007]);
    let depth = d.parameter("cutDepth", "-2 mm".into());
    d.features.push(ExtrudeFeature {
        id: Uuid::new_v4(),
        name: "Upward cut".into(),
        sketch,
        boundary: vec![],
        depth,
        operation: ExtrudeOperation::Cut,
        target: Some(id),
    });
    d.sync_construction();
    assert!((mesh(&d).volume - (before - 0.002f64.powi(3))).abs() < 1e-12);
}
#[test]
fn cylindrical_face_is_not_a_sketch_plane() {
    let mut d = Design::default();
    confusion::sketch::edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
    let depth = d.parameter("thickness", "10 mm".into());
    let id = Uuid::new_v4();
    d.extrusion = Some(Extrusion {
        id,
        depth,
        boundary: vec![],
    });
    d.sync_construction();
    let original = mesh(&d);
    assert_eq!(original.planar_faces.len(), 2);
    let side = original
        .names
        .iter()
        .find(|n| n.key.contains("/side/"))
        .unwrap()
        .key
        .clone();
    d.create_sketch(SketchPlane::NamedFace {
        support: id,
        reference: side,
    })
    .unwrap();
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("not planar"), "{error}");
}
#[test]
fn flat_face_created_by_split_supports_a_sketch() {
    let (mut d, id) = base();
    d.solid_edits.push(SolidEdit {
        id: Uuid::new_v4(),
        kind: ModifyKind::SplitBody,
        target: id,
        tool: None,
        face: 0,
        edge_points: vec![],
        face_reference: None,
        tool_reference: None,
        values: [0.01, 1., 0., 0.],
        parameters: [None; 4],
        copy: false,
        mode: 0,
    });
    d.sync_construction();
    let before = mesh(&d);
    let face = before
        .planar_faces
        .iter()
        .find(|f| f.frame.nx > 0.9 && (f.frame.ox - 0.01).abs() < 1e-9)
        .unwrap()
        .face;
    attach(&mut d, id, &before, face, ExtrudeOperation::Cut, "2 mm");
    assert!((mesh(&d).volume - (before.volume - 0.002f64.powi(3))).abs() < 1e-12);
}
