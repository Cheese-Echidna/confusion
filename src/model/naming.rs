//! Persistent face references use native feature/boundary provenance.
//! Resolution and history tracking live in kernel/native/naming.inc. Split or
//! merged references require explicit repair rather than an ordinal fallback.

#[cfg(feature = "kernel")]
pub fn bind_legacy_references(
    design: &mut crate::document::schema::Design,
    references: &[crate::kernel::bridge::ffi::BoundFaceReference],
) -> bool {
    let mut changed = false;
    for reference in references {
        let Ok(id) = uuid::Uuid::parse_str(&reference.edit) else {
            continue;
        };
        let Some(edit) = design.solid_edits.iter_mut().find(|edit| edit.id == id) else {
            continue;
        };
        if edit.face_reference.is_none() && !reference.face.is_empty() {
            edit.face_reference = Some(reference.face.clone());
            changed = true;
        }
        if edit.tool_reference.is_none() && !reference.tool.is_empty() {
            edit.tool_reference = Some(reference.tool.clone());
            changed = true;
        }
    }
    changed
}
