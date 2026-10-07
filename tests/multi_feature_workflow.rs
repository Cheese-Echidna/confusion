use confusion::document::{
    model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane},
    schema::{Design, Extrusion},
};
use confusion::{persistence::container, sketch::regions};
use uuid::Uuid;
fn base_design() -> (Design, Uuid) {
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
fn add_rectangle(
    d: &mut Design,
    plane: SketchPlane,
    operation: ExtrudeOperation,
    target: Option<Uuid>,
    dimension: (&str, &str),
    a: [f64; 2],
    b: [f64; 2],
) -> Uuid {
    let (name, depth) = dimension;
    let sketch = d.create_sketch(plane).unwrap();
    d.rectangle(a, b);
    let boundary = regions::select(d, &d.points.iter().map(|p| p.xy).collect::<Vec<_>>(), &[])
        .unwrap()
        .boundary;
    let depth = d.parameter(name, depth.into());
    let id = Uuid::new_v4();
    d.features.push(ExtrudeFeature {
        id,
        name: name.into(),
        sketch,
        boundary,
        depth,
        operation,
        target,
    });
    d.sync_construction();
    d.validate().unwrap();
    id
}
#[test]
fn sketches_switch_without_losing_geometry_and_legacy_round_trip() {
    let (mut d, base) = base_design();
    let first = d.current_sketch_id().unwrap();
    let next = d
        .create_sketch(SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        })
        .unwrap();
    d.rectangle([0.01, 0.01], [0.03, 0.02]);
    assert_eq!(d.sketch_input(first).unwrap().points[2].xy, [0.08, 0.05]);
    d.activate_sketch(first).unwrap();
    assert_eq!(d.points[2].xy, [0.08, 0.05]);
    d.activate_sketch(next).unwrap();
    assert_eq!(d.points[2].xy, [0.03, 0.02]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("model.con");
    container::save(&path, &d).unwrap();
    let d = container::load(&path).unwrap();
    assert_eq!(d.current_sketch_id(), Some(next));
    assert_eq!(d.sketches.len(), 1);
    assert_eq!(d.sketch_input(first).unwrap().points.len(), 4);
}
#[test]
fn invalid_dependencies_and_cycles_are_rejected() {
    let (mut d, base) = base_design();
    let cut = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Cut,
        Some(base),
        ("pocketDepth", "3 mm"),
        [0.01, 0.01],
        [0.03, 0.02],
    );
    d.features[0].target = Some(cut);
    assert!(d.validate().is_err());
    d.features[0].target = Some(base);
    d.active_plane = SketchPlane::Face {
        support: cut,
        producer: base,
        role: CapRole::End,
    };
    assert!(d.validate().is_err());
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn face_pocket_follows_thickness_width_and_reopens() {
    use confusion::evaluation::solid;
    let (mut d, base) = base_design();
    let first = d.current_sketch_id().unwrap();
    add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Cut,
        Some(base),
        ("pocketDepth", "3 mm"),
        [0.01, 0.01],
        [0.03, 0.02],
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pocket.con");
    for (width, thickness) in [(0.08, 0.01), (0.10, 0.015), (0.09, 0.012)] {
        d.parameters
            .iter_mut()
            .find(|p| p.name == "width")
            .unwrap()
            .expression = format!("{} mm", width * 1000.);
        d.parameters
            .iter_mut()
            .find(|p| p.name == "thickness")
            .unwrap()
            .expression = format!("{} mm", thickness * 1000.);
        container::save(&path, &d).unwrap();
        d = container::load(&path).unwrap();
        let model = solid::evaluate(&d, || false).unwrap();
        let mesh = model.mesh.unwrap();
        let expected = width * 0.05 * thickness - 0.02 * 0.01 * 0.003;
        assert!(
            (mesh.volume - expected).abs() < 1e-11,
            "{} != {expected}",
            mesh.volume
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|v| (v.z - (thickness - 0.003)).abs() < 1e-8)
        );
        assert!(
            mesh.anchors
                .iter()
                .any(|a| a.producer == 0 && a.role == 2 && !a.ambiguous)
        );
    }
    d.activate_sketch(first).unwrap();
    let mesh = solid::evaluate(&d, || false).unwrap().mesh.unwrap();
    assert!(mesh.volume > 0.);
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn join_and_independent_new_body_have_exact_volumes() {
    use confusion::evaluation::solid;
    let (mut d, base) = base_design();
    let joined = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Join,
        Some(base),
        ("bossDepth", "4 mm"),
        [0.01, 0.01],
        [0.03, 0.02],
    );
    let mesh = solid::evaluate(&d, || false).unwrap().mesh.unwrap();
    assert!((mesh.volume - (0.08 * 0.05 * 0.01 + 0.02 * 0.01 * 0.004)).abs() < 1e-11);
    add_rectangle(
        &mut d,
        SketchPlane::Xy,
        ExtrudeOperation::NewBody,
        None,
        ("secondDepth", "5 mm"),
        [0.10, 0.],
        [0.12, 0.02],
    );
    let mesh = solid::evaluate(&d, || false).unwrap().mesh.unwrap();
    assert!(
        (mesh.volume - (0.08 * 0.05 * 0.01 + 0.02 * 0.01 * 0.004 + 0.02 * 0.02 * 0.005)).abs()
            < 1e-11
    );
    assert!(mesh.anchors.iter().any(|a| a.support == 1));
    assert!(mesh.anchors.iter().any(|a| a.support == 2));
    assert_eq!(d.features[0].id, joined);
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn split_support_reports_repair_instead_of_choosing_a_face() {
    use confusion::evaluation::solid;
    let (mut d, base) = base_design();
    let groove = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Cut,
        Some(base),
        ("grooveDepth", "3 mm"),
        [0.03, -0.005],
        [0.04, 0.055],
    );
    let second = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: groove,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Cut,
        Some(groove),
        ("nextDepth", "1 mm"),
        [0.01, 0.01],
        [0.02, 0.02],
    );
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("split"), "{error}");
    let preview = d.through_feature(groove).unwrap();
    assert!(!preview.features.iter().any(|f| f.id == second));
    assert!(solid::evaluate(&preview, || false).is_ok());
}
#[test]
fn malformed_active_sketch_and_missing_construction_fail_without_panicking() {
    let (mut d, _) = base_design();
    d.construction.clear();
    d.active_sketch = Some(Uuid::new_v4());
    assert!(d.validate().is_err());
    let dir = tempfile::tempdir().unwrap();
    assert!(container::save(&dir.path().join("bad.con"), &d).is_err());
    let (mut d, _) = base_design();
    let id = d.create_sketch(SketchPlane::Xy).unwrap();
    d.construction.retain(|f| f.id != id);
    assert!(d.validate().is_err());
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn cut_from_bottom_cap_points_inward() {
    let (mut d, base) = base_design();
    add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::Start,
        },
        ExtrudeOperation::Cut,
        Some(base),
        ("bottomPocket", "3 mm"),
        [0.01, -0.02],
        [0.03, -0.01],
    );
    let mesh = confusion::evaluation::solid::evaluate(&d, || false)
        .unwrap()
        .mesh
        .unwrap();
    assert!((mesh.volume - (0.08 * 0.05 * 0.01 - 0.02 * 0.01 * 0.003)).abs() < 1e-11);
    assert!(mesh.vertices.iter().any(|v| (v.z - 0.003).abs() < 1e-8));
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn unused_sketch_attachment_is_validated_and_cancellation_is_observed() {
    use confusion::evaluation::solid;
    let (mut d, base) = base_design();
    let groove = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Cut,
        Some(base),
        ("groove", "3 mm"),
        [0.03, -0.005],
        [0.04, 0.055],
    );
    d.create_sketch(SketchPlane::Face {
        support: groove,
        producer: base,
        role: CapRole::End,
    })
    .unwrap();
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("split"), "{error}");
    assert!(
        solid::evaluate(&d, || true)
            .err()
            .unwrap()
            .contains("superseded")
    );
}
#[test]
fn timeline_prefix_owns_the_correct_sketch_without_changing_the_document() {
    let (mut d, base) = base_design();
    let first = d.current_sketch_id().unwrap();
    let next = d
        .create_sketch(SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        })
        .unwrap();
    d.rectangle([0.01, 0.01], [0.03, 0.02]);
    let preview = d.through_feature(first).unwrap();
    assert_eq!(preview.current_sketch_id(), Some(first));
    assert_eq!(preview.points[2].xy, [0.08, 0.05]);
    assert!(preview.extrusion.is_none());
    preview.validate().unwrap();
    assert_eq!(d.current_sketch_id(), Some(next));
    assert_eq!(d.points[2].xy, [0.03, 0.02]);
    let preview = d.through_feature(base).unwrap();
    assert_eq!(preview.current_sketch_id(), Some(first));
    assert!(preview.extrusion.is_some());
    preview.validate().unwrap();
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn deleted_cap_and_no_op_cut_report_failures() {
    use confusion::evaluation::solid;
    let (mut d, base) = base_design();
    let joined = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Join,
        Some(base),
        ("fullBoss", "4 mm"),
        [0., 0.],
        [0.08, 0.05],
    );
    d.create_sketch(SketchPlane::Face {
        support: joined,
        producer: base,
        role: CapRole::End,
    })
    .unwrap();
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("deleted"), "{error}");
    let (mut d, base) = base_design();
    add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::Cut,
        Some(base),
        ("outsideCut", "3 mm"),
        [0.10, 0.],
        [0.12, 0.02],
    );
    let error = solid::evaluate(&d, || false).err().unwrap();
    assert!(error.contains("does not change"), "{error}");
}
#[test]
fn failed_sketch_creation_preserves_current_intent() {
    let (mut d, base) = base_design();
    let before = serde_json::to_value(&d).unwrap();
    assert!(
        d.create_sketch(SketchPlane::Face {
            support: Uuid::new_v4(),
            producer: base,
            role: CapRole::End
        })
        .is_err()
    );
    assert_eq!(serde_json::to_value(d).unwrap(), before);
}

