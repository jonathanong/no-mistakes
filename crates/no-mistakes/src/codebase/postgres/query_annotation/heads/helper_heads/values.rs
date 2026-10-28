//! A direct exported string primitive has no mutable SQL builder identity.
use super::{Bindings, HeadProof, Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;

pub(super) fn proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> Option<HeadProof> {
    let declaration = (owner.span.start, owner.span.end);
    if !syntax.string_constants.contains(&declaration)
        || !bindings.declaration_safe(declaration)
        || owner.references.iter().any(|usage| {
            let span = (usage.span.start, usage.span.end);
            syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
        })
    {
        return None;
    }
    let head = *syntax.constants.get(&declaration)?;
    let proof = head.proof(false);
    (proof != HeadProof::Unproven).then_some(proof)
}
