#![cfg(all(feature = "solver", feature = "kernel"))]
use confusion::{
    document::schema::{Design, Extrusion},
    evaluation::{cache::EvaluationCache, solid},
    model::modify::{ModifyKind, SolidEdit},
    persistence::container,
};
use uuid::Uuid;
fn base() -> (Design, Uuid) {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.05]);
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
fn mesh(d: &Design) -> confusion::kernel::bridge::ffi::Mesh {
    solid::evaluate(d, || false).unwrap().mesh.unwrap()
}
fn edit(target: Uuid, kind: ModifyKind, key: String, values: [f64; 4]) -> SolidEdit {
    SolidEdit {
        id: Uuid::new_v4(),
        kind,
        target,
        tool: None,
        face: 99999,
        edge_points: vec![],
        face_reference: Some(key),
        tool_reference: None,
        values,
        parameters: [None; 4],
        copy: false,
        mode: 0,
    }
}
#[test]
fn named_caps_and_sides_survive_resize_reopen_and_cache() {
    let (mut d, id) = base();
    let original = mesh(&d);
    assert_eq!(original.names.iter().filter(|n| !n.ambiguous).count(), 6);
    let key = format!("{id}/cap/end");
    assert!(original.names.iter().any(|n| n.key == key && !n.ambiguous));
    d.solid_edits.push(edit(
        id,
        ModifyKind::PressPull,
        key.clone(),
        [0.002, 0., 0., 0.],
    ));
    assert!((mesh(&d).volume - 0.08 * 0.05 * 0.012).abs() < 1e-12);
    let saved = d.clone();
    d.parameters
        .iter_mut()
        .find(|p| p.name == "thickness")
        .unwrap()
        .expression = "20 mm".into();
    assert!((mesh(&d).volume - 0.08 * 0.05 * 0.022).abs() < 1e-12);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("named.con");
    container::save(&path, &d).unwrap();
    let reopened = container::load(&path).unwrap();
    assert_eq!(
        reopened.solid_edits[0].face_reference.as_deref(),
        Some(key.as_str())
    );
    let mut cache = EvaluationCache::default();
    for design in [&saved, &d, &reopened, &saved] {
        let full = mesh(design);
        let cached = solid::evaluate_cached(design, || false, &mut cache)
            .unwrap()
            .mesh
            .unwrap();
        assert!((full.volume - cached.volume).abs() < 1e-12);
        assert_eq!(
            full.names
                .iter()
                .map(|n| (&n.key, n.ambiguous))
                .collect::<Vec<_>>(),
            cached
                .names
                .iter()
                .map(|n| (&n.key, n.ambiguous))
                .collect::<Vec<_>>()
        );
    }
}
#[test]
fn split_reference_reports_ambiguity_and_deleted_reference_reports_repair() {
    let (mut d, id) = base();
    let key = format!("{id}/cap/end");
    let mut split = edit(id, ModifyKind::SplitFace, key.clone(), [0.04, 1., 0., 0.]);
    split.face = 1;
    d.solid_edits.push(split);
    let split_mesh = mesh(&d);
    assert!(split_mesh.names.iter().any(|n| n.key == key && n.ambiguous));
    d.solid_edits
        .push(edit(id, ModifyKind::PressPull, key, [0.001, 0., 0., 0.]));
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("ambiguously"), "{error}");
    let branch = split_mesh
        .names
        .iter()
        .find(|name| {
            !name.ambiguous && name.key.contains("/branch/") && name.key.ends_with("positive")
        })
        .unwrap()
        .key
        .clone();
    d.solid_edits.last_mut().unwrap().face_reference = Some(branch);
    assert!(mesh(&d).volume > split_mesh.volume);
    let (mut d, id) = base();
    d.solid_edits.push(edit(
        id,
        ModifyKind::PressPull,
        "missing/face".into(),
        [0.001, 0., 0., 0.],
    ));
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("reselect"), "{error}");
}
#[test]
fn fillet_and_shell_keep_unrelated_named_faces() {
    for kind in [ModifyKind::Fillet, ModifyKind::Shell] {
        let (mut d, id) = base();
        d.solid_edits
            .push(edit(id, kind, format!("{id}/cap/end"), [0.001, 0., 0., 0.]));
        let evaluated = mesh(&d);
        let bottom = format!("{id}/cap/start");
        assert!(
            evaluated
                .names
                .iter()
                .any(|n| n.key == bottom && !n.ambiguous),
            "{kind:?}"
        );
        d.solid_edits
            .push(edit(id, ModifyKind::PressPull, bottom, [0.001, 0., 0., 0.]));
        assert!(mesh(&d).volume > 0.);
    }
}

#[test]
fn legacy_ordinals_bind_once_to_persisted_provenance() {
    let (mut d, id) = base();
    let original = mesh(&d);
    let top = original
        .names
        .iter()
        .find(|name| name.key == format!("{id}/cap/end"))
        .unwrap()
        .face;
    let mut legacy = edit(
        id,
        ModifyKind::PressPull,
        String::new(),
        [0.002, 0., 0., 0.],
    );
    legacy.face_reference = None;
    legacy.face = top;
    d.solid_edits.push(legacy);
    let evaluated = mesh(&d);
    assert!(confusion::model::naming::bind_legacy_references(
        &mut d,
        &evaluated.references
    ));
    assert!(!confusion::model::naming::bind_legacy_references(
        &mut d,
        &evaluated.references
    ));
    d.solid_edits[0].face = 99999;
    d.parameters
        .iter_mut()
        .find(|p| p.name == "thickness")
        .unwrap()
        .expression = "20 mm".into();
    assert!((mesh(&d).volume - 0.08 * 0.05 * 0.022).abs() < 1e-12);
}
