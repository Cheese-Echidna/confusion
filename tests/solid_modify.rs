use confusion::{
    document::schema::{Design, Extrusion},
    model::modify::{BodyStyle, ModifyKind, SolidEdit},
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
fn edit(target: Uuid, kind: ModifyKind, face: u32, values: [f64; 4]) -> SolidEdit {
    SolidEdit {
        id: Uuid::new_v4(),
        kind,
        target,
        tool: None,
        face,
        edge_points: vec![],
        face_reference: None,
        tool_reference: None,
        values,
        parameters: [None; 4],
        copy: false,
        mode: 0,
    }
}
#[test]
fn edits_and_materials_round_trip_and_invalid_inputs_are_rejected() {
    let (mut d, id) = base();
    d.solid_edits
        .push(edit(id, ModifyKind::Scale, 0, [1.2, 0., 0., 0.]));
    d.body_styles.push(BodyStyle {
        body: id,
        material: "Steel".into(),
        density: 7850.,
        color: [0.4, 0.5, 0.6],
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("modify.con");
    container::save(&path, &d).unwrap();
    let loaded = container::load(&path).unwrap();
    assert_eq!(loaded.solid_edits[0].values, [1.2, 0., 0., 0.]);
    assert_eq!(loaded.body_styles[0].density, 7850.);
    d.solid_edits[0].values[0] = f64::NAN;
    assert!(d.validate().is_err());
    d.solid_edits[0].values[0] = 1.;
    d.solid_edits[0].target = Uuid::new_v4();
    assert!(d.validate().is_err());
}
#[test]
fn modify_parameter_units_and_removed_body_dependencies_are_checked() {
    let (mut d, id) = base();
    let wrong = d.parameter("length_factor", "2 mm".into());
    let mut e = edit(id, ModifyKind::Scale, 0, [2., 0., 0., 0.]);
    e.parameters[0] = Some(wrong);
    d.solid_edits.push(e);
    assert!(d.validate().unwrap_err().contains("incompatible units"));
    d.solid_edits.clear();
    d.solid_edits.push(edit(id, ModifyKind::Remove, 0, [0.; 4]));
    d.solid_edits
        .push(edit(id, ModifyKind::Scale, 0, [2., 0., 0., 0.]));
    assert!(d.validate().unwrap_err().contains("removed body"));
}
#[test]
fn every_modify_menu_item_has_an_action() {
    let group = confusion::ui::toolbar::groups(confusion::ui::toolbar::Mode::Solid)
        .into_iter()
        .find(|g| g.name == "Modify")
        .unwrap();
    assert_eq!(group.features.len(), 21);
    assert!(group.features.iter().all(|f| f.available()));
}
#[cfg(all(feature = "solver", feature = "kernel"))]
mod exact {
    use super::*;
    use confusion::{evaluation::solid, kernel::bridge::ffi};
    fn mesh(d: &Design) -> ffi::Mesh {
        solid::evaluate(d, || false).unwrap().mesh.unwrap()
    }
    fn top(mesh: &ffi::Mesh) -> u32 {
        mesh.vertices.iter().find(|v| v.nz > 0.99).unwrap().face
    }
    #[test]
    fn scale_move_copy_remove_and_split_regenerate_exact_volume() {
        let (d, id) = base();
        let original = mesh(&d).volume;
        let mut scaled = d.clone();
        scaled
            .solid_edits
            .push(edit(id, ModifyKind::Scale, 0, [2., 0., 0., 0.]));
        assert!((mesh(&scaled).volume / original - 8.).abs() < 1e-6);
        let mut moved = d.clone();
        let mut e = edit(id, ModifyKind::MoveCopy, 0, [0.1, 0., 0., 0.]);
        e.copy = true;
        moved.solid_edits.push(e);
        assert!((mesh(&moved).volume / original - 2.).abs() < 1e-6);
        let mut split = d.clone();
        split
            .solid_edits
            .push(edit(id, ModifyKind::SplitBody, 0, [0.005, 0., 0., 1.]));
        let m = mesh(&split);
        assert!((m.volume - original).abs() < 1e-10);
        assert!(m.faces > 6);
        let mut removed = d;
        removed
            .solid_edits
            .push(edit(id, ModifyKind::Remove, 0, [0.; 4]));
        assert_eq!(mesh(&removed).vertices.len(), 0);
    }
    #[test]
    fn fillet_chamfer_shell_press_pull_and_face_split_are_real_geometry() {
        let (d, id) = base();
        let original = mesh(&d);
        let face = top(&original);
        for kind in [ModifyKind::Fillet, ModifyKind::Chamfer] {
            let mut changed = d.clone();
            changed
                .solid_edits
                .push(edit(id, kind, face, [0.001, 0., 0., 0.]));
            let result = mesh(&changed);
            assert!(result.volume < original.volume);
            assert!(result.faces > original.faces);
        }
        let mut shell = d.clone();
        shell
            .solid_edits
            .push(edit(id, ModifyKind::Shell, face, [0.001, 0., 0., 0.]));
        assert!(mesh(&shell).volume < original.volume * 0.5);
        for kind in [ModifyKind::PressPull, ModifyKind::OffsetFace] {
            let mut changed = d.clone();
            changed
                .solid_edits
                .push(edit(id, kind, face, [0.002, 0., 0., 0.]));
            assert!((mesh(&changed).volume / original.volume - 1.2).abs() < 1e-6);
        }
        let mut split = d.clone();
        split
            .solid_edits
            .push(edit(id, ModifyKind::SplitFace, face, [0.04, 1., 0., 0.]));
        let result = mesh(&split);
        assert!((result.volume - original.volume).abs() < 1e-10);
        assert!(result.faces > original.faces);
        let mut bad = d;
        bad.solid_edits
            .push(edit(id, ModifyKind::Fillet, face, [1., 0., 0., 0.]));
        assert!(solid::evaluate(&bad, || false).is_err());
    }
    fn two_bodies() -> (Design, Uuid, Uuid) {
        use confusion::document::model::{ExtrudeFeature, ExtrudeOperation, SketchPlane};
        let (mut d, base) = base();
        let sketch = d.create_sketch(SketchPlane::Xy).unwrap();
        d.rectangle([0.04, 0.], [0.10, 0.05]);
        let depth = d.parameter("tool_depth", "10 mm".into());
        let tool = Uuid::new_v4();
        d.features.push(ExtrudeFeature {
            id: tool,
            name: "Tool".into(),
            sketch,
            boundary: vec![],
            depth,
            operation: ExtrudeOperation::NewBody,
            target: None,
        });
        d.sync_construction();
        (d, base, tool)
    }
    #[test]
    fn combine_modes_alignment_and_planar_replacement() {
        let (d, id, tool) = two_bodies();
        for (mode, expected) in [(0, 0.00005), (1, 0.00002), (2, 0.00002)] {
            let mut changed = d.clone();
            let mut e = edit(id, ModifyKind::Combine, 0, [0.; 4]);
            e.tool = Some(tool);
            e.mode = mode;
            changed.solid_edits.push(e);
            assert!((mesh(&changed).volume - expected).abs() < 1e-10);
        }
        let mut aligned = d.clone();
        let mut e = edit(id, ModifyKind::Align, 0, [0.; 4]);
        e.tool = Some(tool);
        aligned.solid_edits.push(e);
        let result = mesh(&aligned);
        assert!(result.vertices.iter().all(|v| v.x >= 0.029999));
        let mut replacement = d.clone();
        replacement
            .parameters
            .iter_mut()
            .find(|p| p.name == "tool_depth")
            .unwrap()
            .expression = "20 mm".into();
        let original = mesh(&replacement);
        let face = original
            .vertices
            .iter()
            .find(|v| v.nz > 0.99 && (v.z - 0.01).abs() < 1e-7)
            .unwrap()
            .face;
        let tool_face = original
            .vertices
            .iter()
            .find(|v| v.nz > 0.99 && (v.z - 0.02).abs() < 1e-7)
            .unwrap()
            .face;
        let mut e = edit(
            id,
            ModifyKind::ReplaceFace,
            face,
            [0., tool_face as f64, 0., 0.],
        );
        e.tool = Some(tool);
        replacement.solid_edits.push(e);
        assert!((mesh(&replacement).volume - 0.00014).abs() < 1e-10);
    }
    #[test]
    fn draft_silhouette_and_parameter_edits_regenerate() {
        use confusion::document::model::{ExtrudeFeature, ExtrudeOperation, SketchPlane};
        let (d, id) = base();
        let original = mesh(&d);
        let side = original.vertices.iter().find(|v| v.nx > 0.99).unwrap().face;
        let mut drafted = d.clone();
        drafted
            .solid_edits
            .push(edit(id, ModifyKind::Draft, side, [0.05, 0., 0., 0.]));
        assert!((mesh(&drafted).volume - original.volume).abs() > 1e-9);
        let mut scaled = d.clone();
        let factor = scaled.parameter("factor", "1.5".into());
        scaled.parameters.last_mut().unwrap().scalar = true;
        let mut e = edit(id, ModifyKind::Scale, 0, [1.5, 0., 0., 0.]);
        e.parameters[0] = Some(factor);
        scaled.solid_edits.push(e);
        assert!((mesh(&scaled).volume / original.volume - 3.375).abs() < 1e-6);
        scaled
            .parameters
            .iter_mut()
            .find(|p| p.id == factor)
            .unwrap()
            .expression = "2".into();
        assert!((mesh(&scaled).volume / original.volume - 8.).abs() < 1e-6);
        let mut silhouette = d;
        let sketch = silhouette.create_sketch(SketchPlane::Xy).unwrap();
        silhouette.rectangle([0.01, 0.01], [0.03, 0.03]);
        let depth = silhouette.parameter("projection_depth", "20 mm".into());
        let tool = Uuid::new_v4();
        silhouette.features.push(ExtrudeFeature {
            id: tool,
            name: "Silhouette tool".into(),
            sketch,
            boundary: vec![],
            depth,
            operation: ExtrudeOperation::NewBody,
            target: None,
        });
        silhouette.sync_construction();
        let before = mesh(&silhouette);
        let mut e = edit(id, ModifyKind::SilhouetteSplit, top(&before), [0.; 4]);
        e.tool = Some(tool);
        silhouette.solid_edits.push(e);
        let result = mesh(&silhouette);
        assert!((result.volume - before.volume).abs() < 1e-10);
        assert!(result.faces > before.faces);
    }
    #[test]
    fn simplify_merges_split_faces_and_delete_heals_a_pocket() {
        use confusion::document::model::{CapRole, ExtrudeFeature, ExtrudeOperation, SketchPlane};
        let (mut d, id) = base();
        let original = mesh(&d);
        let face = top(&original);
        let mut simplified = d.clone();
        simplified
            .solid_edits
            .push(edit(id, ModifyKind::SplitFace, face, [0.04, 1., 0., 0.]));
        assert!(mesh(&simplified).faces > 6);
        simplified
            .solid_edits
            .push(edit(id, ModifyKind::Simplify, 0, [0.; 4]));
        assert_eq!(mesh(&simplified).faces, 6);
        let sketch = d
            .create_sketch(SketchPlane::Face {
                support: id,
                producer: id,
                role: CapRole::End,
            })
            .unwrap();
        d.rectangle([0.01, 0.01], [0.03, 0.03]);
        let depth = d.parameter("pocket", "3 mm".into());
        let cut = Uuid::new_v4();
        d.features.push(ExtrudeFeature {
            id: cut,
            name: "Pocket".into(),
            sketch,
            boundary: vec![],
            depth,
            operation: ExtrudeOperation::Cut,
            target: Some(id),
        });
        d.sync_construction();
        let pocket = mesh(&d);
        let bottom = pocket
            .vertices
            .iter()
            .find(|v| (v.z - 0.007).abs() < 1e-7 && v.nz > 0.99)
            .unwrap()
            .face;
        for kind in [ModifyKind::Delete, ModifyKind::Remove, ModifyKind::Simplify] {
            let mut healed = d.clone();
            healed.solid_edits.push(edit(cut, kind, bottom, [0.; 4]));
            let result = mesh(&healed);
            assert!(
                (result.volume - original.volume).abs() < 1e-10,
                "{kind:?}: healed {}, original {}",
                result.volume,
                original.volume
            );
        }
    }
    #[test]
    fn primitive_create_and_modify_share_evaluation_and_timeline_prefix() {
        use confusion::model::solid_create::{CreateFeature, CreateKind};
        let mut d = Design::default();
        let id = Uuid::new_v4();
        let parameters = (0..3)
            .map(|i| d.parameter(&format!("box_{i}"), "20 mm".into()))
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
        let e = edit(id, ModifyKind::Scale, 0, [2., 0., 0., 0.]);
        let eid = e.id;
        d.solid_edits.push(e);
        d.sync_construction();
        assert!((mesh(&d).volume / original.volume - 8.).abs() < 1e-6);
        assert_eq!(d.through_feature(id).unwrap().solid_edits.len(), 0);
        assert_eq!(d.through_feature(eid).unwrap().solid_edits.len(), 1);
    }
}

#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn exact_body_topology_and_selected_edge_fillet_survive_cached_evaluation() {
    use confusion::evaluation::{cache::EvaluationCache, solid};
    let (mut d, id) = base();
    let mut cache = EvaluationCache::default();
    let initial = solid::evaluate_cached(&d, || false, &mut cache)
        .unwrap()
        .mesh
        .unwrap();
    assert_eq!(initial.edges.len(), 12);
    assert_eq!(initial.corners.len(), 8);
    assert!(initial.edges.iter().all(|e| e.faces.len() == 2));
    let edges: Vec<_> = initial
        .edges
        .iter()
        .filter(|e| e.points.iter().all(|p| (p.z - 0.01).abs() < 1e-8))
        .collect();
    // Use a point strictly inside the line, so matching never chooses an adjacent edge.
    let midpoint = |i: usize| {
        let a = &edges[i].points[0];
        let b = edges[i].points.last().unwrap();
        [(a.x + b.x) / 2., (a.y + b.y) / 2., (a.z + b.z) / 2.]
    };
    let mut feature = edit(id, ModifyKind::Fillet, 0, [0.001, 0., 0., 0.]);
    feature.edge_points = vec![midpoint(0)];
    d.solid_edits.push(feature);
    let first = solid::evaluate_cached(&d, || false, &mut cache)
        .unwrap()
        .mesh
        .unwrap();
    assert!(first.volume < initial.volume);
    assert_eq!(first.faces, initial.faces + 1);
    d.solid_edits[0].edge_points = vec![midpoint(1)];
    let second = solid::evaluate_cached(&d, || false, &mut cache)
        .unwrap()
        .mesh
        .unwrap();
    assert_eq!(second.faces, initial.faces + 1);
    let fresh = solid::evaluate(&d, || false).unwrap().mesh.unwrap();
    assert!((second.volume - fresh.volume).abs() < 1e-12);
    assert_eq!(
        second
            .vertices
            .iter()
            .map(|v| (v.x, v.y, v.z))
            .collect::<Vec<_>>(),
        fresh
            .vertices
            .iter()
            .map(|v| (v.x, v.y, v.z))
            .collect::<Vec<_>>()
    );
    d.solid_edits[0].edge_points[0] = [0.5, 0.5, 0.5];
    assert!(
        solid::evaluate(&d, || false)
            .err()
            .unwrap()
            .contains("select it again")
    );
}

#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn face_extrusion_can_create_a_separate_solid() {
    use confusion::evaluation::solid;
    let (mut d, id) = base();
    let initial = solid::evaluate(&d, || false).unwrap().mesh.unwrap();
    let face = initial.vertices.iter().find(|v| v.nz > 0.99).unwrap().face;
    let mut feature = edit(id, ModifyKind::PressPull, face, [0.005, 0., 0., 0.]);
    feature.copy = true;
    d.solid_edits.push(feature);
    let changed = solid::evaluate(&d, || false).unwrap().mesh.unwrap();
    assert!((changed.volume - initial.volume * 1.5).abs() < 1e-10);
    assert_eq!(changed.inspection.solids, 2);
}
