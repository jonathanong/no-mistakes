//! Paired configured reads indexed once; every result use must be admitted.
use super::lifetime::Memo;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
#[derive(Clone, Copy)]
struct Endpoint {
    text: Span,
    values: Option<Span>,
    seen: bool,
}
pub(in super::super::super) fn demanded(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    executors: &FxHashSet<Span>,
) -> bool {
    if !syntax.dynamic_refs.is_empty() {
        return false;
    }
    syntax.calls.iter().any(|call| {
        executors.contains(&call.span)
            && syntax.record_result_calls.contains(&call.span)
            && match call.head {
                Head::FieldReference(reference) => bindings.owner(reference).is_some_and(|owner| {
                    !owner.imported
                        && syntax
                            .returned_param_roots
                            .contains_key(&(owner.span.start, owner.span.end))
                }),
                _ => false,
            }
    })
}
pub(in super::super::super) fn projected(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
    scopes: &FxHashSet<usize>,
) -> Vec<(Span, usize, HeadProof)> {
    if syntax.returned_param_roots.is_empty()
        || executors.is_empty()
        || !syntax.dynamic_refs.is_empty()
        || !module.diagnostics.is_empty()
    {
        return Vec::new();
    }
    let mut endpoints = FxHashMap::default();
    let mut groups: FxHashMap<Span, Vec<Span>> = FxHashMap::default();
    for call in &syntax.calls {
        if !executors.contains(&call.span) || !syntax.record_result_calls.contains(&call.span) {
            continue;
        }
        let Head::FieldReference(reference) = call.head else {
            continue;
        };
        let Some(owner) = bindings.owner(reference) else {
            continue;
        };
        let declaration = (owner.span.start, owner.span.end);
        if owner.imported || !syntax.returned_param_roots.contains_key(&declaration) {
            continue;
        }
        groups.entry(declaration).or_default().push(call.span);
        endpoints.insert(
            call.span,
            Endpoint {
                text: reference,
                values: None,
                seen: false,
            },
        );
    }
    if endpoints.is_empty() {
        return Vec::new();
    }
    for (&reference, call) in &syntax.value_reads {
        if let Some(endpoint) = endpoints.get_mut(call) {
            endpoint.values = (!endpoint.seen).then_some(reference);
            endpoint.seen = true;
        }
    }
    let exports = module
        .exports
        .iter()
        .filter(|item| !item.type_only && item.specifier.is_none())
        .map(|item| item.local.as_str())
        .collect::<FxHashSet<_>>();
    let mut memo = Memo::default();
    let mut result = Vec::new();
    for (declaration, calls) in groups {
        let root = &syntax.returned_param_roots[&declaration];
        let Some(owner) = bindings.owner(endpoints[&calls[0]].text) else {
            continue;
        };
        if scopes.contains(&owner.scope_id) && exports.contains(owner.name.as_str()) {
            continue;
        }
        let reads = calls
            .iter()
            .map(|call| {
                let endpoint = endpoints[call];
                Some((*call, endpoint.text, endpoint.values?))
            })
            .collect::<Option<Vec<_>>>();
        let Some(reads) = reads else {
            continue;
        };
        let proof = if let Some(proof) = memo.owners.get(&owner.id) {
            *proof
        } else {
            let proof = memo.prove(syntax, bindings, owner, root, &reads);
            memo.owners.insert(owner.id, proof);
            proof
        };
        if let Some(proof) = proof {
            result.extend(
                reads
                    .iter()
                    .map(|(call, _, _)| (*call, proof.callee, proof.head)),
            );
        }
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.returned_caller_indexes", 1);
    }
    memo.report();
    result.sort_unstable_by_key(|entry| entry.0);
    result
}
