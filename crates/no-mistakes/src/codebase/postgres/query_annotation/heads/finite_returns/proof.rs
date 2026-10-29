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

pub(in crate::codebase::postgres::query_annotation::heads) fn primitive_head_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
    mut stability: impl FnMut(&ModuleBinding) -> bool,
) -> Option<HeadProof> {
    let span = (owner.span.start, owner.span.end);
    let Candidate::Returns { heads, .. } = syntax.finite_returns.get(&span)? else {
        return None;
    };
    if !heads.iter().all(|head| head.primitive) {
        return None;
    }
    join_with_stability(syntax, bindings, owner, &mut stability).map(|(proof, _)| proof)
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
    join_with_stability(syntax, bindings, owner, &mut |owner| {
        stable(syntax, bindings, owner)
    })
}

fn join_with_stability(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
    stability: &mut impl FnMut(&ModuleBinding) -> bool,
) -> Option<(HeadProof, Option<Span>)> {
    let declaration = (owner.span.start, owner.span.end);
    let Candidate::Returns { heads, abrupt } = syntax.finite_returns.get(&declaration)? else {
        return None;
    };
    if owner.imported || !stability(owner) {
        return None;
    }
    for &reference in abrupt {
        let callee = bindings
            .owner(reference)
            .filter(|callee| !callee.imported)?;
        let span = (callee.span.start, callee.span.end);
        if !stability(callee)
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
    stable_with_visit(syntax, bindings, owner, || {})
}

pub(in crate::codebase::postgres::query_annotation::heads) fn primitive_stable(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
) -> bool {
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment("postgres.annotation.primitive_stability_censuses", 1);
    }
    let result = stable_with_visit(syntax, bindings, owner, || {
        if let Some(observer) = &observer {
            observer.increment("postgres.annotation.primitive_stability_refs_visited", 1);
        }
    });
    if let Some(observer) = observer {
        observer.increment(
            if result {
                "postgres.annotation.primitive_stability_positive_censuses"
            } else {
                "postgres.annotation.primitive_stability_negative_censuses"
            },
            1,
        );
    }
    result
}

fn stable_with_visit(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
    mut visit: impl FnMut(),
) -> bool {
    bindings.declaration_safe((owner.span.start, owner.span.end))
        && owner.references.iter().all(|usage| {
            visit();
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
