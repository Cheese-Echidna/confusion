//! End-to-end intent → solve → exact solid tests, including save/reopen and invalid designs.
use confusion::{document::schema::Design, parameters::expression, persistence::container};
fn rectangle() -> Design {
    let mut d = Design::default();
    d.rectangle([0., 0.], [0.08, 0.04]);
    d
}
#[test]
fn expressions_use_units_references_and_detect_cycles() {
    let mut d = rectangle();
    let id = d.parameter("depth", "width / 8 + 2 mm".into());
    assert!((expression::evaluate(&d).unwrap()[&id] - 0.012).abs() < 1e-12);
    d.parameters[0].expression = "depth".into();
    assert!(expression::evaluate(&d).unwrap_err().contains("Cyclic"));
    d.parameters[0].expression = "80 mm + 2".into();
    assert!(expression::evaluate(&d).unwrap_err().contains("scalars"));
}
#[test]
fn intent_container_round_trip_without_derived_data() {
    let d = rectangle();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.con");
    container::save(&path, &d).unwrap();
    let loaded = container::load(&path).unwrap();
    assert_eq!(
        serde_json::to_value(d).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(zip.len(), 2);
    let mut text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("design.json").unwrap(), &mut text).unwrap();
    assert!(!text.contains("history"));
    assert!(!text.contains("mesh"));
}
#[cfg(feature = "solver")]
#[test]
fn constraints_drive_geometry_report_dof_and_conflicts() {
    use confusion::{document::schema::ConstraintKind, solver::nonlinear::solve};
    let mut d = rectangle();
    let p = expression::evaluate(&d).unwrap();
    let s = solve(&d, &p).unwrap();
    assert_eq!(s.dof, 0);
    assert!(s.conflicts.is_empty());
    d.parameters[0].expression = "120 mm".into();
    let s = solve(&d, &expression::evaluate(&d).unwrap()).unwrap();
    assert!((s.points[1][0] - 0.12).abs() < 1e-8);
    let removed = d.constraints.pop().unwrap();
    let s = solve(&d, &expression::evaluate(&d).unwrap()).unwrap();
    assert_eq!(s.dof, 1);
    d.constraints.push(removed);
    let other = d.parameter("other_width", "60 mm".into());
    d.constrain(ConstraintKind::DistanceX {
        points: [d.points[0].id, d.points[1].id],
        parameter: other,
    });
    let s = solve(&d, &expression::evaluate(&d).unwrap()).unwrap();
    assert!(!s.conflicts.is_empty());
    assert!(s.residual > 0.01);
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn exact_extrusion_regenerates_after_dimension_edit_and_reopen() {
    use confusion::{
        kernel::bridge::ffi, sketch::profiles::closed_profile, solver::nonlinear::solve,
    };
    let mut d = rectangle();
    let depth = d.parameter("depth", "10 mm".into());
    d.extrusion = Some(confusion::document::schema::Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
    });
    let evaluate = |d: &Design| {
        let p = expression::evaluate(d).unwrap();
        let s = solve(d, &p).unwrap();
        let points: Vec<_> = closed_profile(d, &s.points)
            .unwrap()
            .into_iter()
            .map(|p| ffi::Point2 { x: p[0], y: p[1] })
            .collect();
        ffi::extrude(&points, p[&d.extrusion.as_ref().unwrap().depth]).unwrap()
    };
    let mesh = evaluate(&d);
    assert_eq!(mesh.faces, 6);
    assert!((mesh.volume - 0.08 * 0.04 * 0.01).abs() < 1e-12);
    assert_eq!(mesh.indices.len(), 36);
    d.parameters[0].expression = "120 mm".into();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bracket.con");
    container::save(&path, &d).unwrap();
    let d = container::load(&path).unwrap();
    let mesh = evaluate(&d);
    assert!((mesh.volume - 0.12 * 0.04 * 0.01).abs() < 1e-12);
    assert!(mesh.vertices.iter().any(|v| (v.x - 0.12).abs() < 1e-8));
}
#[test]
fn profiles_reject_open_crossing_and_disconnected_wires() {
    use confusion::sketch::profiles::closed_profile;
    let mut d = rectangle();
    let xy = d.points.iter().map(|p| p.xy).collect::<Vec<_>>();
    assert!(closed_profile(&d, &xy).is_ok());
    d.lines.pop();
    assert!(closed_profile(&d, &xy).unwrap_err().contains("open"));
    let mut d = Design::default();
    let points = [[0., 0.], [0.1, 0.1], [0., 0.1], [0.1, 0.]];
    for i in 0..4 {
        d.line(points[i], points[(i + 1) % 4]);
    }
    let xy = d.points.iter().map(|p| p.xy).collect::<Vec<_>>();
    assert!(closed_profile(&d, &xy).is_err());
    let mut d = rectangle();
    d.rectangle([0.2, 0.2], [0.3, 0.3]);
    let xy = d.points.iter().map(|p| p.xy).collect::<Vec<_>>();
    assert!(closed_profile(&d, &xy).unwrap_err().contains("Multiple"));
}
#[test]
fn malformed_references_rejected_and_reverse_rectangle_correct() {
    let mut d = Design::default();
    d.rectangle([0.08, 0.04], [0., 0.]);
    assert_eq!(d.points[0].xy, [0., 0.]);
    assert_eq!(d.points[2].xy, [0.08, 0.04]);
    d.lines[0].ends[1] = uuid::Uuid::new_v4();
    assert!(d.validate().is_err());
}

#[cfg(feature = "solver")]
#[test]
fn superseded_solves_stop_and_missing_parameters_do_not_panic() {
    use confusion::solver::nonlinear::{solve, solve_cancellable};
    let d = rectangle();
    let p = expression::evaluate(&d).unwrap();
    assert!(
        solve_cancellable(&d, &p, || true)
            .unwrap_err()
            .contains("superseded")
    );
    assert!(solve(&d, &std::collections::HashMap::new()).is_err());
}
#[test]
fn invalid_save_preserves_existing_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.con");
    let mut d = rectangle();
    container::save(&path, &d).unwrap();
    let before = std::fs::read(&path).unwrap();
    d.lines[0].ends[0] = uuid::Uuid::new_v4();
    assert!(container::save(&path, &d).is_err());
    assert_eq!(before, std::fs::read(&path).unwrap());
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn background_latest_revision_regenerates_and_kernel_errors_are_results() {
    use confusion::{kernel::bridge::ffi, runtime::worker::Worker};
    let worker = Worker::new();
    let mut d = rectangle();
    let depth = d.parameter("depth", "10 mm".into());
    d.extrusion = Some(confusion::document::schema::Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
    });
    worker.submit(1, d.clone());
    d.parameters[0].expression = "120 mm".into();
    worker.submit(2, d);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(result) = worker.poll()
            && result.revision == 2
        {
            let mesh = result.mesh.unwrap().unwrap();
            assert!((mesh.volume - 0.12 * 0.04 * 0.01).abs() < 1e-12);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "worker did not finish"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let points = [
        ffi::Point2 { x: 0., y: 0. },
        ffi::Point2 { x: 0.08, y: 0. },
        ffi::Point2 { x: 0.08, y: 0.04 },
    ];
    assert!(ffi::extrude(&points, -0.01).is_err());
}
