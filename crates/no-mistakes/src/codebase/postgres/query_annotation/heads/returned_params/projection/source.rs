//! Exact binding and dynamic-scope checks for the returned identity summary.
use super::super::Candidate;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, Span,
};
use crate::codebase::ts_source::module_facts::ModuleBinding;

pub(super) fn source_proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    callable: &ModuleBinding,
    candidate: &Candidate,
    dynamic: &[Span],
) -> bool {
    let declaration = (callable.span.start, callable.span.end);
    if !bindings.declaration_safe(declaration)
        || !bindings.declaration_safe(candidate.function)
        || callable.references.iter().any(|usage| {
            let span = (usage.span.start, usage.span.end);
            !bindings.declaration_safe(span)
                || syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
        })
        || dynamic
            .get(dynamic.partition_point(|span| span.0 < candidate.body.0))
            .is_some_and(|span| span.0 < candidate.body.1)
    {
        return false;
    }
    let Some(tag) = bindings.owner(candidate.default_tag) else {
        return false;
    };
    if !tag.imported || !bindings.trusted_import(tag.id) {
        return false;
    }
    let Some(parameter) = bindings.owner(candidate.allowed_parameter) else {
        return false;
    };
    let Some(alias) = bindings.owner(candidate.allowed_alias[0]) else {
        return false;
    };
    if (parameter.span.start, parameter.span.end) != candidate.parameter
        || (alias.span.start, alias.span.end) != candidate.alias
    {
        return false;
    }
    exact_references(
        syntax,
        bindings,
        parameter,
        &[candidate.allowed_parameter],
        candidate.function,
    ) && exact_references(
        syntax,
        bindings,
        alias,
        &candidate.allowed_alias,
        candidate.function,
    )
}

fn exact_references(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    owner: &ModuleBinding,
    allowed: &[Span],
    function: Span,
) -> bool {
    let declaration = (owner.span.start, owner.span.end);
    bindings.declaration_safe(declaration)
        && owner
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .count()
            == allowed.len()
        && owner
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .all(|usage| {
                let span = (usage.span.start, usage.span.end);
                allowed.contains(&span)
                    && bindings
                        .owner(span)
                        .is_some_and(|actual| actual.id == owner.id)
                    && syntax.contexts.get(&span) == Some(&Some(function))
                    && !syntax.assignments.writes.contains_key(&span)
                    && !syntax.assignments.unsupported_writes.contains(&span)
            })
}
