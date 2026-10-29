use super::{AnnotationHeads, HeadProof, Span};

impl AnnotationHeads {
    pub(crate) fn has_pending_imports(&self) -> bool {
        !self.pending_imports.is_empty()
            || !self.pending_async_returns.is_empty()
            || !self.pending_namespaces.is_empty()
            || !self.pending_effects.is_empty()
            || !self.pending_callbacks.is_empty()
            || !self.pending_returned_params.is_empty()
    }
    pub(crate) fn helper(&self, span: Span, helpers: &[String]) -> Option<HeadProof> {
        if helpers.is_empty() {
            return None;
        }
        let (identity, proof) = self.helper_calls.as_ref()?.get(&span)?;
        helpers.contains(identity).then_some(*proof)
    }
    pub(crate) fn accepts(&self, span: Span) -> bool {
        matches!(self.calls.get(&span), Some(HeadProof::Literal))
    }
    pub(crate) fn proves_missing(&self, span: Span) -> bool {
        matches!(self.calls.get(&span), Some(HeadProof::Missing))
    }
    pub(crate) fn accepts_trusted_append(&self, span: Span) -> bool {
        self.trusted_append_calls.contains(&span)
    }
}
