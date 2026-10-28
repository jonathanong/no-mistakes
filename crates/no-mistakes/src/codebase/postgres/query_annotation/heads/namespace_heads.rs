//! Exact namespace destructure candidates for prepared helper-head projection.
use super::{bindings::Bindings, syntax::Syntax, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
mod export;
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
    let Some(module) = module else {
        return Vec::new();
    };
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
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
        let mut imports = module
            .imports
            .iter()
            .filter(|import| !import.type_only)
            .flat_map(|import| import.bindings.iter().map(move |binding| (import, binding)))
            .filter(|(_, binding)| {
                binding.binding_id == namespace.id
                    && !binding.type_only
                    && binding.kind == "namespace"
                    && binding.imported == "*"
            });
        let Some((import, _)) = imports.next() else {
            continue;
        };
        if imports.next().is_some() {
            continue;
        }
        let mut locals = module.bindings.iter().filter(|owner| {
            !owner.imported
                && owner.runtime
                && roots.contains(&owner.scope_id)
                && (owner.span.start, owner.span.end) == candidate.local
        });
        let Some(local) = locals.next() else {
            continue;
        };
        if locals.next().is_some()
            || !bindings.declaration_safe(candidate.local)
            || bindings.exported(local)
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
                && syntax.helper_arguments.iter().any(|(call, reference)| {
                    executors.contains(call)
                        && *reference == span
                        && !syntax.optional_calls.contains(call)
                })
        }) {
            continue;
        }
        for (&call, &reference) in &syntax.helper_arguments {
            if !executors.contains(&call) || syntax.optional_calls.contains(&call) {
                continue;
            }
            if bindings
                .owner(reference)
                .is_some_and(|owner| owner.id == local.id)
            {
                pending.push(Pending {
                    span: call,
                    specifier: import.specifier.clone(),
                    exported: candidate.exported.clone(),
                });
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
