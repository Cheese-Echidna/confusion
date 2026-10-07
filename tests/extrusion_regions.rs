use confusion::{
    document::schema::{Design, Extrusion},
    persistence::container,
    sketch::{edit, regions},
};
fn xy(d: &Design) -> Vec<[f64; 2]> {
    d.points.iter().map(|p| p.xy).collect()
}
#[test]
fn disjoint_regions_require_selection_and_preserve_identity() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    d.rectangle([0.1, 0.], [0.12, 0.02]);
    assert!(regions::select(&d, &xy(&d), &[]).is_err());
    let r = regions::regions(&d, &xy(&d)).unwrap().remove(1);
    let depth = d.parameter("depth", "5 mm".into());
    d.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary: r.boundary.clone(),
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("regions.con");
    container::save(&path, &d).unwrap();
    let mut d = container::load(&path).unwrap();
    assert_eq!(
        regions::select(&d, &xy(&d), &r.boundary).unwrap().boundary,
        r.boundary
    );
    d.lines.remove(4);
    assert!(d.validate().is_err());
    assert!(regions::select(&d, &xy(&d), &r.boundary).is_err());
}
#[test]
fn intersections_and_ellipses_are_supported_but_open_sketches_have_no_region() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    edit::circle(&mut d, [0.04, 0.01], [0.05, 0.01], None).unwrap();
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 3);
    let mut d = Design::default();
    d.line([0., 0.], [0.01, 0.]);
    assert!(regions::regions(&d, &xy(&d)).is_err());
    let mut d = Design::default();
    edit::ellipse(&mut d, [0., 0.], [0.02, 0.], [0., 0.01]).unwrap();
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 1);
}
#[test]
fn nested_regions_have_immediate_holes() {
    let mut d = Design::default();
    for r in [0.03, 0.02, 0.01] {
        edit::circle(&mut d, [0., 0.], [r, 0.], None).unwrap();
    }
    let all = regions::regions(&d, &xy(&d)).unwrap();
    assert_eq!(
        all.iter().map(|r| r.wires.len()).collect::<Vec<_>>(),
        vec![2, 2, 1]
    );
    assert_eq!(
        regions::select(&d, &xy(&d), &[]).unwrap().boundary,
        all[0].boundary
    );
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn exact_plate_holes_slot_resize_and_reopen() {
    use confusion::{parameters::expression, solver::nonlinear};
    let mut d = container::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/assets/complex-sketch.con"),
    )
    .unwrap();
    let p = expression::evaluate(&d).unwrap();
    let s = nonlinear::solve(&d, &p).unwrap();
    let r = regions::select(&d, &s.points, &[]).unwrap();
    assert_eq!(r.wires.len(), 6);
    let depth = d.parameter("depth", "10 mm".into());
    d.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary: r.boundary,
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plate.con");
    for width in [0.08, 0.10, 0.09] {
        d.parameters
            .iter_mut()
            .find(|p| p.name == "width")
            .unwrap()
            .expression = format!("{} mm", width * 1000.);
        container::save(&path, &d).unwrap();
        d = container::load(&path).unwrap();
        let p = expression::evaluate(&d).unwrap();
        let s = nonlinear::solve(&d, &p).unwrap();
        assert!(s.conflicts.is_empty());
        let r = regions::select(&d, &s.points, &d.extrusion.as_ref().unwrap().boundary).unwrap();
        let mesh = regions::extrude(&r, p[&depth]).unwrap();
        let area = width * 0.05
            - 4. * std::f64::consts::PI * 0.003_f64.powi(2)
            - ((width - 0.06) * 0.004 + std::f64::consts::PI * 0.002_f64.powi(2));
        assert!(
            (mesh.volume - area * 0.01).abs() < 1e-11,
            "{} != {}",
            mesh.volume,
            area * 0.01
        );
        assert!(!mesh.indices.is_empty());
    }
}
#[cfg(feature = "kernel")]
#[test]
fn exact_circle_and_reversed_slot() {
    let mut d = Design::default();
    edit::circle(&mut d, [0., 0.], [0.02, 0.], None).unwrap();
    let r = regions::select(&d, &xy(&d), &[]).unwrap();
    let mesh = regions::extrude(&r, 0.01).unwrap();
    assert!((mesh.volume - std::f64::consts::PI * 0.02_f64.powi(2) * 0.01).abs() < 1e-12);
    let mut d = Design::default();
    edit::slot(&mut d, [0.04, 0.], [0., 0.], [0., 0.005]).unwrap();
    let r = regions::select(&d, &xy(&d), &[]).unwrap();
    let mesh = regions::extrude(&r, 0.01).unwrap();
    assert!(
        (mesh.volume - (0.04 * 0.01 + std::f64::consts::PI * 0.005_f64.powi(2)) * 0.01).abs()
            < 1e-12
    );
}
#[test]
fn tangent_circles_remain_separately_selectable() {
    let mut d = Design::default();
    edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
    let t = 0.013_f64;
    let center = [0.02 * t.cos(), 0.02 * t.sin()];
    edit::circle(&mut d, center, [center[0] + 0.01, center[1]], None).unwrap();
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 2);
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn background_worker_evaluates_curves_and_holes() {
    use confusion::runtime::worker::Worker;
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    edit::circle(&mut d, [0.01, 0.01], [0.013, 0.01], None).unwrap();
    let boundary = regions::select(&d, &xy(&d), &[]).unwrap().boundary;
    let depth = d.parameter("depth", "5 mm".into());
    d.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary,
    });
    let worker = Worker::new();
    worker.submit(7, d);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(result) = worker.poll() {
            assert_eq!(result.revision, 7);
            let mesh = result.mesh.unwrap().unwrap();
            assert!(
                (mesh.volume - (0.04 * 0.02 - std::f64::consts::PI * 0.003_f64.powi(2)) * 0.005)
                    .abs()
                    < 1e-11
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
#[cfg(feature = "kernel")]
#[test]
fn selected_nested_region_has_exact_annulus_volume() {
    let mut d = Design::default();
    for r in [0.03, 0.02, 0.01] {
        edit::circle(&mut d, [0., 0.], [r, 0.], None).unwrap();
    }
    let all = regions::regions(&d, &xy(&d)).unwrap();
    for (i, radii) in [
        (0, (0.03_f64, 0.02_f64)),
        (1, (0.02, 0.01)),
        (2, (0.01, 0.)),
    ] {
        let region = regions::select(&d, &xy(&d), &all[i].boundary).unwrap();
        let mesh = regions::extrude(&region, 0.01).unwrap();
        let volume = std::f64::consts::PI * (radii.0.powi(2) - radii.1.powi(2)) * 0.01;
        assert!((mesh.volume - volume).abs() < 1e-12);
    }
}

#[test]
fn dividing_line_and_open_tail_create_two_selectable_faces() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    d.line([0.02, -0.01], [0.02, 0.03]);
    let all = regions::regions(&d, &xy(&d)).unwrap();
    assert_eq!(all.len(), 2);
    let left = regions::at(&d, &xy(&d), [0.01, 0.01]).unwrap().unwrap();
    let right = regions::at(&d, &xy(&d), [0.03, 0.01]).unwrap().unwrap();
    assert_ne!(left.boundary, right.boundary);
    assert_eq!(
        regions::select(&d, &xy(&d), &left.boundary)
            .unwrap()
            .boundary,
        left.boundary
    );
    let depth = d.parameter("depth", "5 mm".into());
    d.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary: left.boundary,
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("split.con");
    container::save(&path, &d).unwrap();
    let reopened = container::load(&path).unwrap();
    assert_eq!(
        regions::select(
            &reopened,
            &xy(&reopened),
            &reopened.extrusion.as_ref().unwrap().boundary
        )
        .unwrap()
        .wires
        .len(),
        1
    );
}
#[test]
fn overlapping_rectangles_and_shared_edges_form_faces() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.03, 0.02]);
    d.rectangle([0.01, 0.01], [0.04, 0.03]);
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 3);
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.02, 0.02]);
    d.rectangle([0.02, 0.], [0.04, 0.02]);
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 2);
}
#[test]
fn spline_boundary_and_unrelated_open_curves_are_selectable() {
    for fit in [false, true] {
        let mut d = Design::default();
        edit::spline(
            &mut d,
            &[[0., 0.], [0.01, 0.02], [0.03, 0.02], [0.04, 0.]],
            fit,
        )
        .unwrap();
        d.line([0.04, 0.], [0., 0.]);
        d.line([0.1, 0.1], [0.2, 0.2]);
        assert!(regions::at(&d, &xy(&d), [0.02, 0.005]).unwrap().is_some());
    }
}
#[cfg(feature = "kernel")]
#[test]
fn curved_and_split_regions_extrude_as_exact_solids() {
    let mut d = Design::default();
    edit::ellipse(&mut d, [0., 0.], [0.02, 0.], [0., 0.01]).unwrap();
    let r = regions::select(&d, &xy(&d), &[]).unwrap();
    let mesh = regions::extrude(&r, 0.005).unwrap();
    assert!((mesh.volume - std::f64::consts::PI * 0.02 * 0.01 * 0.005).abs() < 1e-12);
    for fit in [false, true] {
        let mut d = Design::default();
        edit::spline(
            &mut d,
            &[[0., 0.], [0.01, 0.02], [0.03, 0.02], [0.04, 0.]],
            fit,
        )
        .unwrap();
        d.line([0.04, 0.], [0., 0.]);
        let r = regions::select(&d, &xy(&d), &[]).unwrap();
        assert!(regions::extrude(&r, 0.005).unwrap().volume > 0.);
    }
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    d.line([0.02, -0.01], [0.02, 0.03]);
    for r in regions::regions(&d, &xy(&d)).unwrap() {
        assert!((regions::extrude(&r, 0.005).unwrap().volume - 0.02 * 0.02 * 0.005).abs() < 1e-12);
    }
}

