//! Prove the SQL head only; values observations are allowed effects, never scalars.
use super::{ownership::Ownership, Span};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Call, Head, Syntax},
    HeadProof,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

pub(in super::super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> BTreeMap<Span, HeadProof> {
    let mut found = BTreeMap::new();
    let Some(module) = module else {
        return found;
    };
    if !syntax.native_values.demanded()
        || executors.is_empty()
        || !module.diagnostics.is_empty()
        || !syntax.dynamic_refs.is_empty()
    {
        return found;
    }
    let observations = syntax.native_values.allowed();
    let observed = observations
        .iter()
        .filter_map(|&reference| bindings.owner(reference).map(|owner| owner.id))
        .collect::<FxHashSet<_>>();
    let mut reads: FxHashMap<usize, (&ModuleBinding, Vec<&Call>)> = FxHashMap::default();
    let mut read_references = FxHashSet::default();
    for call in syntax.calls.iter().filter(|call| {
        executors.contains(&call.span) && !syntax.optional_calls.contains(&call.span)
    }) {
        if let Head::Reference(reference) = call.head {
            if let Some(owner) = bindings
                .owner(reference)
                .filter(|owner| observed.contains(&owner.id))
            {
                reads
                    .entry(owner.id)
                    .or_insert_with(|| (owner, Vec::new()))
                    .1
                    .push(call);
                read_references.insert(reference);
            }
        }
    }
    if reads.is_empty() {
        return found;
    }
    let mut ownership = Ownership::new(module);
    for (owner, calls) in reads.values() {
        let declaration = (owner.span.start, owner.span.end);
        let Some(context) = syntax.contexts.get(&declaration).copied().flatten() else {
            continue;
        };
        let Some(head) = syntax.constants.get(&declaration).copied() else {
            continue;
        };
        if owner.imported
            || ownership.module_heap(owner)
            || !bindings.trusted_root(declaration)
            || !continued(head)
            || !bindings.declaration_safe(declaration)
            || !calls
                .iter()
                .all(|call| call.context.function == Some(context.0))
        {
            continue;
        }
        let Some(&tag) = syntax.append_tags.get(&declaration) else {
            continue;
        };
        if !ownership.constructor(syntax, bindings, tag) {
            continue;
        }
        let mut visits = 0;
        let closed = owner.references.iter().all(|usage| {
            visits += 1;
            let reference = (usage.span.start, usage.span.end);
            usage.runtime
                && bindings
                    .owner(reference)
                    .is_some_and(|actual| actual.id == owner.id)
                && syntax.contexts.get(&reference).copied().flatten() == Some(context)
                && !syntax.assignments.writes.contains_key(&reference)
                && !syntax.assignments.unsupported_writes.contains(&reference)
                && (observations.contains(&reference)
                    || syntax.discarded_appends.contains(&reference)
                    || syntax.prefix_discarded_appends.contains(&reference)
                    || read_references.contains(&reference))
        });
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.native_value_owner_censuses", 1);
            observer.increment(
                "postgres.annotation.native_value_owner_reference_visits",
                visits,
            );
        }
        if closed {
            for call in calls {
                found.insert(call.span, HeadProof::Literal);
            }
        }
    }
    found
}
fn continued(head: Head) -> bool {
    match head {
        Head::Tag | Head::AppendTag | Head::AppendPrefixTag => true,
        Head::Text(value) if value.tagged => value.continued == HeadProof::Literal,
        _ => false,
    }
}
