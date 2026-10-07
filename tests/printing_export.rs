use confusion::exchange::export::{ExportFormat, ExportMesh, save};
use std::io::Read;

fn tetrahedron() -> ExportMesh {
    ExportMesh::new(
        vec![[0., 0., 0.], [0.01, 0., 0.], [0., 0.01, 0.], [0., 0., 0.01]],
        &[0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3],
    )
    .unwrap()
}
#[test]
fn binary_stl_has_millimetres_normals_and_triangle_count() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.stl");
    save(&path, &tetrahedron(), ExportFormat::Stl).unwrap();
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(bytes.len(), 84 + 4 * 50);
    assert_eq!(u32::from_le_bytes(bytes[80..84].try_into().unwrap()), 4);
    assert_eq!(f32::from_le_bytes(bytes[92..96].try_into().unwrap()), -1.);
    assert_eq!(f32::from_le_bytes(bytes[112..116].try_into().unwrap()), 10.);
}
#[test]
fn three_mf_has_core_package_relationship_and_explicit_units() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.3mf");
    save(&path, &tetrahedron(), ExportFormat::ThreeMf).unwrap();
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(zip.len(), 3);
    let mut model = String::new();
    zip.by_name("3D/3dmodel.model")
        .unwrap()
        .read_to_string(&mut model)
        .unwrap();
    assert!(model.contains("unit=\"millimeter\""));
    assert!(model.contains("x=\"10\""));
    assert_eq!(model.matches("<triangle ").count(), 4);
    let mut rels = String::new();
    zip.by_name("_rels/.rels")
        .unwrap()
        .read_to_string(&mut rels)
        .unwrap();
    assert!(rels.contains("Target=\"/3D/3dmodel.model\""));
    assert!(zip.by_name("[Content_Types].xml").is_ok());
}
#[test]
fn invalid_mesh_preserves_existing_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.stl");
    std::fs::write(&path, b"existing").unwrap();
    let mesh = ExportMesh {
        vertices: vec![],
        triangles: vec![[0, 1, 2]],
    };
    assert!(save(&path, &mesh, ExportFormat::Stl).is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"existing");
    assert!(ExportMesh::new(vec![[f64::NAN, 0., 0.]], &[0, 0, 0]).is_err());
    assert!(ExportMesh::new(vec![[0.; 3]], &[0, 0]).is_err());
}
#[test]
fn duplicated_face_vertices_are_welded_without_changing_winding() {
    let mesh = ExportMesh::new(
        vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., -0., 0.]],
        &[3, 1, 2],
    )
    .unwrap();
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.triangles, [[0, 1, 2]]);
}
#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn export_evaluates_current_parameters_and_rejects_empty_design() {
    use confusion::document::schema::{Design, Extrusion};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.stl");
    assert!(
        confusion::exchange::export::export_design(&path, &Design::default(), ExportFormat::Stl)
            .is_err()
    );
    let mut design = Design::default();
    design.rectangle([0., 0.], [0.02, 0.03]);
    let depth = design.parameter("depth", "5 mm".into());
    design.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary: vec![],
    });
    design.sync_construction();
    confusion::exchange::export::export_design(&path, &design, ExportFormat::Stl).unwrap();
    let bytes = std::fs::read(path).unwrap();
    let max_z = bytes[84..]
        .as_chunks::<50>()
        .0
        .iter()
        .flat_map(|t| {
            [24, 36, 48].map(|offset| f32::from_le_bytes(t[offset - 4..offset].try_into().unwrap()))
        })
        .fold(f32::NEG_INFINITY, f32::max);
    assert_eq!(max_z, 5.);
}

#[test]
fn open_or_inconsistently_wound_mesh_is_not_exported() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.3mf");
    let mut mesh = tetrahedron();
    mesh.triangles.pop();
    assert!(save(&path, &mesh, ExportFormat::ThreeMf).is_err());
    assert!(!path.exists());
    let mut mesh = tetrahedron();
    mesh.triangles[0].swap(0, 1);
    assert!(save(&path, &mesh, ExportFormat::Stl).is_err());
}
