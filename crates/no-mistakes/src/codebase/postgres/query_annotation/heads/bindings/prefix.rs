use super::{Bindings, Head, Span, Syntax};
use crate::fx::FxHashSet;

impl Bindings<'_> {
    pub(in crate::codebase::postgres::query_annotation::heads) fn trusted_import(
        &self,
        id: usize,
    ) -> bool {
        self.trusted_imports.contains(&id)
    }
    pub(super) fn trusted_roots(&self, syntax: &Syntax) -> FxHashSet<Span> {
        syntax
            .append_tags
            .iter()
            .filter_map(|(declaration, tag)| {
                self.owner(*tag)
                    .filter(|owner| owner.imported && self.trusted_import(owner.id))
                    .map(|_| *declaration)
            })
            .collect()
    }
    pub(in crate::codebase::postgres::query_annotation::heads) fn prefix_candidate(
        &self,
        head: Head,
        call: Span,
    ) -> Option<super::super::HeadProof> {
        use super::super::HeadProof;
        match head {
            Head::AppendPrefixTag => Some(HeadProof::Literal),
            Head::Text(value) if value.opaque => Some(value.current),
            Head::Reference(reference) | Head::AppendPrefixReference(reference) => {
                let owner = self.owner(reference).filter(|owner| !owner.imported)?;
                let declaration = (owner.span.start, owner.span.end);
                if !self.append_safety.accepts(declaration, call) {
                    return None;
                }
                let root = *self.constants.get(&declaration)?;
                if self.append_safety.opaque_before(declaration, call) {
                    return match root {
                        Head::Tag | Head::AppendTag | Head::AppendPrefixTag => {
                            Some(HeadProof::Literal)
                        }
                        Head::Text(value) => Some(value.continued),
                        _ => None,
                    };
                }
                match root {
                    Head::AppendPrefixTag => Some(HeadProof::Literal),
                    Head::Text(value) if value.opaque => Some(value.current),
                    Head::Tag | Head::AppendTag
                        if matches!(head, Head::AppendPrefixReference(_)) =>
                    {
                        Some(HeadProof::Literal)
                    }
                    Head::Text(value) if matches!(head, Head::AppendPrefixReference(_)) => {
                        Some(value.continued)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
