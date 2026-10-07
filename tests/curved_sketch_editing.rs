use confusion::{
    document::schema::{ConstraintKind as C, Design},
    sketch::{edit, entities},
};
use std::f64::consts::{PI, TAU};
fn at(angle: f64) -> [f64; 2] {
    [0.01 * angle.cos(), 0.01 * angle.sin()]
}
fn close(a: [f64; 2], b: [f64; 2]) {
    assert!((a[0] - b[0]).hypot(a[1] - b[1]) < 1e-9, "{a:?} != {b:?}");
}
#[test]
fn circle_trim_preserves_identity_center_dimensions_and_roundtrip() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), None).unwrap();
    let owner = d.active_sketch;
    let center = d.circles[0].center;
    d.line([-0.02, 0.], [0.02, 0.]);
    let parameter = d.parameter("radius", "10 mm".into());
    d.constrain(C::Radius {
        circle: id,
        parameter,
    });
    d.constrain(C::Fixed {
        point: center,
        xy: [0., 0.],
    });
    edit::toggle_construction(&mut d, &[id]);
    edit::trim(&mut d, id, at(PI / 2.)).unwrap();
    assert_eq!(d.active_sketch, owner);
    assert_eq!(d.circles[0].id, id);
    assert_eq!(d.circles[0].center, center);
    assert!(d.construction_geometry.contains(&id));
    assert_eq!(d.constraints.len(), 2);
    close(entities::point(&d, d.circles[0].rim), at(PI));
    close(entities::point(&d, d.circles[0].end.unwrap()), at(TAU));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("curved.con");
    confusion::persistence::container::save(&path, &d).unwrap();
    let loaded = confusion::persistence::container::load(&path).unwrap();
    loaded.validate().unwrap();
    assert_eq!(loaded.circles[0].id, id);
}
#[test]
fn arc_middle_trim_preserves_endpoints_and_shared_center() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), Some(at(PI))).unwrap();
    let original = d.circles[0].clone();
    d.line([-0.005, -0.02], [-0.005, 0.02]);
    d.line([0.005, -0.02], [0.005, 0.02]);
    edit::trim(&mut d, id, at(PI / 2.)).unwrap();
    assert_eq!(d.circles.len(), 2);
    assert_eq!(d.circles[0].id, id);
    assert_eq!(d.circles[0].rim, original.rim);
    assert_eq!(d.circles[1].end, original.end);
    assert!(d.circles.iter().all(|c| c.center == original.center));
    assert!(
        d.circles
            .iter()
            .all(|c| entities::samples(&d, c.id).iter().all(|p| p[1] < 0.009))
    );
}
#[test]
fn wraparound_circle_circle_and_arc_arc_boundaries() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(PI / 2.), None).unwrap();
    edit::circle(&mut d, [0.01, 0.], [0.02, 0.], Some([0., 0.])).unwrap();
    // Only the upper intersection lies on the boundary arc: one cut cannot trim a circle.
    let before = serde_json::to_value(&d).unwrap();
    assert!(edit::trim(&mut d, id, at(0.)).is_err());
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
    d.circles[1].end = None;
    edit::trim(&mut d, id, at(0.)).unwrap();
    close(entities::point(&d, d.circles[0].rim), at(PI / 3.));
    close(
        entities::point(&d, d.circles[0].end.unwrap()),
        at(5. * PI / 3.),
    );
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(-PI / 2.), Some(at(PI / 2.))).unwrap();
    edit::circle(&mut d, [0.01, 0.], [0.02, 0.], None).unwrap();
    edit::trim(&mut d, id, at(0.)).unwrap();
    assert_eq!(d.circles.len(), 3);
}
#[test]
fn break_arc_shares_split_and_center_and_rejects_endpoints_atomically() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), Some(at(PI))).unwrap();
    let before = serde_json::to_value(&d).unwrap();
    assert!(edit::split(&mut d, id, at(0.)).is_err());
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
    edit::split(&mut d, id, at(PI / 2.)).unwrap();
    assert_eq!(d.circles[0].center, d.circles[1].center);
    assert_eq!(d.circles[0].end, Some(d.circles[1].rim));
    d.validate().unwrap();
}
#[test]
fn line_circle_and_line_arc_extend_filter_boundary_sweep() {
    let mut d = Design::default();
    let id = d.line([-0.03, 0.], [-0.02, 0.]);
    let arc = edit::circle(&mut d, [0., 0.], at(-PI / 2.), Some(at(PI / 2.))).unwrap();
    edit::extend(&mut d, id, [-0.02, 0.]).unwrap();
    close(entities::point(&d, d.lines[0].ends[1]), [0.01, 0.]);
    assert_eq!(d.circles[0].id, arc);
    let mut d = Design::default();
    let id = d.line([-0.03, 0.], [-0.02, 0.]);
    edit::circle(&mut d, [0., 0.], at(0.), None).unwrap();
    edit::extend(&mut d, id, [-0.02, 0.]).unwrap();
    close(entities::point(&d, d.lines[0].ends[1]), [-0.01, 0.]);
}
#[test]
fn arc_extend_nearest_endpoint_with_line_circle_and_arc_boundaries() {
    for kind in 0..3 {
        let mut d = Design::default();
        let id = edit::circle(&mut d, [0., 0.], at(0.), Some(at(PI / 4.))).unwrap();
        if kind == 0 {
            d.line([-0.02, 0.01], [0.02, 0.01]);
        } else {
            edit::circle(
                &mut d,
                [0., 0.02],
                [0.01, 0.02],
                if kind == 2 { Some([0., 0.01]) } else { None },
            )
            .unwrap();
        }
        edit::extend(&mut d, id, at(PI / 4.)).unwrap();
        close(entities::point(&d, d.circles[0].end.unwrap()), at(PI / 2.));
    }
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), Some(at(PI / 2.))).unwrap();
    d.line([-0.02, -0.01], [0.02, -0.01]);
    edit::extend(&mut d, id, at(0.)).unwrap();
    close(entities::point(&d, d.circles[0].rim), at(-PI / 2.));
}
#[test]
fn tangency_and_no_boundary_failure_are_atomic() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), None).unwrap();
    d.line([-0.02, 0.01], [0.02, 0.01]);
    let before = serde_json::to_value(&d).unwrap();
    assert!(edit::trim(&mut d, id, at(PI)).is_err());
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
    assert!(edit::extend(&mut d, id, at(0.)).is_err());
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
    assert!(edit::trim(&mut d, id, [f64::NAN, 0.]).is_err());
}
#[test]
fn trim_line_retains_id_orientation_and_unrelated_constraints() {
    let mut d = Design::default();
    let id = d.line([-0.02, 0.], [0.02, 0.]);
    d.constrain(C::Horizontal { line: id });
    let other = d.line([0.03, 0.], [0.04, 0.]);
    d.constrain(C::Horizontal { line: other });
    edit::circle(&mut d, [0., 0.], at(0.), None).unwrap();
    edit::trim(&mut d, id, [0., 0.]).unwrap();
    assert_eq!(d.lines[0].id, id);
    assert_eq!(d.lines.len(), 3);
    assert_eq!(d.constraints.len(), 3);
    d.validate().unwrap();
}
#[test]
fn inactive_sketch_edit_preserves_owner_and_active_geometry() {
    use confusion::document::model::SketchPlane;
    let mut d = Design::default();
    let owner = d.create_sketch(SketchPlane::Xy).unwrap();
    let id = edit::circle(&mut d, [0., 0.], at(0.), Some(at(PI))).unwrap();
    let active = d.create_sketch(SketchPlane::Xy).unwrap();
    let unrelated = d.line([0., 0.], [0.01, 0.]);
    edit::split(&mut d, id, at(PI / 2.)).unwrap();
    assert_eq!(d.active_sketch, Some(active));
    assert_eq!(d.lines[0].id, unrelated);
    let stored = d.sketches.iter().find(|s| s.id == owner).unwrap();
    assert_eq!(stored.geometry.circles.len(), 2);
    assert!(d.circles.is_empty());
}
#[cfg(feature = "solver")]
#[test]
fn broken_arc_has_only_split_angle_dof() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), Some(at(PI))).unwrap();
    for p in entities::curve_points(&d, id) {
        d.constrain(C::Fixed {
            point: p,
            xy: entities::point(&d, p),
        });
    }
    edit::split(&mut d, id, at(PI / 2.)).unwrap();
    let parameters = confusion::parameters::expression::evaluate(&d).unwrap();
    let solution = confusion::solver::nonlinear::solve(&d, &parameters).unwrap();
    assert!(solution.conflicts.is_empty(), "{:?}", solution.conflicts);
    assert_eq!(solution.dof, 1);
    let split = d.circles[1].rim;
    d.constrain(C::Fixed {
        point: split,
        xy: entities::point(&d, split),
    });
    let solution = confusion::solver::nonlinear::solve(&d, &parameters).unwrap();
    assert_eq!(solution.dof, 0);
}
#[test]
fn trim_and_break_form_an_exact_closed_profile() {
    let mut d = Design::default();
    let id = edit::circle(&mut d, [0., 0.], at(0.), None).unwrap();
    d.line([-0.01, 0.], [0.01, 0.]);
    edit::trim(&mut d, id, at(PI / 2.)).unwrap();
    edit::split(&mut d, id, at(3. * PI / 2.)).unwrap();
    let xy = d.points.iter().map(|p| p.xy).collect::<Vec<_>>();
    let regions = confusion::sketch::regions::regions(&d, &xy).unwrap();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].boundary.len(), 3);
    #[cfg(feature = "kernel")]
    {
        let mesh = confusion::sketch::regions::extrude(&regions[0], 0.01).unwrap();
        assert!((mesh.volume - PI * 0.01_f64.powi(2) / 2. * 0.01).abs() < 1e-10);
    }
}
#[test]
fn driven_curve_dimensions_cleanup_keeps_unrelated_point_dimensions() {
    use confusion::document::schema::Constraint;
    use uuid::Uuid;
    let mut d = Design::default();
    let id = d.line([0., 0.], [0.01, 0.]);
    let parameter = d.parameter("length", "10 mm".into());
    let label = Uuid::new_v4();
    d.driven_dimensions.push(Constraint {
        id: label,
        kind: C::Length {
            line: id,
            parameter,
        },
    });
    d.dimension_positions.insert(label, [0.005, 0.005]);
    let endpoints = d.lines[0].ends;
    let point_dimension = Uuid::new_v4();
    d.driven_dimensions.push(Constraint {
        id: point_dimension,
        kind: C::Distance {
            points: endpoints,
            parameter,
        },
    });
    d.line([0.02, -0.01], [0.02, 0.01]);
    edit::extend(&mut d, id, [0.01, 0.]).unwrap();
    assert!(d.driven_dimensions.iter().all(|c| c.id != label));
    assert!(!d.dimension_positions.contains_key(&label));
    assert!(d.driven_dimensions.iter().any(|c| c.id == point_dimension));
    assert!(d.points.iter().any(|p| p.id == endpoints[1]));
    d.validate().unwrap();
}
