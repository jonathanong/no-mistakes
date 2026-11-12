use super::{Bindings, HeadProof, Root, Span, Syntax};
use crate::codebase::postgres::query_annotation::heads::syntax::Head;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) fn collect(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
) -> Vec<(Span, HeadProof)> {
    let candidates = &syntax.fresh_builders;
    if candidates.declarations.is_empty() {
        return Vec::new();
    }
    let allowed = syntax
        .calls
        .iter()
        .filter_map(|call| match call.head {
            Head::Reference(reference) if executors.contains(&call.span) => Some((reference, call)),
            _ => None,
        })
        .collect::<FxHashMap<_, _>>();
    let mut result = Vec::new();
    for binding in &module.bindings {
        let declaration = (binding.span.start, binding.span.end);
        let Some(initial) = candidates.declarations.get(&declaration) else {
            continue;
        };
        let Some(function) = initial.context.function else {
            continue;
        };
        if binding.imported
            || !binding.runtime
            || !bindings.declaration_safe(initial.span)
            || !trusted(&initial.root, bindings)
        {
            continue;
        }
        let mut calls = Vec::new();
        let valid = binding
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .all(|usage| {
                let reference = (usage.span.start, usage.span.end);
                if !bindings.declaration_safe(reference)
                    || syntax.assignments.unsupported_writes.contains(&reference)
                    || syntax
                        .contexts
                        .get(&reference)
                        .copied()
                        .flatten()
                        .map(|span| span.0)
                        != Some(function)
                {
                    return false;
                }
                if let Some(write) = syntax.assignments.writes.get(&reference) {
                    return write.context.function == Some(function)
                        && bindings.declaration_safe(write.span)
                        && candidates
                            .writes
                            .get(&reference)
                            .is_some_and(|root| trusted(root, bindings));
                }
                if let Some(call) = allowed.get(&reference) {
                    if call.span.0 <= initial.span.1 {
                        return false;
                    }
                    calls.push(call.span);
                    return true;
                }
                syntax.discarded_appends.contains(&reference)
                    || syntax.prefix_discarded_appends.contains(&reference)
                    || bindings.allowed_effect(reference)
            });
        if valid {
            result.extend(calls.into_iter().map(|call| (call, HeadProof::Literal)));
        }
    }
    result
}
fn trusted(root: &Root, bindings: &Bindings<'_>) -> bool {
    root.tags.iter().all(|&tag| {
        bindings
            .owner(tag)
            .is_some_and(|owner| owner.imported && bindings.trusted_import(owner.id))
    })
}
