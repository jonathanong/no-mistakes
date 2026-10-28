use super::super::HeadProof;
use super::{Bindings, Head, Span};

impl Bindings<'_> {
    pub fn proof(&self, head: Head, call: Span) -> HeadProof {
        match head {
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
                    return if self.append_safety.accepts(declaration, call) {
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
                if value.tagged()
                    && !self.safe_tags.contains(&declaration)
                    && !self.append_safety.accepts(declaration, call)
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