#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn cut_and_new_body_preserves_removed_material_and_world_planes() {
    let (mut d, base) = base_design();
    let cut = add_rectangle(
        &mut d,
        SketchPlane::Face {
            support: base,
            producer: base,
            role: CapRole::End,
        },
        ExtrudeOperation::CutNewBody,
        Some(base),
        ("pocketDepth", "3 mm"),
        [0.01, 0.01],
        [0.03, 0.02],
    );
    let model = confusion::evaluation::solid::evaluate(&d, || false).unwrap();
    let mesh = model.mesh.unwrap();
    assert!((mesh.volume - 0.08 * 0.05 * 0.01).abs() < 1e-11);
    assert_eq!(mesh.inspection.solids, 2);
    assert!(mesh.bodies.iter().any(|f| f.body == 1 && f.part == 0));
    assert!(mesh.bodies.iter().any(|f| f.body == 1 && f.part == 1));
    assert_eq!(model.features[1], cut);
    assert_eq!(mesh.planes.len(), 2);
    assert!((mesh.planes[1].oz - 0.01).abs() < 1e-10);
    assert_eq!(model.sketches.len(), 2);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cut-piece.con");
    container::save(&path, &d).unwrap();
    let reopened = container::load(&path).unwrap();
    assert_eq!(reopened.features[0].operation, ExtrudeOperation::CutNewBody);
}
