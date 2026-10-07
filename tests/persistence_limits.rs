use confusion::{
    document::schema::Design,
    persistence::container::{load, save},
};
use std::fs;

const LIMIT: usize = 4 * 1024 * 1024;

fn intent_with_size(size: usize) -> Design {
    let mut design = Design::default();
    design.parameter("p", "1 mm".into());
    let baseline = serde_json::to_vec_pretty(&design).unwrap().len();
    design.parameters[0].name = "p".repeat(size - baseline + 1);
    design.validate().unwrap();
    assert_eq!(serde_json::to_vec_pretty(&design).unwrap().len(), size);
    design
}

#[test]
fn oversized_intent_preserves_existing_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("design.con");
    let original = Design::default();
    save(&path, &original).unwrap();
    let bytes = fs::read(&path).unwrap();

    let error = save(&path, &intent_with_size(LIMIT + 1)).unwrap_err();
    assert_eq!(error, "Document entry exceeds size limit");
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(
        serde_json::to_value(load(&path).unwrap()).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn oversized_intent_does_not_create_destination() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("design.con");
    assert!(save(&path, &intent_with_size(LIMIT + 1)).is_err());
    assert!(!path.exists());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn entry_at_limit_roundtrips_with_archive_within_limit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("design.con");
    let design = intent_with_size(LIMIT);
    save(&path, &design).unwrap();
    assert!(fs::metadata(&path).unwrap().len() <= LIMIT as u64);
    let mut archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
    assert_eq!(archive.len(), 2);
    assert_eq!(archive.by_name("design.json").unwrap().size(), LIMIT as u64);
    assert!(archive.by_name("manifest.json").unwrap().size() <= LIMIT as u64);
    assert_eq!(
        serde_json::to_value(load(&path).unwrap()).unwrap(),
        serde_json::to_value(design).unwrap()
    );
}

#[test]
fn normal_intent_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("design.con");
    let mut design = Design::default();
    design.rectangle([0., 0.], [0.05, 0.025]);
    design.sync_construction();
    save(&path, &design).unwrap();
    assert_eq!(
        serde_json::to_value(load(&path).unwrap()).unwrap(),
        serde_json::to_value(design).unwrap()
    );
}
