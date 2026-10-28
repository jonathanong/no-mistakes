//! Exact namespace destructure candidates for prepared helper-head projection.
use super::{bindings::Bindings, syntax::Syntax, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
mod export;
mod index;
pub(super) use export::project;

#[derive(Clone)]
pub(super) struct Candidate {
    pub local: Span,
    pub namespace: Span,
    pub exported: String,
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub span: Span,
    pub specifier: String,
    pub exported: String,
}

pub(super) fn pending(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> Vec<Pending> {
    if syntax.namespace_destructures.is_empty() {
        return Vec::new();
    }
    let Some(module) = module else {
        return Vec::new();
    };
    let indexes = index::NamespaceIndexes::build(syntax, module, executors);
    let mut pending = Vec::new();
    for candidate in &syntax.namespace_destructures {
        let Some(namespace) = bindings
            .owner(candidate.namespace)
            .filter(|owner| owner.imported)
        else {
            continue;
        };
        if namespace.references.len() != 1
            || !namespace.references[0].runtime
            || ref_span(&namespace.references[0]) != candidate.namespace
        {
            continue;
        }
        let Some(import) = indexes.namespace_import(namespace.id) else {
            continue;
        };
        let Some(local) = indexes.top_level_local(candidate.local) else {
            continue;
        };
        if !bindings.declaration_safe(candidate.local)
            || indexes.local_is_exported(&local.name)
            || local.references.is_empty()
        {
            continue;
        }
        if !local.references.iter().all(|usage| {
            let span = ref_span(usage);
            usage.runtime
                && bindings.declaration_safe(span)
                && bindings
                    .owner(span)
                    .is_some_and(|owner| owner.id == local.id)
                && !syntax.assignments.writes.contains_key(&span)
                && !syntax.assignments.unsupported_writes.contains(&span)
                && indexes.helper_calls.contains_key(&span)
        }) {
            continue;
        }
        for usage in &local.references {
            let reference = ref_span(usage);
            if bindings
                .owner(reference)
                .is_some_and(|owner| owner.id == local.id)
            {
                for &call in indexes.helper_calls.get(&reference).into_iter().flatten() {
                    pending.push(Pending {
                        span: call,
                        specifier: import.to_string(),
                        exported: candidate.exported.clone(),
                    });
                }
            }
        }
    }
    pending.sort_unstable_by_key(|value| value.span);
    pending.dedup_by_key(|value| value.span);
    pending
}

fn ref_span(value: &crate::codebase::ts_source::module_facts::BindingReference) -> Span {
    (value.span.start, value.span.end)
}
