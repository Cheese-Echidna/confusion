//! Exact inspection regressions use analytic boxes and a cylinder.
#![cfg(feature = "kernel")]
use confusion::kernel::bridge::ffi;
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}
fn edges(x: f64) -> Vec<ffi::ProfileEdge> {
    let points = [[x, 0.], [x + 0.02, 0.], [x + 0.02, 0.01], [x, 0.01]];
    (0..4)
        .map(|i| ffi::ProfileEdge {
            wire: 0,
            sx: points[i][0],
            sy: points[i][1],
            ex: points[(i + 1) % 4][0],
            ey: points[(i + 1) % 4][1],
            cx: 0.,
            cy: 0.,
            sweep: 0.,
        })
        .collect()
}
fn step(start: u32) -> ffi::ModelStep {
    ffi::ModelStep {
        edge_start: start,
        edge_count: 4,
        depth: 0.005,
        operation: 0,
        target: -1,
        support: -1,
        producer: -1,
        role: 0,
    }
}
#[test]
fn box_mass_area_validity_and_planar_curvature() {
    let mesh = ffi::extrude_region(&edges(0.), 0.005).unwrap();
    let data = mesh.inspection;
    close(mesh.volume, 0.02 * 0.01 * 0.005);
    close(data.area, 2. * (0.02 * 0.01 + 0.02 * 0.005 + 0.01 * 0.005));
    close(data.cx, 0.01);
    close(data.cy, 0.005);
    close(data.cz, 0.0025);
    assert!(data.valid);
    assert_eq!(data.solids, 1);
    assert_eq!(data.faces.len(), 6);
    for face in &data.faces {
        assert_eq!(face.samples, 25);
        close(face.min_curvature, 0.);
        close(face.max_curvature, 0.);
    }
    assert!(data.faces.iter().any(|f| (f.min_draft - 90.).abs() < 1e-8));
    assert!(data.faces.iter().any(|f| (f.min_draft + 90.).abs() < 1e-8));
}
#[test]
fn interference_distinguishes_overlap_touch_and_separation() {
    for (x, expected) in [
        (0.01, Some(0.01 * 0.01 * 0.005)),
        (0.02, None),
        (0.03, None),
    ] {
        let mut all = edges(0.);
        all.extend(edges(x));
        let mesh = ffi::evaluate_model(&all, &[step(0), step(4)], &[]).unwrap();
        assert!(mesh.inspection.error.is_empty());
        if let Some(volume) = expected {
            assert_eq!(mesh.inspection.interference.len(), 1);
            close(mesh.inspection.interference[0].volume, volume);
        } else {
            assert!(mesh.inspection.interference.is_empty());
        }
    }
}
#[test]
fn cylinder_curvature_uses_inverse_metres() {
    let r = 0.01;
    let circle = [ffi::ProfileEdge {
        wire: 0,
        sx: r,
        sy: 0.,
        ex: r,
        ey: 0.,
        cx: 0.,
        cy: 0.,
        sweep: std::f64::consts::TAU,
    }];
    let mesh = ffi::extrude_region(&circle, 0.005).unwrap();
    let side = mesh
        .inspection
        .faces
        .iter()
        .find(|f| f.max_curvature > 1.)
        .unwrap();
    close(side.min_curvature, 0.);
    close(side.max_curvature, 100.);
    close(side.min_draft, 0.);
    close(side.max_draft, 0.);
}
#[test]
fn inspect_menu_actions_are_all_available() {
    for mode in [
        confusion::ui::toolbar::Mode::Sketch,
        confusion::ui::toolbar::Mode::Solid,
    ] {
        let inspect = confusion::ui::toolbar::groups(mode)
            .into_iter()
            .find(|g| g.name == "Inspect")
            .unwrap();
        assert!(inspect.features.iter().all(|f| f.available()));
    }
}
