use confusion::exchange::export::{ExportFormat, ExportMesh, save};

fn tetrahedron(vertices: Vec<[f64; 3]>) -> ExportMesh {
    ExportMesh::new(vertices, &[0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3]).unwrap()
}

fn rejected_stl_preserves_destination(mesh: &ExportMesh) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("part.stl");
    std::fs::write(&path, b"existing part").unwrap();
    assert!(save(&path, mesh, ExportFormat::Stl).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"existing part");
    // The same geometry retains its f64 precision in 3MF.
    save(
        &directory.path().join("part.3mf"),
        mesh,
        ExportFormat::ThreeMf,
    )
    .unwrap();
}

#[test]
fn rejects_vertices_that_collapse_at_stl_precision() {
    rejected_stl_preserves_destination(&tetrahedron(vec![
        [1e6, 0., 0.],
        [1e6 + 0.001, 0., 0.],
        [1e6, 1., 0.],
        [1e6, 0., 1.],
    ]));
}

#[test]
fn rejects_distinct_vertices_that_become_collinear_at_stl_precision() {
    rejected_stl_preserves_destination(&tetrahedron(vec![
        [0., 0., 0.],
        [1., 1., 0.],
        [2., 2. + 1e-8, 0.],
        [0., 0., 1.],
    ]));
}

#[test]
fn rejects_stl_overflow_without_restricting_three_mf_to_f32() {
    rejected_stl_preserves_destination(&tetrahedron(vec![
        [0.; 3],
        [1e304, 0., 0.],
        [0., 1e304, 0.],
        [0., 0., 1e304],
    ]));
    assert!(ExportMesh::new(vec![[f64::MAX, 0., 0.]], &[0, 0, 0]).is_err());
    assert!(ExportMesh::new(vec![[f64::INFINITY, 0., 0.]], &[0, 0, 0]).is_err());
}

#[test]
fn extreme_finite_stl_coordinates_have_finite_unit_normals() {
    let mesh = tetrahedron(vec![
        [0.; 3],
        [1e30, 0., 0.],
        [0., 1e30, 0.],
        [0., 0., 1e30],
    ]);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("part.stl");
    save(&path, &mesh, ExportFormat::Stl).unwrap();
    let bytes = std::fs::read(path).unwrap();
    for triangle in bytes[84..].chunks_exact(50) {
        let values: Vec<_> = triangle[..48]
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        assert!(values.iter().all(|v| v.is_finite()));
        let length = values[..3].iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((length - 1.).abs() < 1e-6);
    }
}

#[test]
fn welding_keeps_quantized_and_extreme_coordinate_keys_distinct() {
    let mesh = tetrahedron(vec![
        [1e291, 0., 0.],
        [1e300, 0., 0.],
        [0., 1e300, 0.],
        [0., 0., 1e300],
    ]);
    assert_eq!(mesh.vertices.len(), 4);
}
