#![cfg(all(feature = "solver", feature = "kernel"))]
use confusion::{
    document::{
        model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
        schema::{Design, Extrusion},
    },
    evaluation::{cache::EvaluationCache, solid},
};
fn design(pockets: usize) -> Design {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.05]);
    let depth = d.parameter("thickness", "10 mm".into());
    let base = uuid::Uuid::new_v4();
    d.extrusion = Some(Extrusion {
        id: base,
        depth,
        boundary: vec![],
    });
    d.sync_construction();
    let mut target = base;
    for n in 0..pockets {
        let sketch = d
            .create_sketch(SketchPlane::Face {
                support: target,
                producer: base,
                role: CapRole::End,
            })
            .unwrap();
        let x = 0.004 + (n % 8) as f64 * 0.009;
        let y = 0.004 + (n / 8) as f64 * 0.012;
        d.rectangle([x, y], [x + 0.005, y + 0.007]);
        let boundary = confusion::sketch::regions::select(
            &d,
            &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(),
            &[],
        )
        .unwrap()
        .boundary;
        let depth = d.parameter(&format!("pocket{n}"), "3 mm".into());
        let id = uuid::Uuid::new_v4();
        d.features.push(ExtrudeFeature {
            id,
            name: format!("Pocket {n}"),
            sketch,
            boundary,
            depth,
            operation: ExtrudeOperation::Cut,
            target: Some(target),
        });
        d.sync_construction();
        target = id;
    }
    d
}

fn equivalent(d: &Design, cache: &mut EvaluationCache) {
    let cached = solid::evaluate_cached(d, || false, cache).unwrap();
    let fresh = solid::evaluate(d, || false).unwrap();
    let a = cached.mesh.unwrap();
    let b = fresh.mesh.unwrap();
    assert!((a.volume - b.volume).abs() < 1e-12);
    assert_eq!(a.faces, b.faces);
    assert_eq!(a.indices, b.indices);
    assert_eq!(a.vertices.len(), b.vertices.len());
    for (a, b) in a.vertices.iter().zip(&b.vertices) {
        assert_eq!(
            (a.x, a.y, a.z, a.nx, a.ny, a.nz, a.face),
            (b.x, b.y, b.z, b.nx, b.ny, b.nz, b.face)
        );
    }
    assert_eq!(a.anchors.len(), b.anchors.len());
    for (a, b) in a.anchors.iter().zip(&b.anchors) {
        assert_eq!(
            (a.face, a.support, a.producer, a.role, a.ambiguous),
            (b.face, b.support, b.producer, b.role, b.ambiguous)
        );
    }
    assert_eq!(a.planes.len(), b.planes.len());
    for (a, b) in a.planes.iter().zip(&b.planes) {
        assert_eq!(
            (a.ox, a.oy, a.oz, a.nx, a.ny, a.nz),
            (b.ox, b.oy, b.oz, b.nx, b.ny, b.nz)
        );
    }
    assert!((a.inspection.area - b.inspection.area).abs() < 1e-12);
    assert_eq!(a.inspection.valid, b.inspection.valid);
}
#[test]
fn late_edit_upstream_edit_undo_and_reopen_match_full_rebuild() {
    let mut d = design(3);
    let original = d.clone();
    let mut cache = EvaluationCache::default();
    equivalent(&d, &mut cache);
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 4);
    assert_eq!(cache.reused_sketches, 4);
    d.parameters
        .iter_mut()
        .find(|p| p.name == "pocket2")
        .unwrap()
        .expression = "4 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 3);
    assert_eq!(cache.reused_sketches, 4);
    d.parameters
        .iter_mut()
        .find(|p| p.name == "thickness")
        .unwrap()
        .expression = "12 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 0);
    equivalent(&original, &mut cache);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cached.con");
    confusion::persistence::container::save(&path, &original).unwrap();
    let reopened = confusion::persistence::container::load(&path).unwrap();
    equivalent(&reopened, &mut cache);
    equivalent(&reopened, &mut EvaluationCache::default());
}
#[test]
fn failed_and_cancelled_evaluations_do_not_poison_cache() {
    let mut d = design(1);
    let original = d.clone();
    let mut cache = EvaluationCache::default();
    equivalent(&d, &mut cache);
    d.parameters
        .iter_mut()
        .find(|p| p.name == "pocket0")
        .unwrap()
        .expression = "-3 mm".into();
    assert!(solid::evaluate_cached(&d, || false, &mut cache).is_err());
    assert!(solid::evaluate_cached(&original, || true, &mut cache).is_err());
    equivalent(&original, &mut cache);
}

