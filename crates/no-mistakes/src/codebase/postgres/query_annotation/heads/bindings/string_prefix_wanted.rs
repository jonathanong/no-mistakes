//! Expand only primitive += owners demanded by confirmed executor reads.
use super::{Span, Syntax};
use crate::codebase::postgres::query_annotation::heads::syntax::Head;
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn extend(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
    wanted: &mut FxHashSet<Span>,
) {
    if syntax.string_heads.is_empty() || syntax.string_appends.is_empty() {
        return;
    }
    let endpoints = syntax
        .calls
        .iter()
        .filter(|call| {
            executors.contains(&call.span) && !syntax.optional_calls.contains(&call.span)
        })
        .filter_map(|call| match call.head {
            Head::Reference(reference) => Some(reference),
            _ => None,
        })
        .collect::<FxHashSet<_>>();
    if endpoints.is_empty() {
        return;
    }
    let mut owners: FxHashMap<Span, Option<&ModuleBinding>> = FxHashMap::default();
    let mut lookups = 0;
    // Canonical facts once, never all bindings for each root/read.
    for binding in &module.bindings {
        for usage in binding.references.iter().filter(|usage| usage.runtime) {
            lookups += 1;
            let span = (usage.span.start, usage.span.end);
            if endpoints.contains(&span) || syntax.string_appends.contains(&span) {
                owners
                    .entry(span)
                    .and_modify(|owner| *owner = None)
                    .or_insert(Some(binding));
            }
        }
    }
    let appended = syntax
        .string_appends
        .iter()
        .filter_map(|span| owners.get(span).copied().flatten().map(|owner| owner.id))
        .collect::<FxHashSet<_>>();
    let mut eligible = FxHashMap::default();
    let mut expanded = FxHashSet::default();
    let mut visits = 0;
    for reference in endpoints {
        let Some(owner) = owners.get(&reference).copied().flatten() else {
            continue;
        };
        let valid = *eligible.entry(owner.id).or_insert_with(|| {
            !owner.imported
                && appended.contains(&owner.id)
                && syntax
                    .string_heads
                    .contains_key(&(owner.span.start, owner.span.end))
        });
        if !valid || !expanded.insert(owner.id) {
            continue;
        }
        for usage in &owner.references {
            visits += 1;
            if usage.runtime {
                wanted.insert((usage.span.start, usage.span.end));
            }
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.string_wanted_owner_visits",
            expanded.len() as u64,
        );
        observer.increment("postgres.annotation.string_wanted_reference_visits", visits);
        observer.increment("postgres.annotation.string_wanted_lookup_visits", lookups);
        observer.increment(
            "postgres.annotation.string_wanted_eligibility_censuses",
            eligible.len() as u64,
        );
    }
}
