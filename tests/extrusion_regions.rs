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
fn invalid_and_unsupported_contours_are_rejected() {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.04, 0.02]);
    edit::circle(&mut d, [0.04, 0.01], [0.05, 0.01], None).unwrap();
    assert!(regions::regions(&d, &xy(&d)).is_err());
    let mut d = Design::default();
    d.line([0., 0.], [0.01, 0.]);
    assert!(regions::regions(&d, &xy(&d)).is_err());
    let mut d = Design::default();
    edit::ellipse(&mut d, [0., 0.], [0.02, 0.], [0., 0.01]).unwrap();
    assert!(regions::regions(&d, &xy(&d)).is_err());
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
fn tangent_circles_between_sample_vertices_are_rejected() {
    let mut d = Design::default();
    edit::circle(&mut d, [0., 0.], [0.01, 0.], None).unwrap();
    let t = 0.013_f64;
    let center = [0.02 * t.cos(), 0.02 * t.sin()];
    edit::circle(&mut d, center, [center[0] + 0.01, center[1]], None).unwrap();
    assert!(regions::regions(&d, &xy(&d)).is_err());
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
