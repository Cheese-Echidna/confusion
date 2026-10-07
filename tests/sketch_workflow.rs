use confusion::{
    document::schema::{ConstraintKind as C, Design},
    parameters::expression,
    settings::keymap::Keymap,
    sketch::{dimensions, edit, entities},
};
use uuid::Uuid;
#[test]
fn circles_arcs_driven_dimensions_and_construction_roundtrip() {
    let mut d = Design::default();
    let c = edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
    let a = edit::circle(&mut d, [0.03, 0.], [0.04, 0.], Some([0.03, 0.01])).unwrap();
    edit::toggle_construction(&mut d, &[a]);
    d.driven_dimensions
        .push(confusion::document::schema::Constraint {
            id: Uuid::new_v4(),
            kind: C::Diameter {
                circle: c,
                parameter: Uuid::new_v4(),
            },
        });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sketch.con");
    confusion::persistence::container::save(&path, &d).unwrap();
    let reopen = confusion::persistence::container::load(&path).unwrap();
    assert_eq!(reopen.circles.len(), 2);
    assert!(reopen.construction_geometry.contains(&a));
    assert!(dimensions::measure(&reopen, &[c]).contains("20.000"));
}
#[test]
fn angular_parameters_keep_units_and_cycles() {
    let mut d = Design::default();
    let id = d.parameter("angle", "45 deg".into());
    d.parameters[0].angular = true;
    let other = d.parameter("other", "angle / 2 + 10 deg".into());
    d.parameters[1].angular = true;
    let p = expression::evaluate(&d).unwrap();
    assert!((p[&id] - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
    assert!((p[&other].to_degrees() - 32.5).abs() < 1e-12);
    d.parameters[1].expression = "angle + 1 mm".into();
    assert!(expression::evaluate(&d).is_err());
}
#[test]
fn keymap_defaults_overrides_conflicts_and_text_focus() {
    let k = Keymap::default();
    assert_eq!(k.resolve("c", true, false), Some("sketch-circle"));
    assert_eq!(k.resolve("i", true, false), Some("sketch-measure-sketch"));
    assert_eq!(k.resolve("l", true, true), None);
    assert_eq!(k.resolve("l", false, false), None);
    let k=Keymap::from_json(&serde_json::json!({"keybindings":{"sketch":{"sketch-line":"shift-l","sketch-circle":null}}})).unwrap();
    assert_eq!(k.resolve("l", true, false), None);
    assert_eq!(k.resolve("shift-l", true, false), Some("sketch-line"));
    assert!(
        Keymap::from_json(&serde_json::json!({"keybindings":{"sketch":{"sketch-line":"r"}}}))
            .is_err()
    );
}
#[test]
fn break_preserves_original_length_and_delete_repairs_references() {
    let mut d = Design::default();
    let l = d.line([0., 0.], [0.04, 0.]);
    d.constrain(C::Horizontal { line: l });
    let p = d.parameter("length", "40 mm".into());
    d.constrain(C::Length {
        line: l,
        parameter: p,
    });
    edit::split(&mut d, l, [0.01, 0.]).unwrap();
    d.validate().unwrap();
    assert_eq!(d.lines.len(), 2);
    assert!(
        d.constraints
            .iter()
            .any(|c| matches!(c.kind, C::Distance { .. }))
    );
    let point = d.lines[0].ends[1];
    edit::delete(&mut d, &[point]);
    d.validate().unwrap();
    assert!(d.lines.is_empty());
}
#[test]
fn trim_removes_only_clicked_interval_and_extend_reaches_intersection() {
    let mut d = Design::default();
    let l = d.line([0., 0.], [0.04, 0.]);
    d.line([0.01, -0.01], [0.01, 0.01]);
    d.line([0.03, -0.01], [0.03, 0.01]);
    edit::trim(&mut d, l, [0.02, 0.]).unwrap();
    d.validate().unwrap();
    assert_eq!(d.lines.len(), 4);
    assert!(!d.lines.iter().any(|l| {
        entities::samples(&d, l.id)
            .windows(2)
            .any(|p| entities::segment_distance([0.02, 0.], p[0], p[1]) < 1e-8)
    }));
    let short = d.line([0.05, 0.02], [0.06, 0.02]);
    d.line([0.08, 0.01], [0.08, 0.03]);
    edit::extend(&mut d, short, [0.06, 0.02]).unwrap();
    assert_eq!(
        entities::point(&d, d.lines.iter().find(|l| l.id == short).unwrap().ends[1]),
        [0.08, 0.02]
    );
}
#[cfg(feature = "solver")]
fn solve(d: &Design) -> confusion::solver::nonlinear::Solution {
    confusion::solver::nonlinear::solve(d, &expression::evaluate(d).unwrap()).unwrap()
}
#[cfg(feature = "solver")]
#[test]
fn local_dof_distinguishes_locked_and_free_geometry() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.04]);
    d.line([0.1, 0.], [0.12, 0.02]);
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert_eq!(s.dof, 4);
    assert!(s.point_dof[..4].iter().all(|d| *d == 0));
    assert!(s.point_dof[4..].iter().all(|d| *d > 0));
}
#[cfg(feature = "solver")]
#[test]
fn fully_dimensioned_circle_has_three_physical_dof_and_locks() {
    let mut d = Design::default();
    let c = edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
    assert_eq!(solve(&d).dof, 3);
    let center = d.circles[0].center;
    d.constrain(C::Fixed {
        point: center,
        xy: [0., 0.],
    });
    let p = d.parameter("diameter", "30 mm".into());
    d.constrain(C::Diameter {
        circle: c,
        parameter: p,
    });
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert_eq!(s.dof, 0);
    assert!(s.point_dof.iter().all(|d| *d == 0));
    let rim = d
        .points
        .iter()
        .position(|p| p.id == d.circles[0].rim)
        .unwrap();
    assert!((s.points[rim][0] - 0.015).abs() < 1e-8);
}
#[cfg(feature = "solver")]
#[test]
fn line_relationships_angle_and_midpoint_solve() {
    let mut d = Design::default();
    let a = d.line([0., 0.], [0.04, 0.]);
    let b = d.line([0.01, 0.01], [0.02, 0.04]);
    edit::fixed(&mut d, &[a]);
    let p = d.parameter("angle", "90 deg".into());
    d.parameters.last_mut().unwrap().angular = true;
    d.constrain(C::Angle {
        lines: [a, b],
        parameter: p,
    });
    d.constrain(C::Equal { curves: [a, b] });
    let m = d.point([0.025, 0.01]);
    d.constrain(C::Midpoint { point: m, line: a });
    let s = solve(&d);
    assert!(s.conflicts.is_empty(), "{:?}", s.conflicts);
    let i = d.points.iter().position(|p| p.id == m).unwrap();
    assert!((s.points[i][0] - 0.02).abs() < 1e-8);
    assert!(s.points[i][1].abs() < 1e-8);
}
#[cfg(feature = "solver")]
#[test]
fn circle_tangent_to_fixed_line_and_arc_radius_regenerate() {
    let mut d = Design::default();
    let l = d.line([-0.04, 0.], [0.04, 0.]);
    edit::fixed(&mut d, &[l]);
    let c = edit::circle(&mut d, [0., 0.012], [0.01, 0.012], None).unwrap();
    let r = d.parameter("radius", "10 mm".into());
    d.constrain(C::Radius {
        circle: c,
        parameter: r,
    });
    d.constrain(C::Tangent { curves: [l, c] });
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    let center = d
        .points
        .iter()
        .position(|p| p.id == d.circles[0].center)
        .unwrap();
    assert!((s.points[center][1].abs() - 0.01).abs() < 1e-8);
}

