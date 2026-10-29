//! Exact return owner and every post-allocation use carry the effect proof.
use super::super::{effects, Span};
use super::{Candidate, Summary};
use crate::fx::FxHashMap;
pub(super) struct Events<'a> {
    pub dynamic: Vec<Span>,
    pub awaits: Vec<Span>,
    pub pending_effects: FxHashMap<Span, &'a effects::Pending>,
}
impl Events<'_> {
    pub fn dynamic_in(&self, span: Span) -> bool {
        inside(&self.dynamic, span)
    }
}
fn inside(events: &[Span], range: Span) -> bool {
    events
        .get(events.partition_point(|span| span.0 < range.0))
        .is_some_and(|span| span.0 < range.1)
}
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;

pub(super) fn proof(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    callable: &ModuleBinding,
    candidate: &Candidate,
    events: &Events<'_>,
) -> Option<Summary> {
    let mut visits = 0;
    let result = prove(syntax, bindings, callable, candidate, events, &mut visits);
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.async_return_censuses", 1);
        observer.increment("postgres.annotation.async_return_reference_visits", visits);
    }
    result
}
fn prove(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    callable: &ModuleBinding,
    candidate: &Candidate,
    events: &Events<'_>,
    visits: &mut u64,
) -> Option<Summary> {
    if !bindings.declaration_safe(candidate.function)
        || callable.references.iter().any(|usage| {
            *visits += 1;
            let span = (usage.span.start, usage.span.end);
            !bindings.declaration_safe(span)
                || syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
        })
        || events.dynamic_in(candidate.body)
        || inside(
            &events.awaits,
            (candidate.initializer_start, candidate.body.1),
        )
    {
        return None;
    }
    let tag = bindings.owner(candidate.tag)?;
    if !tag.imported || !bindings.trusted_import(tag.id) {
        return None;
    }
    let owner = bindings.owner(candidate.returned)?;
    if owner.imported || (owner.span.start, owner.span.end) != candidate.declaration {
        return None;
    }
    let allowed = candidate
        .appends
        .iter()
        .chain(&candidate.effects)
        .copied()
        .chain([candidate.returned])
        .collect::<crate::fx::FxHashSet<_>>();
    let mut count = 0;
    for usage in owner.references.iter().filter(|usage| usage.runtime) {
        *visits += 1;
        let span = (usage.span.start, usage.span.end);
        if !allowed.contains(&span)
            || !bindings.declaration_safe(span)
            || syntax.contexts.get(&span) != Some(&Some(candidate.function))
            || bindings
                .owner(span)
                .is_none_or(|actual| actual.id != owner.id)
            || syntax.assignments.writes.contains_key(&span)
            || syntax.assignments.unsupported_writes.contains(&span)
        {
            return None;
        }
        count += 1;
    }
    if count != allowed.len() {
        return None;
    }
    let mut required = Vec::new();
    for reference in &candidate.effects {
        if bindings.allowed_effect(*reference) {
            continue;
        }
        let pending = events.pending_effects.get(reference)?;
        required.push((*pending).clone());
        if required.len() > 8 {
            return None;
        }
    }
    Some(Summary {
        proof: candidate.proof,
        required,
    })
}
