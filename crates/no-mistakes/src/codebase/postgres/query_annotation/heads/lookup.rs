use super::{AnnotationHeads, HeadProof, Span};

impl AnnotationHeads {
    pub(crate) fn has_pending_imports(&self) -> bool {
        !self.pending_imports.is_empty()
            || !self.pending_async_returns.is_empty()
            || !self.pending_namespaces.is_empty()
            || !self.pending_effects.is_empty()
            || !self.pending_callbacks.is_empty()
            || !self.pending_returned_params.is_empty()
            || !self.import_gated_finite.is_empty()
    }
    pub(crate) fn helper(&self, span: Span, helpers: &[String]) -> Option<HeadProof> {
        if helpers.is_empty() {
            return None;
        }
        let proof = self.helper_calls.as_ref()?.get(&span)?;
        helpers.contains(&proof.identity).then_some(proof.exact)
    }
    pub(crate) fn annotation_helper(&self, span: Span, helpers: &[String]) -> Option<HeadProof> {
        if let Some(prefix) = self.annotation_helper_prefix(span) {
            Some(super::text::annotation_prefix(prefix))
        } else {
            let proof = self.helper_calls.as_ref()?.get(&span)?;
            if builtin(&proof.identity) {
                Some(proof.exact)
            } else {
                self.helper(span, helpers)
            }
        }
    }
    pub(crate) fn annotation_helper_prefix(&self, span: Span) -> Option<&str> {
        let proof = self.helper_calls.as_ref()?.get(&span)?;
        builtin(&proof.identity)
            .then_some(proof.prefix.as_deref())
            .flatten()
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

fn builtin(identity: &str) -> bool {
    matches!(identity, "pg-copy-streams#from" | "pg-copy-streams#to")
}