#[cfg(feature = "solver")]
#[test]
fn ellipse_spline_slot_and_fillet_are_editable_and_solve() {
    let mut d = Design::default();
    let ellipse = edit::ellipse(&mut d, [0.1, 0.], [0.13, 0.01], [0.1, 0.015]).unwrap();
    assert_eq!(entities::samples(&d, ellipse).len(), 129);
    let spline = edit::spline(&mut d, &[[0., 0.1], [0.02, 0.13], [0.04, 0.1]], true).unwrap();
    let samples = entities::samples(&d, spline);
    assert_eq!(samples[0], [0., 0.1]);
    assert_eq!(*samples.last().unwrap(), [0.04, 0.1]);
    edit::slot(&mut d, [0., 0.], [0.04, 0.], [0.04, 0.01]).unwrap();
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    let mut d = Design::default();
    let a = d.line([0., 0.], [0.04, 0.]);
    let b = d.line([0.04, 0.], [0.04, 0.04]);
    d.constrain(C::Horizontal { line: a });
    d.constrain(C::Vertical { line: b });
    edit::fillet(&mut d, &[a, b], 0.005).unwrap();
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert_eq!(d.circles.len(), 1);
    assert!((entities::radius(&d, &d.circles[0]) - 0.005).abs() < 1e-9);
}
#[cfg(feature = "solver")]
#[test]
fn copied_geometry_keeps_internal_constraints_and_shares_driving_parameters() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    let ids = d.lines.iter().map(|l| l.id).collect::<Vec<_>>();
    let old = d.constraints.len();
    let copies = edit::transform(&mut d, &ids, |p| [p[0] + 0.1, p[1]], true).unwrap();
    assert_eq!(copies.len(), 4);
    assert_eq!(d.constraints.len(), old * 2);
    d.parameters[0].expression = "60 mm".into();
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert_eq!(s.dof, 0);
    assert!(s.points.iter().any(|p| (p[0] - 0.16).abs() < 1e-8));
}
#[cfg(feature = "solver")]
#[test]
fn dragging_seeds_moves_free_geometry_and_preserves_hard_dimensions() {
    let mut d = Design::default();
    let l = d.line([0., 0.], [0.04, 0.]);
    let p = d.parameter("length", "40 mm".into());
    d.constrain(C::Length {
        line: l,
        parameter: p,
    });
    d.constrain(C::Horizontal { line: l });
    d.points[1].xy = [0.07, 0.03];
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert!((s.points[0][1] - s.points[1][1]).abs() < 1e-8);
    assert!(((s.points[1][0] - s.points[0][0]).abs() - 0.04).abs() < 1e-8);
    for (p, xy) in d.points.iter_mut().zip(&s.points) {
        p.xy = *xy
    }
    let locked = d.points[1].xy;
    edit::fixed(&mut d, &[l]);
    d.points[1].xy = [0.1, 0.02];
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert!((s.points[1][0] - locked[0]).abs() < 1e-8);
}