#[cfg(feature = "kernel")]
#[test]
fn intersecting_ellipses_and_splines_keep_exact_curves() {
    for axes in [[0.02, 0.01], [0.01, 0.02]] {
        let mut d = Design::default();
        edit::ellipse(&mut d, [0., 0.], [axes[0], 0.], [0., axes[1]]).unwrap();
        d.line([0., -0.03], [0., 0.03]);
        let all = regions::regions(&d, &xy(&d)).unwrap();
        assert_eq!(all.len(), 2);
        for r in all {
            assert!(
                (regions::extrude(&r, 0.005).unwrap().volume
                    - std::f64::consts::PI * axes[0] * axes[1] * 0.005 / 2.)
                    .abs()
                    < 1e-11
            );
        }
    }
    let mut d = Design::default();
    edit::ellipse(&mut d, [0., 0.], [0.02, 0.], [0., 0.01]).unwrap();
    edit::ellipse(&mut d, [0.02, 0.], [0.04, 0.], [0.02, 0.01]).unwrap();
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 3);
    for r in regions::regions(&d, &xy(&d)).unwrap() {
        assert!(regions::extrude(&r, 0.005).unwrap().volume > 0.);
    }
    let mut d = Design::default();
    edit::spline(
        &mut d,
        &[[0., 0.], [0.01, 0.02], [0.03, 0.02], [0.04, 0.]],
        false,
    )
    .unwrap();
    d.line([0.04, 0.], [0., 0.]);
    d.line([0.02, -0.01], [0.02, 0.03]);
    let all = regions::regions(&d, &xy(&d)).unwrap();
    assert_eq!(all.len(), 2);
    for r in all {
        assert!(regions::extrude(&r, 0.005).unwrap().volume > 0.);
    }
}
#[test]
fn coincident_circles_and_changed_crossing_sources_do_not_rebind() {
    let mut d = Design::default();
    edit::circle(&mut d, [0., 0.], [0.02, 0.], None).unwrap();
    edit::circle(&mut d, [0., 0.], [0.02, 0.], None).unwrap();
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 1);
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    d.line([0.02, -0.01], [0.02, 0.03]);
    let boundary = regions::at(&d, &xy(&d), [0.01, 0.01])
        .unwrap()
        .unwrap()
        .boundary;
    d.lines.last_mut().unwrap().id = uuid::Uuid::new_v4();
    assert!(regions::select(&d, &xy(&d), &boundary).is_err());
}

