//! Canonical current-state fingerprint; view state and in-memory undo do not mark a design dirty.
use super::schema::Design;
pub fn fingerprint(design: &Design) -> blake3::Hash {
    let mut canonical = design.clone();
    if let Some(primary) = canonical.primary_sketch_id() {
        canonical
            .activate_sketch(primary)
            .expect("Valid active sketch");
        canonical.active_sketch = Some(primary);
        canonical.sketches.sort_by_key(|sketch| sketch.id);
    }
    let value = serde_json::to_value(&canonical).expect("Document is serializable");
    blake3::hash(&serde_json::to_vec(&value).expect("Document is serializable"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn switching_active_sketch_is_view_state() {
        let mut d = Design::default();
        d.rectangle([0., 0.], [0.04, 0.02]);
        let primary = d.current_sketch_id().unwrap();
        d.create_sketch(crate::document::model::SketchPlane::Xy)
            .unwrap();
        d.rectangle([0., 0.], [0.01, 0.01]);
        let hash = fingerprint(&d);
        d.activate_sketch(primary).unwrap();
        assert_eq!(fingerprint(&d), hash);
    }
    #[test]
    fn edits_and_undo_return_to_saved_state() {
        let saved = Design::default();
        let hash = fingerprint(&saved);
        let mut edited = saved.clone();
        edited.rectangle([0., 0.], [0.04, 0.02]);
        assert_ne!(fingerprint(&edited), hash);
        assert_eq!(fingerprint(&saved.clone()), hash);
    }
}
