//! A candidate can commit only into the document and editor that submitted it.
use crate::document::{dirty::fingerprint, schema::Design};
pub struct CandidateEdit {
    pub revision: u64,
    base: blake3::Hash,
    context: String,
    design: Design,
}
impl CandidateEdit {
    pub fn new(revision: u64, base: &Design, context: String, design: Design) -> Self {
        Self {
            revision,
            base: fingerprint(base),
            context,
            design,
        }
    }
    pub fn is_current(&self, base: &Design, context: &str) -> bool {
        self.context == context && self.base == fingerprint(base)
    }
    pub fn finish(
        self,
        revision: u64,
        success: bool,
        base: &Design,
        context: &str,
    ) -> Option<Design> {
        (success && revision == self.revision && self.is_current(base, context))
            .then_some(self.design)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn candidate() -> (Design, CandidateEdit) {
        let base = Design::default();
        let mut edited = base.clone();
        edited.rectangle([0., 0.], [0.02, 0.01]);
        let candidate = CandidateEdit::new(3, &base, "create-box".into(), edited);
        (base, candidate)
    }
    #[test]
    fn failure_stale_revision_changed_document_and_changed_editor_cannot_commit() {
        let (base, edit) = candidate();
        assert!(edit.finish(3, false, &base, "create-box").is_none());
        let (base, edit) = candidate();
        assert!(edit.finish(2, true, &base, "create-box").is_none());
        let (base, edit) = candidate();
        assert!(edit.finish(3, true, &base, "modify-shell").is_none());
        let (mut base, edit) = candidate();
        base.rectangle([0., 0.], [0.04, 0.03]);
        assert!(edit.finish(3, true, &base, "create-box").is_none());
    }
    #[test]
    fn success_returns_candidate_without_mutating_base() {
        let (base, edit) = candidate();
        let original = fingerprint(&base);
        let committed = edit.finish(3, true, &base, "create-box").unwrap();
        assert_ne!(fingerprint(&committed), original);
        assert_eq!(fingerprint(&base), original);
    }
}