#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn ellipse_axis_edit_invalidates_exact_geometry_cache() {
    use confusion::evaluation::{cache::EvaluationCache, solid};
    let mut d = Design::default();
    edit::ellipse(&mut d, [0., 0.], [0.02, 0.], [0., 0.01]).unwrap();
    let depth = d.parameter("depth", "5 mm".into());
    d.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary: vec![],
    });
    let mut cache = EvaluationCache::default();
    let initial = solid::evaluate_cached(&d, || false, &mut cache)
        .unwrap()
        .mesh
        .unwrap()
        .volume;
    let minor = d.ellipses[0].minor;
    d.points.iter_mut().find(|p| p.id == minor).unwrap().xy = [0., 0.015];
    let updated = solid::evaluate_cached(&d, || false, &mut cache)
        .unwrap()
        .mesh
        .unwrap()
        .volume;
    assert!((updated - initial * 1.5).abs() < 1e-11);
    assert_eq!(cache.reused_features(), 0);
}

#[test]
fn partitioned_inner_component_is_one_hole_in_surrounding_face() {
    let mut d = Design::default();
    d.rectangle([-0.03, -0.03], [0.03, 0.03]);
    edit::circle(&mut d, [0., 0.], [0.02, 0.], None).unwrap();
    d.line([-0.02, 0.], [0.02, 0.]);
    let all = regions::regions(&d, &xy(&d)).unwrap();
    assert_eq!(all.len(), 3);
    let surrounding = regions::at(&d, &xy(&d), [0.025, 0.]).unwrap().unwrap();
    assert_eq!(surrounding.wires.len(), 2);
    assert_eq!(
        regions::select(&d, &xy(&d), &[]).unwrap().boundary,
        surrounding.boundary
    );
    assert!(
        regions::at(&d, &xy(&d), [0., 0.01])
            .unwrap()
            .unwrap()
            .nested
    );
    #[cfg(feature = "kernel")]
    assert!(
        (regions::extrude(&surrounding, 0.005).unwrap().volume
            - (0.06 * 0.06 - std::f64::consts::PI * 0.02 * 0.02) * 0.005)
            .abs()
            < 1e-11
    );
}

#[test]
fn open_bridge_to_hole_does_not_destroy_the_surrounding_face() {
    let mut d = Design::default();
    d.rectangle([-0.03, -0.03], [0.03, 0.03]);
    edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
    d.line([0.01, 0.], [0.03, 0.]);
    let outer = regions::at(&d, &xy(&d), [0.02, 0.01]).unwrap().unwrap();
    assert_eq!(outer.wires.len(), 2);
    assert_eq!(regions::regions(&d, &xy(&d)).unwrap().len(), 2);
    #[cfg(feature = "kernel")]
    assert!(
        (regions::extrude(&outer, 0.005).unwrap().volume
            - (0.06 * 0.06 - std::f64::consts::PI * 0.01 * 0.01) * 0.005)
            .abs()
            < 1e-11
    );
}
