use confusion::{document::schema::Design, exchange::step::export_design};

#[test]
fn empty_export_preserves_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unchanged.step");
    std::fs::write(&path, b"existing").unwrap();
    assert!(export_design(&path, &Design::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"existing");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[cfg(all(feature = "solver", feature = "kernel"))]
mod native {
    use super::*;
    use confusion::{
        kernel::bridge::ffi,
        model::solid_create::{CreateFeature, CreateKind, Unit},
    };
    use uuid::Uuid;
    fn add(d: &mut Design, kind: CreateKind, values: &[&str], target: Option<Uuid>) -> Uuid {
        let id = Uuid::new_v4();
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
                p.angular = f.unit == Unit::Angle;
                p.scalar = f.unit == Unit::Number;
                pid
            })
            .collect();
        d.create_features.push(CreateFeature {
            id,
            name: kind.name().into(),
            kind,
            parameters,
            sketch: None,
            second_sketch: None,
            boundary: vec![],
            target,
            second_target: None,
        });
        d.sync_construction();
        id
    }
    fn roundtrip(d: &Design, count: u32, volume: f64) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("模型.step");
        std::fs::write(&path, b"old").unwrap();
        let report = export_design(&path, d).unwrap();
        assert_eq!(report.solids, count);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("ISO-10303-21;"));
        assert!(text.contains("MANIFOLD_SOLID_BREP"));
        assert!(text.contains(".MILLI.,.METRE."));
        assert!(!text.contains("FACETED_BREP"));
        let mesh = ffi::inspect_step(path.to_str().unwrap()).unwrap();
        assert!(mesh.inspection.valid);
        assert_eq!(mesh.inspection.solids, count);
        assert!((mesh.volume - volume).abs() < volume * 1e-5);
        assert!((report.volume_m3 - volume).abs() < volume * 1e-5);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn exact_curved_solid_and_units_roundtrip() {
        let mut d = Design::default();
        add(&mut d, CreateKind::Cylinder, &["10 mm", "20 mm"], None);
        roundtrip(&d, 1, std::f64::consts::PI * 0.01f64.powi(2) * 0.02);
    }
    #[test]
    fn consumed_boolean_target_is_excluded() {
        let mut d = Design::default();
        let id = add(&mut d, CreateKind::Box, &[], None);
        add(
            &mut d,
            CreateKind::Hole,
            &["3 mm", "20 mm", "10 mm", "10 mm", "0 mm"],
            Some(id),
        );
        roundtrip(
            &d,
            1,
            0.02f64.powi(3) - std::f64::consts::PI * 0.003f64.powi(2) * 0.02,
        );
    }
    #[test]
    fn pattern_exports_all_final_solids() {
        let mut d = Design::default();
        let id = add(&mut d, CreateKind::Box, &[], None);
        add(
            &mut d,
            CreateKind::RectangularPattern,
            &["3", "2", "30 mm", "30 mm"],
            Some(id),
        );
        roundtrip(&d, 6, 6. * 0.02f64.powi(3));
    }
    #[test]
    fn extrusion_and_final_scale_edit_roundtrip() {
        use confusion::{
            document::schema::Extrusion,
            model::modify::{ModifyKind, SolidEdit},
        };
        let mut d = Design::default();
        d.rectangle([0., 0.], [0.08, 0.05]);
        let depth = d.parameter("depth", "10 mm".into());
        let id = Uuid::new_v4();
        d.extrusion = Some(Extrusion {
            id,
            depth,
            boundary: vec![],
        });
        d.sync_construction();
        d.solid_edits.push(SolidEdit {
            id: Uuid::new_v4(),
            kind: ModifyKind::Scale,
            target: id,
            tool: None,
            face: 0,
            values: [2., 0., 0., 0.],
            parameters: [None; 4],
            copy: false,
            mode: 0,
        });
        roundtrip(&d, 1, 0.08 * 0.05 * 0.01 * 8.);
    }
    #[test]
    fn conflicting_unused_sketch_blocks_export() {
        use confusion::document::schema::ConstraintKind;
        let mut d = Design::default();
        d.rectangle([0., 0.], [0.08, 0.04]);
        let width = d.parameter("contradictory_width", "60 mm".into());
        d.constrain(ConstraintKind::DistanceX {
            points: [d.points[0].id, d.points[1].id],
            parameter: width,
        });
        add(&mut d, CreateKind::Box, &[], None);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conflict.step");
        assert!(export_design(&path, &d).unwrap_err().contains("conflict"));
        assert!(!path.exists());
    }
    #[test]
    fn persistence_failure_cleans_up_temporary_file() {
        let mut d = Design::default();
        add(&mut d, CreateKind::Box, &[], None);
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("directory.step");
        std::fs::create_dir(&destination).unwrap();
        assert!(export_design(&destination, &d).is_err());
        assert!(destination.is_dir());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn fresh_evaluation_rejects_invalid_parameter_and_preserves_file() {
        let mut d = Design::default();
        add(&mut d, CreateKind::Box, &[], None);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("part.step");
        export_design(&path, &d).unwrap();
        let before = std::fs::read(&path).unwrap();
        d.parameters[0].expression = "-1 mm".into();
        assert!(export_design(&path, &d).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
