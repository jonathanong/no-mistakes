//! Exact, bounded comment-name projection; no SQL variant composition.
use super::TemplateUse;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
use std::collections::BTreeMap;

pub(in crate::codebase::postgres::query_annotation::heads) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> BTreeMap<Span, HeadProof> {
    let Some(module) = module else {
        return BTreeMap::new();
    };
    if executors.is_empty()
        || !syntax.collect_finite
        || (!syntax
            .finite_templates
            .iter()
            .any(|v| executors.contains(&v.call))
            && syntax.finite_roots.is_empty())
        || !module.diagnostics.is_empty()
        || !syntax.dynamic_refs.is_empty()
    {
        return BTreeMap::new();
    }
    let mut index = super::index::Index::new(syntax, bindings, module, executors);
    let mut found = BTreeMap::new();
    for candidate in &syntax.finite_templates {
        if executors.contains(&candidate.call) && direct(syntax, bindings, &mut index, candidate) {
            found.insert(candidate.call, HeadProof::Literal);
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
    found
}

fn direct(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    index: &mut super::index::Index<'_>,
    candidate: &TemplateUse,
) -> bool {
    let Some(owner) = bindings.owner(candidate.reference) else {
        return false;
    };
    let declaration = (owner.span.start, owner.span.end);
    declaration == candidate.domain.declaration
        && !owner.imported
        && owner.runtime
        && index.stable(syntax, bindings, owner)
        && all_named(
            &candidate.before,
            &candidate.after,
            &candidate.domain.values,
        )
}

fn all_named(before: &str, after: &str, values: &[String]) -> bool {
    !values.is_empty()
        && values.len() <= 16
        && values
            .iter()
            .all(|value| super::super::syntax::literals::named(&format!("{before}{value}{after}")))
}
