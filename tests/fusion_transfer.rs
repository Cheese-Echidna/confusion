use confusion::{
    document::schema::{Design, Extrusion},
    exchange::import::load_fusion_transfer,
};

#[test]
fn editable_transfer_preserves_named_expressions_and_driving_dimensions() {
    let mut design = Design::default();
    design.rectangle([0., 0.], [0.02, 0.03]);
    let width = design.parameter("width", "20 mm".into());
    let depth = design.parameter("depth", "width / 4".into());
    let dimension = design
        .constraints
        .iter()
        .find(|c| c.kind.parameter() == Some(width))
        .unwrap()
        .id;
    design.extrusion = Some(Extrusion {
        id: uuid::Uuid::new_v4(),
        depth,
        boundary: vec![],
    });
    design.sync_construction();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.fusion.json");
    let transfer =
        serde_json::json!({"format":"confusion-fusion-transfer", "version":1, "design":design});
    std::fs::write(&path, serde_json::to_vec(&transfer).unwrap()).unwrap();
    let mut imported = load_fusion_transfer(&path).unwrap();
    assert!(
        imported
            .constraints
            .iter()
            .any(|c| c.id == dimension && c.kind.parameter() == Some(width))
    );
    assert_eq!(
        imported
            .parameters
            .iter()
            .find(|p| p.id == depth)
            .unwrap()
            .expression,
        "width / 4"
    );
    assert_eq!(
        confusion::parameters::expression::evaluate(&imported).unwrap()[&depth],
        0.005
    );
    imported.parameters[0].expression = "40 mm".into();
    assert_eq!(
        confusion::parameters::expression::evaluate(&imported).unwrap()[&depth],
        0.01
    );
    let saved = dir.path().join("part.con");
    confusion::persistence::container::save(&saved, &imported).unwrap();
    let reopened = confusion::persistence::container::load(&saved).unwrap();
    assert_eq!(
        reopened
            .constraints
            .iter()
            .find(|c| c.id == dimension)
            .unwrap()
            .kind
            .parameter(),
        Some(width)
    );
}
#[test]
fn invalid_expression_or_transfer_version_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.fusion.json");
    let mut design = Design::default();
    design.parameter("unsupported", "sin(20 deg)".into());
    let transfer =
        serde_json::json!({"format":"confusion-fusion-transfer", "version":1, "design":design});
    std::fs::write(&path, serde_json::to_vec(&transfer).unwrap()).unwrap();
    assert!(load_fusion_transfer(&path).is_err());
    let transfer = serde_json::json!({"format":"confusion-fusion-transfer", "version":2, "design":Design::default()});
    std::fs::write(&path, serde_json::to_vec(&transfer).unwrap()).unwrap();
    assert!(load_fusion_transfer(&path).is_err());
}
#[test]
fn native_archive_is_not_misreported_as_a_con_design() {
    let error =
        confusion::exchange::import::load_design(std::path::Path::new("example.F3D")).unwrap_err();
    assert!(error.contains("Native Fusion archives"));
    assert!(error.contains(".fusion.json"));
}

#[test]
fn python_adapter_fixture_loads_without_losing_circle_dimension_links() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fusion-circle.fusion.json");
    let design = load_fusion_transfer(&path).unwrap();
    assert_eq!(design.circles.len(), 1);
    assert_eq!(design.features.len(), 1);
    let values = confusion::parameters::expression::evaluate(&design).unwrap();
    assert_eq!(values[&design.features[0].depth], 0.005);
    assert_eq!(
        design.constraints[0].kind.parameter(),
        Some(design.parameters[1].id)
    );
}

#[cfg(all(feature = "solver", feature = "kernel"))]
#[test]
fn transferred_circle_extrusion_re_evaluates_after_parameter_edit_and_exports_3mf() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fusion-circle.fusion.json");
    let mut design = load_fusion_transfer(&path).unwrap();
    let before = confusion::evaluation::solid::evaluate(&design, || false)
        .unwrap()
        .mesh
        .unwrap();
    assert!((before.volume - std::f64::consts::PI * 0.01 * 0.01 * 0.005).abs() < 1e-12);
    design
        .parameters
        .iter_mut()
        .find(|p| p.name == "width")
        .unwrap()
        .expression = "40 mm".into();
    let after = confusion::evaluation::solid::evaluate(&design, || false)
        .unwrap()
        .mesh
        .unwrap();
    assert!((after.volume - before.volume * 8.).abs() < 1e-12);
    let dir = tempfile::tempdir().unwrap();
    confusion::exchange::export::export_design(
        &dir.path().join("circle.3mf"),
        &design,
        confusion::exchange::export::ExportFormat::ThreeMf,
    )
    .unwrap();
}
