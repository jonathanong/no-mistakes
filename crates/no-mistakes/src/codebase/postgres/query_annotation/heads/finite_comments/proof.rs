//! Exact, bounded comment-name projection; no SQL variant composition.
use super::TemplateUse;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

pub(in crate::codebase::postgres::query_annotation::heads) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> (BTreeMap<Span, HeadProof>, FxHashSet<Span>) {
    let Some(module) = module else {
        return (BTreeMap::new(), FxHashSet::default());
    };
    if executors.is_empty()
        || !syntax.collect_finite
        || (syntax.finite_templates.is_empty() && syntax.finite_roots.is_empty())
        || !module.diagnostics.is_empty()
        || !syntax.dynamic_refs.is_empty()
    {
        return (BTreeMap::new(), FxHashSet::default());
    }
    let const_uses = build_const_uses(syntax, bindings, executors);
    if syntax.finite_roots.is_empty()
        && !syntax.finite_templates.iter().any(|candidate| {
            executors.contains(&candidate.call) || const_uses.contains_key(&candidate.call)
        })
    {
        return (BTreeMap::new(), FxHashSet::default());
    }
    let mut index = super::index::Index::new(syntax, bindings, module, executors);
    let mut found = BTreeMap::new();
    let mut gated = FxHashSet::default();
    for candidate in &syntax.finite_templates {
        let direct_call = executors.contains(&candidate.call);
        if !direct_call && !const_uses.contains_key(&candidate.call) {
            continue;
        }
        let named = if candidate.inline {
            Some(all_named(
                &candidate.before,
                &candidate.after,
                &candidate.domain.values,
            ))
        } else {
            direct(syntax, bindings, &mut index, candidate)
        };
        if let Some(named) = named {
            if !direct_call {
                if !named {
                    for &call in &const_uses[&candidate.call] {
                        found.insert(call, HeadProof::Unproven);
                    }
                }
                continue;
            }
            let proof = if named {
                HeadProof::Literal
            } else {
                HeadProof::Unproven
            };
            found.insert(candidate.call, proof);
            if named && (candidate.inline || candidate.domain.import_gated) {
                gated.insert(candidate.call);
            }
        }
    }
    for root in &syntax.finite_roots {
        let Some(calls) = index.endpoints.get(&root.declaration) else {
            continue;
        };
        if calls.is_empty() {
            continue;
        }
        let Some(values) = index.helper_domain(syntax, bindings, root) else {
            continue;
        };
        if !all_named(&root.before, &root.after, values) {
            continue;
        }
        for &(call, reference) in &index.endpoints[&root.declaration] {
            index.work.endpoints += 1;
            if syntax.contexts.get(&reference).copied().flatten() == Some(root.function)
                && bindings.finite_root_accepts(syntax, root.declaration, reference)
            {
                found.insert(call, HeadProof::Literal);
            }
        }
    }
    index.report();
    (found, gated)
}

fn build_const_uses(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
) -> FxHashMap<Span, Vec<Span>> {
    let const_candidates: FxHashSet<_> = syntax
        .finite_templates
        .iter()
        .map(|candidate| candidate.call)
        .filter(|span| syntax.constants.contains_key(span))
        .collect();
    let mut const_uses = FxHashMap::<Span, Vec<Span>>::default();
    if !const_candidates.is_empty() {
        for call in syntax
            .calls
            .iter()
            .filter(|call| executors.contains(&call.span))
        {
            let Head::Reference(reference) = call.head else {
                continue;
            };
            if let Some(owner) = bindings.owner(reference).filter(|owner| !owner.imported) {
                let declaration = (owner.span.start, owner.span.end);
                if const_candidates.contains(&declaration) {
                    const_uses.entry(declaration).or_default().push(call.span);
                }
            }
        }
    }
    const_uses
}

fn direct(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    index: &mut super::index::Index<'_>,
    candidate: &TemplateUse,
) -> Option<bool> {
    let owner = bindings.owner(candidate.reference)?;
    let declaration = (owner.span.start, owner.span.end);
    (declaration == candidate.domain.declaration
        && !owner.imported
        && owner.runtime
        && index.stable(syntax, bindings, owner))
    .then(|| {
        all_named(
            &candidate.before,
            &candidate.after,
            &candidate.domain.values,
        )
    })
}

fn all_named(before: &str, after: &str, values: &[String]) -> bool {
    !values.is_empty()
        && values.len() <= 16
        && values
            .iter()
            .all(|value| super::super::syntax::literals::named(&format!("{before}{value}{after}")))
}
