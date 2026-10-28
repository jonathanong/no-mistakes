//! Bounded same-file composition of prefix-preserving parameter effects.
use super::{CallSyntax, FunctionSyntax, Signature, Span};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::FxHashMap;

type EffectId = (usize, u8);
type Candidate = Option<(Signature, Vec<EffectId>)>;

struct Input<'a> {
    summary: &'a FunctionSyntax,
    callable: &'a ModuleBinding,
    parameter: &'a ModuleBinding,
}

pub(super) fn proven(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    declarations: &FxHashMap<Span, Option<&ModuleBinding>>,
) -> FxHashMap<EffectId, Signature> {
    let calls = syntax
        .effect_calls
        .iter()
        .map(|call| (call.builder, call))
        .collect::<FxHashMap<_, _>>();
    let mut dynamic_refs = syntax.dynamic_refs.clone();
    dynamic_refs.sort_unstable();
    let mut candidates: FxHashMap<EffectId, Candidate> = FxHashMap::default();
    for (&declaration, summaries) in &syntax.effect_functions {
        let Some(callable) = declarations.get(&declaration).copied().flatten() else {
            continue;
        };
        for summary in summaries.iter().flatten() {
            let Some(parameter) = declarations.get(&summary.parameter).copied().flatten() else {
                continue;
            };
            let dependencies = dependencies(
                syntax,
                bindings,
                Input {
                    summary,
                    callable,
                    parameter,
                },
                &dynamic_refs,
                &calls,
            );
            candidates
                .entry((callable.id, summary.position))
                .and_modify(|entry| *entry = None)
                .or_insert(dependencies.map(|required| (summary.signature, required)));
        }
    }
    let mut proven = FxHashMap::default();
    // Four local edges are enough for the inventoried helper chain. A cycle
    // cannot seed itself, and deeper or imported call graphs stay unresolved.
    for _ in 0..=4 {
        let next = candidates
            .iter()
            .filter_map(|(&key, candidate)| {
                let (signature, deps) = candidate.as_ref()?;
                (!proven.contains_key(&key)
                    && deps.iter().all(|dep| {
                        proven
                            .get(dep)
                            .is_some_and(|known: &Signature| known.plain())
                    }))
                .then_some((key, *signature))
            })
            .collect::<Vec<_>>();
        if next.is_empty() {
            break;
        }
        proven.extend(next);
    }
    proven
}

fn dependencies(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    input: Input<'_>,
    dynamic_refs: &[Span],
    calls: &FxHashMap<Span, &CallSyntax>,
) -> Option<Vec<EffectId>> {
    let Input {
        summary,
        callable,
        parameter,
    } = input;
    if !callable.runtime
        || callable.imported
        || !parameter.runtime
        || parameter.imported
        || !bindings.declaration_safe((callable.span.start, callable.span.end))
        || !bindings.declaration_safe(summary.parameter)
        || dynamic_refs
            .get(dynamic_refs.partition_point(|span| span.0 < summary.body.0))
            .is_some_and(|span| span.0 < summary.body.1)
        || callable.references.iter().any(|usage| {
            let span = (usage.span.start, usage.span.end);
            !bindings.declaration_safe(span)
                || syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
        })
    {
        return None;
    }
    let mut required = Vec::new();
    let valid = parameter
        .references
        .iter()
        .filter(|usage| usage.runtime)
        .all(|usage| {
            let span = (usage.span.start, usage.span.end);
            let context = syntax.contexts.get(&span).copied().flatten();
            let own_context = context == Some(summary.function);
            let captured_context = context.is_some_and(|function| {
                function != summary.function
                    && summary.body.0 <= function.0
                    && function.1 <= summary.body.1
            });
            if !bindings.declaration_safe(span)
                || !(own_context || captured_context)
                || syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
            {
                return false;
            }
            if syntax.discarded_appends.contains(&span)
                || syntax.prefix_discarded_appends.contains(&span)
            {
                return true;
            }
            // A captured callback may run later, but a discarded append can
            // only add a suffix. Its returned builder or a helper call could
            // escape the parameter, so those stay in the owning function.
            if !own_context {
                return false;
            }
            if summary.position == 0 && syntax.returned_effect_appends.contains(&span) {
                return true;
            }
            let Some(call) = calls.get(&span) else {
                return false;
            };
            let Some(callee) = bindings.owner(call.callee) else {
                return false;
            };
            if callee.imported {
                return false;
            }
            required.push((callee.id, call.position));
            true
        });
    valid.then_some(required)
}
