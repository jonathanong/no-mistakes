use super::{QueryCapture, ScopeVisitor};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn has_capture_key(
        &self,
        scope: u32,
        name: &str,
    ) -> bool {
        self.member_query_proof
            .capture_indices
            .get(&scope)
            .is_some_and(|names| names.contains_key(name))
    }

    pub(super) fn named_query_capture(&self, name: &str) -> Option<&QueryCapture> {
        let scope = self.capture_scope(name)?;
        let index = self
            .member_query_proof
            .capture_indices
            .get(&scope)?
            .get(name)?
            .last()?;
        self.member_query_proof.captures.get(*index)
    }
}
