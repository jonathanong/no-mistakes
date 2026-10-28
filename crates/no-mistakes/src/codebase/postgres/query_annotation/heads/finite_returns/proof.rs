//! Join only complete, fixed return paths of a stable source helper.
use super::{Candidate, Return};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    helper_records::Summary,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::ModuleBinding;

pub(in crate::codebase::postgres::query_annotation::heads) fn head_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> Option<HeadProof> {
    join(syntax, bindings, owner).map(|(proof, _)| proof)
}

pub(in crate::codebase::postgres::query_annotation::heads) fn builder_summary(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> Option<Summary> {
    let (proof, tag) = join(syntax, bindings, owner)?;
    Some(Summary::Builder { proof, tag: tag? })
}

fn join(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> Option<(HeadProof, Option<Span>)> {
    let declaration = (owner.span.start, owner.span.end);
    let Candidate::Returns { heads, abrupt } = syntax.finite_returns.get(&declaration)? else {
        return None;
    };
    if owner.imported || !stable(syntax, bindings, owner) {
        return None;
    }
    for &reference in abrupt {
        let callee = bindings
            .owner(reference)
            .filter(|callee| !callee.imported)?;
        let span = (callee.span.start, callee.span.end);
        if !stable(syntax, bindings, callee)
            || !matches!(syntax.finite_returns.get(&span), Some(Candidate::ThrowOnly))
        {
            return None;
        }
    }
    let mut joined = None;
    let mut builder = None;
    let mut all_builders = true;
    for &value in heads {
        let (proof, tag) = returning(syntax, bindings, value)?;
        if proof == HeadProof::Unproven || joined.is_some_and(|prior| prior != proof) {
            return None;
        }
        joined = Some(proof);
        if builder.is_none() {
            builder = tag;
        }
        all_builders &= tag.is_some();
    }
    Some((joined?, all_builders.then_some(builder).flatten()))
}

fn stable(syntax: &Syntax, bindings: &Bindings<'_>, owner: &ModuleBinding) -> bool {
    bindings.declaration_safe((owner.span.start, owner.span.end))
        && owner.references.iter().all(|usage| {
            let span = (usage.span.start, usage.span.end);
            usage.runtime
                && bindings.declaration_safe(span)
                && !syntax.assignments.writes.contains_key(&span)
                && !syntax.assignments.unsupported_writes.contains(&span)
        })
}

fn returning(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    value: Return,
) -> Option<(HeadProof, Option<Span>)> {
    if let Head::Reference(reference) = value.head {
        let proof = bindings.returned_local_prefix(syntax, reference)?;
        let local = bindings.owner(reference)?;
        let declaration = (local.span.start, local.span.end);
        return Some((proof, Some(*syntax.append_tags.get(&declaration)?)));
    }
    if let Some(tag) = value.tag {
        let owner = bindings.owner(tag)?;
        if !owner.imported || !bindings.trusted_import(owner.id) {
            return None;
        }
    }
    let proof = match value.head {
        Head::AppendPrefixTag => HeadProof::Literal,
        Head::Text(value) if value.opaque => value.current,
        head => head.proof(false),
    };
    Some((proof, value.tag))
}