#[cfg(feature = "solver")]
#[test]
fn temporary_drag_target_tracks_corner_and_cannot_move_locked_geometry() {
    let mut d = Design::default();
    let a = d.line([0., 0.], [0.04, 0.]);
    let b = d.line([0.04, 0.], [0.04, 0.03]);
    let c = d.line([0.04, 0.03], [0., 0.03]);
    let e = d.line([0., 0.03], [0., 0.]);
    for l in [a, c] {
        d.constrain(C::Horizontal { line: l })
    }
    for l in [b, e] {
        d.constrain(C::Vertical { line: l })
    }
    let origin = d.points[0].id;
    d.constrain(C::Fixed {
        point: origin,
        xy: [0., 0.],
    });
    let corner = d.points.iter().find(|p| p.xy == [0.04, 0.03]).unwrap().id;
    let width = d.parameter("width", "40 mm".into());
    d.constrain(C::Length {
        line: a,
        parameter: width,
    });
    let p = expression::evaluate(&d).unwrap();
    let s = confusion::solver::nonlinear::solve_drag(&d, &p, &[(corner, [0.04, 0.05])]).unwrap();
    assert!(s.conflicts.is_empty());
    let i = d.points.iter().position(|p| p.id == corner).unwrap();
    assert!((s.points[i][1] - 0.05).abs() < 1e-6, "{:?}", s.points[i]);
    let zero = confusion::solver::nonlinear::solve_drag(&d, &p, &[(origin, [0.02, 0.02])]).unwrap();
    assert!(zero.conflicts.is_empty());
    assert!(zero.points[0][0].abs() < 1e-8);
}
#[cfg(feature = "solver")]
#[test]
fn rotated_and_scaled_copies_preserve_dimension_intent() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    let ids = d.lines.iter().map(|l| l.id).collect::<Vec<_>>();
    edit::transform(&mut d, &ids, |p| [-p[1] + 0.1, p[0]], true).unwrap();
    let s = solve(&d);
    assert!(s.conflicts.is_empty(), "{:?}", s.conflicts);
    assert_eq!(s.dof, 0);
    assert!(
        s.points
            .iter()
            .any(|p| (p[0] - 0.08).abs() < 1e-8 && (p[1] - 0.04).abs() < 1e-8)
    );
    edit::transform(&mut d, &ids, |p| [p[0] * 2., p[1] * 2.], false).unwrap();
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert!(
        s.points
            .iter()
            .any(|p| (p[0] - 0.08).abs() < 1e-8 && (p[1] - 0.04).abs() < 1e-8)
    );
}