fn add(
    d: &mut Design,
    kind: confusion::model::solid_create::CreateKind,
    values: &[&str],
    target: Option<uuid::Uuid>,
) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let parameters = kind
        .fields()
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let pid = d.parameter(
                &format!("p_{}_{}", id.simple(), i),
                values.get(i).copied().unwrap_or(f.default).into(),
            );
            let p = d.parameters.iter_mut().find(|p| p.id == pid).unwrap();
            p.angular = f.unit == confusion::model::solid_create::Unit::Angle;
            p.scalar = f.unit == confusion::model::solid_create::Unit::Number;
            pid
        })
        .collect();
    let sketch = if kind.uses_profile() {
        Some(d.ensure_sketch())
    } else {
        None
    };
    d.create_features
        .push(confusion::model::solid_create::CreateFeature {
            id,
            name: kind.name().into(),
            kind,
            parameters,
            sketch,
            second_sketch: None,
            boundary: vec![],
            target,
            second_target: None,
        });
    d.sync_construction();
    id
}

#[test]
fn create_and_modify_checkpoints_match_full_rebuild() {
    use confusion::model::{
        modify::{ModifyKind, SolidEdit},
        solid_create::CreateKind,
    };
    let mut d = Design::default();
    let body = add(&mut d, CreateKind::Box, &[], None);
    let mut cache = EvaluationCache::default();
    d.solid_edits.push(SolidEdit {
        id: uuid::Uuid::new_v4(),
        kind: ModifyKind::Scale,
        target: body,
        tool: None,
        face: 0,
        face_reference: None,
        tool_reference: None,
        values: [1.2, 0., 0., 0.],
        parameters: [None; 4],
        copy: false,
        mode: 0,
    });
    equivalent(&d, &mut cache);
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 2);
    d.solid_edits[0].values[0] = 1.4;
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 1);
    let mut second = d.solid_edits[0].clone();
    second.id = uuid::Uuid::new_v4();
    second.values[0] = 1.1;
    d.solid_edits.push(second);
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 2);
    d.solid_edits[1].values[0] = 1.15;
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 2);
    d.solid_edits[0].values[0] = 1.2;
    equivalent(&d, &mut cache);
    let parameter = d.create_features[0].parameters[0];
    d.parameters
        .iter_mut()
        .find(|p| p.id == parameter)
        .unwrap()
        .expression = "25 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 0);
}
#[test]
fn unrelated_extrusion_branch_is_reused() {
    let mut d = design(1);
    let sketch = d.create_sketch(SketchPlane::Xy).unwrap();
    d.rectangle([0.1, 0.], [0.12, 0.02]);
    let depth = d.parameter("other_depth", "5 mm".into());
    d.features.push(ExtrudeFeature {
        id: uuid::Uuid::new_v4(),
        name: "Independent".into(),
        sketch,
        depth,
        boundary: vec![],
        operation: ExtrudeOperation::NewBody,
        target: None,
    });
    d.sync_construction();
    let mut cache = EvaluationCache::default();
    equivalent(&d, &mut cache);
    d.parameters
        .iter_mut()
        .find(|p| p.name == "pocket0")
        .unwrap()
        .expression = "4 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 2);
}

#[test]
fn changed_driving_dimension_resolves_only_its_sketch() {
    let mut d = design(2);
    let mut cache = EvaluationCache::default();
    equivalent(&d, &mut cache);
    let base_sketch = d
        .construction
        .iter()
        .find(|f| {
            matches!(
                f.kind,
                confusion::document::schema::ConstructionKind::Sketch
            )
        })
        .unwrap()
        .id;
    let input = d.sketch_input(base_sketch).unwrap();
    let parameter = input
        .constraints
        .iter()
        .find_map(|c| c.kind.parameter())
        .unwrap();
    d.parameters
        .iter_mut()
        .find(|p| p.id == parameter)
        .unwrap()
        .expression = "90 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_sketches, 2);
    assert_eq!(cache.reused_features(), 0);
}

#[test]
fn unrelated_create_body_is_reused() {
    use confusion::model::solid_create::CreateKind;
    let mut d = Design::default();
    let body = add(&mut d, CreateKind::Box, &[], None);
    add(
        &mut d,
        CreateKind::Hole,
        &["3 mm", "20 mm", "10 mm", "10 mm", "0 mm"],
        Some(body),
    );
    add(&mut d, CreateKind::Sphere, &[], None);
    let mut cache = EvaluationCache::default();
    equivalent(&d, &mut cache);
    let parameter = d.create_features[0].parameters[0];
    d.parameters
        .iter_mut()
        .find(|p| p.id == parameter)
        .unwrap()
        .expression = "25 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 1);
    let parameter = d.create_features[2].parameters[0];
    d.parameters
        .iter_mut()
        .find(|p| p.id == parameter)
        .unwrap()
        .expression = "12 mm".into();
    equivalent(&d, &mut cache);
    assert_eq!(cache.reused_features(), 2);
}
