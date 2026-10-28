//! Exact, bounded comment-name projection; no SQL variant composition.
use super::TemplateUse;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    HeadProof, Span,
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
    if !syntax.collect_finite
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
    let mut found = BTreeMap::new();
    for candidate in &syntax.finite_templates {
        if executors.contains(&candidate.call) && direct(syntax, bindings, module, candidate) {
            found.insert(candidate.call, HeadProof::Literal);
        }
    }
    found.extend(super::scalar::proofs(syntax, bindings, module, executors));
    for root in syntax
        .finite_roots
        .iter()
        .filter(|root| root.scalar.is_none())
    {
        if !syntax.calls.iter().any(|call| {
            executors.contains(&call.span)
                && matches!(call.head, Head::Reference(reference)
                if bindings.owner(reference).is_some_and(|owner|
                    (owner.span.start, owner.span.end) == root.declaration))
        }) {
            continue;
        }
        let Some(values) = super::domain::helper_domain(syntax, bindings, module, root) else {
            continue;
        };
        if !all_named(&root.before, &root.after, &values) {
            continue;
        }
        for call in &syntax.calls {
            if !executors.contains(&call.span) || !matches!(call.head, Head::Reference(_)) {
                continue;
            }
            let Some(reference) = call.head.reference() else {
                continue;
            };
            if syntax.contexts.get(&reference).copied().flatten() == Some(root.function)
                && bindings.finite_root_accepts(syntax, root.declaration, reference)
            {
                found.insert(call.span, HeadProof::Literal);
            }
        }
    }
    found
}

fn direct(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    candidate: &TemplateUse,
) -> bool {
    let Some(owner) = bindings.owner(candidate.reference) else {
        return false;
    };
    let declaration = (owner.span.start, owner.span.end);
    declaration == candidate.domain.declaration
        && !owner.imported
        && owner.runtime
        && super::domain::stable(syntax, bindings, module, owner)
        && all_named(
            &candidate.before,
            &candidate.after,
            &candidate.domain.values,
        )
}

pub(super) fn all_named(before: &str, after: &str, values: &[String]) -> bool {
    !values.is_empty()
        && values.len() <= 16
        && values
            .iter()
            .all(|value| super::super::syntax::literals::named(&format!("{before}{value}{after}")))
}
