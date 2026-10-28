use super::{Bindings, Head, Span, Syntax};
use crate::fx::FxHashSet;

impl Bindings<'_> {
    pub(in crate::codebase::postgres::query_annotation::heads) fn finite_root_accepts(
        &self,
        syntax: &Syntax,
        declaration: Span,
        reference: Span,
    ) -> bool {
        let Some(owner) = self.owner(reference) else {
            return false;
        };
        let Some(&tag) = syntax.append_tags.get(&declaration) else {
            return false;
        };
        let Some(tag_owner) = self.owner(tag) else {
            return false;
        };
        !owner.imported
            && (owner.span.start, owner.span.end) == declaration
            && self.trusted_roots.contains(&declaration)
            && tag_owner.imported
            && self.trusted_import(tag_owner.id)
            && self.append_safety.accepts(declaration, reference)
    }
    pub(in crate::codebase::postgres::query_annotation::heads) fn conditional_prefix(
        &self,
        head: Head,
        call: Span,
    ) -> Option<Vec<Span>> {
        let reference = head.reference()?;
        let owner = self.owner(reference).filter(|owner| !owner.imported)?;
        let declaration = (owner.span.start, owner.span.end);
        if !self.trusted_roots.contains(&declaration)
            || !matches!(
                self.constants.get(&declaration),
                Some(Head::Tag | Head::AppendTag | Head::AppendPrefixTag)
            )
        {
            return None;
        }
        self.append_safety.conditional(declaration, call)
    }
    pub(in crate::codebase::postgres::query_annotation::heads) fn returned_local_prefix(
        &self,
        syntax: &Syntax,
        reference: Span,
    ) -> Option<super::super::HeadProof> {
        use super::super::HeadProof;
        let owner = self.owner(reference).filter(|owner| !owner.imported)?;
        let declaration = (owner.span.start, owner.span.end);
        let context = syntax.contexts.get(&declaration).copied().flatten()?;
        let closed_return = self.returned_prefix_roots.contains(&declaration)
            && syntax.return_sinks.contains(&reference)
            && declaration.1 < reference.0;
        if syntax.contexts.get(&reference).copied().flatten() != Some(context)
            || !(closed_return || self.append_safety.accepts(declaration, reference))
        {
            return None;
        }
        let root = *self.constants.get(&declaration)?;
        if !matches!(root, Head::Tag | Head::AppendTag | Head::AppendPrefixTag) {
            return None;
        }
        let tag = *syntax.append_tags.get(&declaration)?;
        let tag_owner = self.owner(tag)?;
        (tag_owner.imported && self.trusted_import(tag_owner.id)).then_some(HeadProof::Literal)
    }
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
        legacy: bool,
    ) -> Option<super::super::HeadProof> {
        use super::super::HeadProof;
        match head {
            Head::AppendPrefixTag => Some(HeadProof::Literal),
            Head::Text(value) if value.opaque => Some(value.current),
            Head::Reference(reference) | Head::AppendPrefixReference(reference) => {
                let owner = self.owner(reference).filter(|owner| !owner.imported)?;
                let declaration = (owner.span.start, owner.span.end);
                let accepts = if legacy {
                    self.append_safety.accepts_legacy(declaration, call)
                } else {
                    self.append_safety.accepts(declaration, call)
                };
                if !accepts {
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
