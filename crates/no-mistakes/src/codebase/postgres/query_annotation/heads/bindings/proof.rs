use super::super::HeadProof;
use super::{Bindings, Head, Span};

impl Bindings<'_> {
    pub fn proof(&self, head: Head, call: Span) -> HeadProof {
        self.proof_mode(head, call, false)
    }
    pub(in crate::codebase::postgres::query_annotation::heads) fn proof_legacy(
        &self,
        head: Head,
        call: Span,
    ) -> HeadProof {
        self.proof_mode(head, call, true)
    }
    fn proof_mode(&self, head: Head, call: Span, legacy: bool) -> HeadProof {
        let accepts = |declaration| {
            if legacy {
                self.append_safety.accepts_legacy(declaration, call)
            } else {
                self.append_safety.accepts(declaration, call)
            }
        };
        match head {
            Head::FieldReference(span) => {
                if legacy {
                    return HeadProof::Unproven;
                }
                let Some(owner) = self.owner(span).filter(|owner| !owner.imported) else {
                    return HeadProof::Unproven;
                };
                let declaration = (owner.span.start, owner.span.end);
                if !self.trusted_roots.contains(&declaration)
                    || !self.append_safety.accepts(declaration, call)
                {
                    return HeadProof::Unproven;
                }
                self.constants
                    .get(&declaration)
                    .copied()
                    .map_or(HeadProof::Unproven, |root| match root {
                        Head::AppendPrefixTag => HeadProof::Literal,
                        Head::Text(value) if value.opaque => value.current,
                        _ => root.proof(self.append_safety.continued(declaration, call)),
                    })
            }
            Head::Reference(span) | Head::AppendReference(span) => {
                let Some(owner) = self.owner(span).filter(|owner| !owner.imported) else {
                    return HeadProof::Unproven;
                };
                let declaration = (owner.span.start, owner.span.end);
                let Some(value) = self.constants.get(&declaration).copied() else {
                    return self
                        .mutable
                        .calls
                        .get(&span)
                        .copied()
                        .unwrap_or(HeadProof::Unproven);
                };
                if matches!(head, Head::AppendReference(_))
                    || value.appended()
                    || (value.tagged() && self.append_safety.touched(declaration))
                {
                    return if accepts(declaration) {
                        if !legacy && self.append_safety.closed_prefix(declaration) {
                            return HeadProof::Literal;
                        }
                        if self.append_safety.opaque_before(declaration, call) {
                            return HeadProof::Unproven;
                        }
                        value.proof(
                            matches!(head, Head::AppendReference(_))
                                || self.append_safety.continued(declaration, call),
                        )
                    } else {
                        HeadProof::Unproven
                    };
                }
                if value.tagged() && !self.safe_tags.contains(&declaration) && !accepts(declaration)
                {
                    HeadProof::Unproven
                } else {
                    value.proof(false)
                }
            }
            value => value.proof(false),
        }
    }
}
