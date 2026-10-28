//! Module-local successful and failed origin proofs are shared by export aliases.
use super::{source, Pending, Summary};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

pub(in super::super) fn collect(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> (BTreeMap<String, Summary>, Vec<Pending>) {
    if syntax.async_returns.is_empty() && syntax.async_callers.is_empty() {
        return Default::default();
    }
    let Some(module) = module else {
        return Default::default();
    };
    let roots = module
        .scopes
        .iter()
        .filter(|s| s.parent_id.is_none())
        .map(|s| s.id)
        .collect::<FxHashSet<_>>();
    let mut locals = FxHashMap::default();
    for owner in module
        .bindings
        .iter()
        .filter(|b| b.runtime && !b.imported && roots.contains(&b.scope_id))
    {
        locals
            .entry(owner.name.as_str())
            .and_modify(|entry| *entry = None)
            .or_insert(Some(owner));
    }
    let mut dynamic = syntax.dynamic_refs.clone();
    dynamic.sort_unstable();
    let mut awaits = syntax.async_awaits.clone();
    awaits.sort_unstable();
    let pending_effects = bindings
        .effects()
        .pending
        .iter()
        .map(|pending| (pending.builder, pending))
        .collect::<FxHashMap<_, _>>();
    let events = super::source::Events {
        dynamic,
        awaits,
        pending_effects,
    };
    let mut exported_ids = FxHashSet::default();
    let mut export_visits = 0;
    let mut memo = FxHashMap::default();
    let mut exports = BTreeMap::new();
    let mut names = FxHashSet::default();
    let mut ambiguous = FxHashSet::default();
    for export in module.exports.iter().filter(|e| !e.type_only) {
        export_visits += 1;
        if export.specifier.is_none() {
            if let Some(owner) = locals.get(export.local.as_str()).copied().flatten() {
                exported_ids.insert(owner.id);
            }
        }
        if !names.insert(export.exported.as_str()) {
            exports.remove(&export.exported);
            ambiguous.insert(export.exported.as_str());
        }
        if ambiguous.contains(export.exported.as_str()) || export.specifier.is_some() {
            continue;
        }
        let Some(owner) = locals.get(export.local.as_str()).copied().flatten() else {
            continue;
        };
        let Some(candidate) = syntax
            .async_returns
            .get(&(owner.span.start, owner.span.end))
        else {
            continue;
        };
        let proof = memo
            .entry(owner.id)
            .or_insert_with(|| source::proof(syntax, bindings, owner, candidate, &events));
        if let Some(proof) = proof {
            exports.insert(export.exported.clone(), proof.clone());
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.async_export_index_visits",
            export_visits,
        );
    }
    let mut imports = FxHashMap::default();
    for import in module.imports.iter().filter(|i| !i.type_only) {
        for binding in import
            .bindings
            .iter()
            .filter(|b| !b.type_only && b.imported != "*")
        {
            imports
                .entry(binding.binding_id)
                .and_modify(|item| *item = None)
                .or_insert(Some((import.specifier.as_str(), binding.imported.as_str())));
        }
    }
    let mut pending = Vec::new();
    let mut callee_safe = FxHashMap::default();
    for caller in syntax
        .async_callers
        .iter()
        .filter(|c| executors.contains(&c.call))
    {
        if !bindings.declaration_safe(caller.body) || events.dynamic_in(caller.body) {
            continue;
        }
        let Some(owner) = bindings.owner(caller.reference).filter(|b| !b.imported) else {
            continue;
        };
        if (owner.span.start, owner.span.end) != caller.declaration
            || exported_ids.contains(&owner.id)
            || owner.references.iter().filter(|u| u.runtime).count() != 1
            || syntax.assignments.writes.contains_key(&caller.reference)
            || syntax
                .assignments
                .unsupported_writes
                .contains(&caller.reference)
            || syntax.contexts.get(&caller.declaration) != syntax.contexts.get(&caller.reference)
        {
            continue;
        }
        let Some(callee) = bindings.owner(caller.callee).filter(|b| b.imported) else {
            continue;
        };
        if !*callee_safe.entry(callee.id).or_insert_with(|| {
            let mut visits = 0;
            let safe = !callee.references.iter().any(|u| {
                visits += 1;
                let span = (u.span.start, u.span.end);
                !bindings.declaration_safe(span)
                    || syntax.assignments.writes.contains_key(&span)
                    || syntax.assignments.unsupported_writes.contains(&span)
            });
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.async_import_censuses", 1);
                observer.increment("postgres.annotation.async_import_reference_visits", visits);
            }
            safe
        }) {
            continue;
        }
        let Some((specifier, imported)) = imports.get(&callee.id).copied().flatten() else {
            continue;
        };
        pending.push(Pending {
            span: caller.call,
            specifier: specifier.into(),
            imported: imported.into(),
        });
    }
    (exports, pending)
}