#[test]
fn length_squared_is_not_an_angle_and_engineering_expressions_keep_units() {
    let mut d = Design::default();
    d.parameter("width", "40 mm".into());
    let diagonal = d.parameter("diagonal", "sqrt(width^2 + (30 mm)^2)".into());
    let vertical = d.parameter("vertical", "sin(30 deg) * width".into());
    let limit = d.parameter("limit", "max(width, 50 mm)".into());
    let values = expression::evaluate(&d).unwrap();
    assert!((values[&diagonal] - 0.05).abs() < 1e-12);
    assert!((values[&vertical] - 0.02).abs() < 1e-12);
    assert!((values[&limit] - 0.05).abs() < 1e-12);
    let angle = d.parameter("angle", "width * width".into());
    d.parameters
        .iter_mut()
        .find(|p| p.id == angle)
        .unwrap()
        .angular = true;
    assert!(expression::evaluate(&d).is_err());
    assert!(expression::dimension_input("80 / 2", false).contains("mm"));
}
#[cfg(feature = "solver")]
#[test]
fn connected_offsets_remain_closed_and_follow_upstream_dimensions() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.04]);
    let selected = d.lines.iter().map(|l| l.id).collect::<Vec<_>>();
    edit::offset(&mut d, &selected, 0.005).unwrap();
    let original = d.lines[..4].iter().map(|l| l.id).collect::<Vec<_>>();
    let s = solve(&d);
    assert!(s.conflicts.is_empty(), "{:?}", s.conflicts);
    for (p, xy) in d.points.iter_mut().zip(&s.points) {
        p.xy = *xy
    }
    let mut profile = d.clone();
    profile.lines.retain(|l| !original.contains(&l.id));
    let xy = profile.points.iter().map(|p| p.xy).collect::<Vec<_>>();
    assert_eq!(
        confusion::sketch::profiles::closed_profile(&profile, &xy)
            .unwrap()
            .len(),
        4
    );
    d.parameters[0].expression = "100 mm".into();
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    assert!(s.points.iter().any(|p| (p[0] - 0.095).abs() < 1e-8));
}
#[cfg(feature = "solver")]
#[test]
fn circle_offsets_follow_radius_changes_and_circle_trim_creates_an_arc() {
    let mut d = Design::default();
    let c = edit::circle(&mut d, [0., 0.], [0.02, 0.], None).unwrap();
    let p = d.parameter("radius", "20 mm".into());
    d.constrain(C::Radius {
        circle: c,
        parameter: p,
    });
    edit::offset(&mut d, &[c], 0.005).unwrap();
    d.parameters[0].expression = "30 mm".into();
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    let outer = &d.circles[1];
    let center = d.points.iter().position(|p| p.id == outer.center).unwrap();
    let rim = d.points.iter().position(|p| p.id == outer.rim).unwrap();
    assert!(
        ((s.points[center][0] - s.points[rim][0]).hypot(s.points[center][1] - s.points[rim][1])
            - 0.035)
            .abs()
            < 1e-8
    );
    let mut d = Design::default();
    let c = edit::circle(&mut d, [0., 0.], [0.02, 0.], None).unwrap();
    d.line([-0.03, 0.], [0.03, 0.]);
    edit::trim(&mut d, c, [0., 0.02]).unwrap();
    assert_eq!(d.circles.len(), 1);
    assert!(d.circles[0].end.is_some());
    let s = solve(&d);
    assert!(s.conflicts.is_empty());
    let samples = entities::samples(&d, d.circles[0].id);
    assert!(samples.iter().all(|p| p[1] <= 1e-8));
    {
        let arc = d.circles[0].id;
        edit::split(&mut d, arc, [0., -0.02])
    }
    .unwrap();
    assert_eq!(d.circles.len(), 2);
}

#[cfg(feature = "solver")]
#[test]
fn complex_acceptance_sketch_reopens_and_regenerates_without_free_points() {
    use confusion::{persistence::container, solver::nonlinear};
    let mut d = container::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/assets/complex-sketch.con"),
    )
    .unwrap();
    for width in ["80 mm", "100 mm", "90 mm"] {
        d.parameters
            .iter_mut()
            .find(|p| p.name == "width")
            .unwrap()
            .expression = width.into();
        let solution = nonlinear::solve(&d, &expression::evaluate(&d).unwrap()).unwrap();
        assert!(solution.conflicts.is_empty());
        assert_eq!(solution.dof, 0);
        assert!(solution.point_dof.iter().all(|n| *n == 0));
        assert!(solution.redundant.is_empty(), "{:?}", solution.redundant);
        for (p, xy) in d.points.iter_mut().zip(solution.points) {
            p.xy = xy;
        }
    }
}
