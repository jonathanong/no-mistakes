//! Resolve a source helper's recorded result kind with exact lexical ownership.
use super::{Bindings, HeadProof, ModuleBinding, Summary, Syntax};

pub(in crate::codebase::postgres::query_annotation::heads) fn source_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> Option<Summary> {
    let declaration = (owner.span.start, owner.span.end);
    if !bindings.declaration_safe(declaration)
        || owner.references.iter().any(|usage| {
            let span = (usage.span.start, usage.span.end);
            syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
        })
    {
        return None;
    }
    let Some(recorded) = syntax.record_returns.get(&declaration) else {
        return super::super::finite_returns::builder_summary(syntax, bindings, owner);
    };
    let summary = match *recorded {
        Summary::ReturnedLocal { reference } => {
            let proof = bindings.returned_local_prefix(syntax, reference)?;
            let local = bindings.owner(reference)?;
            let tag = *syntax
                .append_tags
                .get(&(local.span.start, local.span.end))?;
            Summary::Builder { proof, tag }
        }
        summary => summary,
    };
    if let Some(tag) = summary.tag() {
        let tag_owner = bindings.owner(tag)?;
        if !tag_owner.imported || !bindings.trusted_import(tag_owner.id) {
            return None;
        }
    }
    (summary.proof() != HeadProof::Unproven).then_some(summary)
}
