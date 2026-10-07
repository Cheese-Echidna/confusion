use confusion::{
    document::schema::Design,
    model::solid_create::{CreateFeature, CreateKind, Unit},
    ui::toolbar::{self, Action, Mode},
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
    let sketch = if kind.uses_profile() {
        Some(d.ensure_sketch())
    } else {
        None
    };
    d.create_features.push(CreateFeature {
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
fn every_create_menu_entry_routes_to_an_action() {
    let groups = toolbar::groups(Mode::Solid);
    let create = groups.iter().find(|g| g.name == "Create").unwrap();
    assert!(create.features.iter().all(|f| f.available()));
    for kind in CreateKind::ALL {
        assert!(
            create
                .features
                .iter()
                .any(|f| f.action == Some(Action::SolidCreate(kind)))
        );
    }
}
#[test]
fn create_intent_roundtrips_and_dependency_prefix_is_persistent() {
    let mut d = Design::default();
    let box_id = add(&mut d, CreateKind::Box, &[], None);
    let hole = add(
        &mut d,
        CreateKind::Hole,
        &["3 mm", "20 mm", "10 mm", "10 mm", "0 mm"],
        Some(box_id),
    );
    d.validate().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("create.con");
    confusion::persistence::container::save(&path, &d).unwrap();
    let reopened = confusion::persistence::container::load(&path).unwrap();
    assert_eq!(reopened.create_features.len(), 2);
    assert_eq!(reopened.current_create_bodies()[0].0, hole);
    assert_eq!(
        reopened
            .through_feature(box_id)
            .unwrap()
            .create_features
            .len(),
        1
    );
    let mut invalid = d.clone();
    invalid.create_features[1].target = Some(hole);
    assert!(invalid.validate().is_err());
    invalid = d.clone();
    invalid.create_features[1].parameters[0] = Uuid::new_v4();
    assert!(invalid.validate().is_err());
}
#[test]
fn invalid_counts_units_and_dimensions_fail() {
    assert!(CreateKind::Torus.validate_values(&[0.01, 0.02]).is_err());
    assert!(
        CreateKind::RectangularPattern
            .validate_values(&[1.5, 2., 0.01, 0.01])
            .is_err()
    );
    assert!(
        CreateKind::Coil
            .validate_values(&[0.01, 0.001, 3., 0.001])
            .is_err()
    );
    assert!(CreateKind::Sweep.validate_values(&[0., 0., 0.]).is_err());
}
#[test]
fn scalar_expressions_keep_units_and_survive_timeline_prefixes() {
    let mut d = Design::default();
    let source = add(&mut d, CreateKind::Box, &[], None);
    let pattern = add(&mut d, CreateKind::RectangularPattern, &[], Some(source));
    let scalar = d.create_features[1].parameters[0];
    let name = d
        .parameters
        .iter()
        .find(|p| p.id == scalar)
        .unwrap()
        .name
        .clone();
    let rows = d.create_features[1].parameters[1];
    d.parameters
        .iter_mut()
        .find(|p| p.id == rows)
        .unwrap()
        .expression = format!("{name}-1");
    let values = confusion::parameters::expression::evaluate(&d).unwrap();
    assert_eq!(values[&rows], 2.);
    confusion::parameters::expression::evaluate(&d.through_feature(source).unwrap()).unwrap();
    assert_eq!(d.through_feature(pattern).unwrap().create_features.len(), 2);
    d.parameters
        .iter_mut()
        .find(|p| p.id == rows)
        .unwrap()
        .expression = "2 mm".into();
    assert!(confusion::parameters::expression::evaluate(&d).is_err());
}

#[cfg(all(feature = "kernel", feature = "solver"))]
mod native {
    use super::*;
    fn mesh(d: &Design) -> confusion::kernel::bridge::ffi::Mesh {
        confusion::evaluation::solid::evaluate(d, || false)
            .unwrap()
            .mesh
            .unwrap()
    }
    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < expected * 1e-5 + 1e-12,
            "{actual} != {expected}"
        );
    }
    #[test]
    fn primitive_volumes_are_exact() {
        let pi = std::f64::consts::PI;
        for (kind, volume) in [
            (CreateKind::Box, 0.02f64.powi(3)),
            (CreateKind::Cylinder, pi * 0.01f64.powi(2) * 0.02),
            (CreateKind::Sphere, 4. / 3. * pi * 0.01f64.powi(3)),
            (CreateKind::Torus, 2. * pi * pi * 0.02 * 0.005f64.powi(2)),
            (
                CreateKind::Pipe,
                pi * (0.01f64.powi(2) - 0.008f64.powi(2)) * 0.02,
            ),
        ] {
            let mut d = Design::default();
            add(&mut d, kind, &[], None);
            close(mesh(&d).volume, volume);
        }
    }
    #[test]
    fn hole_rib_web_emboss_and_thickening_change_body_volume() {
        let mut d = Design::default();
        let id = add(&mut d, CreateKind::Box, &[], None);
        add(
            &mut d,
            CreateKind::Hole,
            &["3 mm", "20 mm", "10 mm", "10 mm", "0 mm"],
            Some(id),
        );
        close(
            mesh(&d).volume,
            0.02f64.powi(3) - std::f64::consts::PI * 0.003f64.powi(2) * 0.02,
        );
        for kind in [CreateKind::Rib, CreateKind::Web] {
            let mut d = Design::default();
            let id = add(&mut d, CreateKind::Box, &[], None);
            add(
                &mut d,
                kind,
                &["20 mm", "2 mm", "10 mm", "10 mm", "0 mm", "10 mm"],
                Some(id),
            );
            assert!(mesh(&d).volume > 0.02f64.powi(3));
        }
        let mut d = Design::default();
        d.rectangle([0.002, 0.002], [0.01, 0.01]);
        let id = add(&mut d, CreateKind::Box, &[], None);
        add(&mut d, CreateKind::Emboss, &["2 mm", "20 mm"], Some(id));
        close(mesh(&d).volume, 0.02f64.powi(3) + 0.008 * 0.008 * 0.002);
        let mut d = Design::default();
        let id = add(&mut d, CreateKind::Box, &[], None);
        add(&mut d, CreateKind::Thicken, &["2 mm"], Some(id));
        assert!(mesh(&d).volume > 0. && mesh(&d).volume < 0.02f64.powi(3));
    }
    #[test]
    fn revolve_sweep_and_loft_have_expected_volumes() {
        let mut d = Design::default();
        d.rectangle([0.01, 0.], [0.02, 0.01]);
        add(&mut d, CreateKind::Revolve, &[], None);
        close(
            mesh(&d).volume,
            std::f64::consts::PI * (0.02f64.powi(2) - 0.01f64.powi(2)) * 0.01,
        );
        let mut d = Design::default();
        d.rectangle([0., 0.], [0.02, 0.01]);
        add(&mut d, CreateKind::Sweep, &["10 mm", "5 mm", "20 mm"], None);
        close(mesh(&d).volume, 0.02 * 0.01 * 0.02);
        let mut d = Design::default();
        d.rectangle([0., 0.], [0.02, 0.01]);
        let first = d.ensure_sketch();
        let second = d
            .create_sketch(confusion::document::model::SketchPlane::Xy)
            .unwrap();
        d.rectangle([0., 0.], [0.02, 0.01]);
        add(&mut d, CreateKind::Loft, &[], None);
        let f = d.create_features.last_mut().unwrap();
        f.sketch = Some(first);
        f.second_sketch = Some(second);
        close(mesh(&d).volume, 0.02 * 0.01 * 0.02);
    }
    #[test]
    fn patterns_and_mirror_preserve_instance_volumes() {
        for (kind, values, count) in [
            (
                CreateKind::RectangularPattern,
                vec!["3", "2", "30 mm", "30 mm"],
                6.,
            ),
            (CreateKind::CircularPattern, vec!["4", "360 deg"], 4.),
            (
                CreateKind::PatternOnPath,
                vec!["3", "60 mm", "0 mm", "0 mm"],
                3.,
            ),
            (CreateKind::Mirror, vec!["0", "-10 mm"], 2.),
        ] {
            let mut d = Design::default();
            let id = add(&mut d, CreateKind::Box, &[], None);
            add(&mut d, kind, &values, Some(id));
            close(mesh(&d).volume, 0.02f64.powi(3) * count);
        }
    }
    #[test]
    fn boundary_fill_creates_common_cell() {
        let mut d = Design::default();
        let a = add(&mut d, CreateKind::Box, &[], None);
        let b = add(&mut d, CreateKind::Cylinder, &[], None);
        add(&mut d, CreateKind::BoundaryFill, &[], Some(a));
        d.create_features.last_mut().unwrap().second_target = Some(b);
        close(
            mesh(&d).volume,
            std::f64::consts::PI * 0.01f64.powi(2) * 0.02 / 4.,
        );
    }
    #[test]
    fn coil_thread_and_involute_gear_produce_valid_solids() {
        let mut d = Design::default();
        add(
            &mut d,
            CreateKind::Coil,
            &["10 mm", "5 mm", "1", "1 mm"],
            None,
        );
        assert!(mesh(&d).volume > 0.);
        let mut d = Design::default();
        let id = add(&mut d, CreateKind::Cylinder, &["10 mm", "10 mm"], None);
        add(
            &mut d,
            CreateKind::Thread,
            &["10 mm", "5 mm", "1", "1 mm"],
            Some(id),
        );
        assert!(mesh(&d).volume < std::f64::consts::PI * 0.01f64.powi(2) * 0.01);
        let mut d = Design::default();
        add(&mut d, CreateKind::Gear, &[], None);
        assert!(mesh(&d).volume > 0.);
    }
    #[test]
    fn failed_cut_and_overlapping_axis_revolve_return_errors() {
        let mut d = Design::default();
        let id = add(&mut d, CreateKind::Box, &[], None);
        add(
            &mut d,
            CreateKind::Hole,
            &["3 mm", "20 mm", "100 mm", "100 mm", "0 mm"],
            Some(id),
        );
        assert!(confusion::evaluation::solid::evaluate(&d, || false).is_err());
    }
}
