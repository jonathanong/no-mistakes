//! Full result references only for demanded, same-factory native caller roots.
use crate::codebase::postgres::query_annotation::heads::{
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
pub(in super::super::super) fn extend(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
    trusted: &FxHashSet<usize>,
    wanted: &mut FxHashSet<Span>,
) {
    if syntax.returned_param_roots.is_empty()
        || !syntax.dynamic_refs.is_empty()
        || !module.diagnostics.is_empty()
    {
        return;
    }
    let endpoints = syntax
        .calls
        .iter()
        .filter(|call| {
            executors.contains(&call.span) && syntax.record_result_calls.contains(&call.span)
        })
        .filter_map(|call| match call.head {
            Head::FieldReference(reference) => Some(reference),
            _ => None,
        })
        .collect::<FxHashSet<_>>();
    if endpoints.is_empty() {
        return;
    }
    let leaves = syntax
        .returned_param_roots
        .values()
        .flat_map(|root| root.references())
        .collect::<FxHashSet<_>>();
    let mut owners: FxHashMap<Span, Option<&ModuleBinding>> = FxHashMap::default();
    let mut lookups = 0;
    for binding in &module.bindings {
        for usage in binding.references.iter().filter(|usage| usage.runtime) {
            lookups += 1;
            let span = (usage.span.start, usage.span.end);
            if endpoints.contains(&span) || leaves.contains(&span) {
                owners
                    .entry(span)
                    .and_modify(|owner| *owner = None)
                    .or_insert(Some(binding));
            }
        }
    }
    let mut expanded = FxHashSet::default();
    let mut eligible = FxHashMap::default();
    let mut visits = 0;
    for reference in endpoints {
        let Some(owner) = owners.get(&reference).copied().flatten() else {
            continue;
        };
        let declaration = (owner.span.start, owner.span.end);
        let Some(root) = syntax.returned_param_roots.get(&declaration) else {
            continue;
        };
        let valid = *eligible.entry(owner.id).or_insert_with(|| {
            let mut callee = None;
            root.leaves.iter().all(|leaf| {
                let Some(actual) = owners.get(&leaf.callee).copied().flatten() else {
                    return false;
                };
                let Some(tag) = owners.get(&leaf.tag).copied().flatten() else {
                    return false;
                };
                let valid = actual.imported
                    && callee.is_none_or(|id| id == actual.id)
                    && tag.imported
                    && trusted.contains(&tag.id);
                callee = Some(actual.id);
                valid
            })
        });
        if !valid || expanded.contains(&declaration) {
            continue;
        }
        expanded.insert(declaration);
        for usage in &owner.references {
            visits += 1;
            if usage.runtime {
                wanted.insert((usage.span.start, usage.span.end));
            }
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.returned_caller_wanted_owners",
            expanded.len() as u64,
        );
        observer.increment(
            "postgres.annotation.returned_caller_wanted_references",
            visits,
        );
        observer.increment(
            "postgres.annotation.returned_caller_wanted_lookup_visits",
            lookups,
        );
    }
}
