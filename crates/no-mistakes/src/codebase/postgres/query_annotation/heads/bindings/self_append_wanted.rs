//! Demand only native let roots read by confirmed direct executor endpoints.
use super::{Span, Syntax};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn extend_if_low(
    low: bool,
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
    bindings: &super::Bindings<'_>,
    wanted: &mut FxHashSet<Span>,
) {
    if low {
        extend(syntax, module, executors, &bindings.trusted_imports, wanted);
    }
}

pub(super) fn extend(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
    trusted: &FxHashSet<usize>,
    wanted: &mut FxHashSet<Span>,
) {
    if syntax.self_appends.roots.is_empty() || trusted.is_empty() {
        return;
    }
    let endpoints = syntax
        .self_appends
        .reads
        .iter()
        .filter_map(|(call, reference)| {
            (executors.contains(call) && !syntax.repeated_calls.contains(call))
                .then_some(*reference)
        })
        .collect::<FxHashSet<_>>();
    if endpoints.is_empty() {
        return;
    }
    let tags = syntax
        .self_appends
        .roots
        .values()
        .map(|root| root.tag)
        .collect::<FxHashSet<_>>();
    let mut owners: FxHashMap<Span, Option<&ModuleBinding>> = FxHashMap::default();
    let mut lookups = 0;
    // One coarse canonical-fact pass, never a binding scan per endpoint/root.
    for binding in &module.bindings {
        for usage in binding.references.iter().filter(|usage| usage.runtime) {
            lookups += 1;
            let span = (usage.span.start, usage.span.end);
            if endpoints.contains(&span) || tags.contains(&span) {
                owners
                    .entry(span)
                    .and_modify(|owner| *owner = None)
                    .or_insert(Some(binding));
            }
        }
    }
    let mut expanded = FxHashSet::default();
    let mut visits = 0;
    for reference in endpoints {
        let Some(owner) = owners.get(&reference).copied().flatten() else {
            continue;
        };
        let declaration = (owner.span.start, owner.span.end);
        let Some(root) = syntax.self_appends.roots.get(&declaration) else {
            continue;
        };
        let Some(tag) = owners.get(&root.tag).copied().flatten() else {
            continue;
        };
        if owner.imported
            || !tag.imported
            || !trusted.contains(&tag.id)
            || !expanded.insert(declaration)
        {
            continue;
        }
        wanted.insert(root.tag);
        for usage in &owner.references {
            visits += 1;
            if usage.runtime {
                wanted.insert((usage.span.start, usage.span.end));
            }
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.self_append_wanted_owner_visits",
            expanded.len() as u64,
        );
        observer.increment(
            "postgres.annotation.self_append_wanted_reference_visits",
            visits,
        );
        observer.increment(
            "postgres.annotation.self_append_wanted_lookup_visits",
            lookups,
        );
    }
}
