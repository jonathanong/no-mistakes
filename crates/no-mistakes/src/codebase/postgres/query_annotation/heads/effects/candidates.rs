use super::{CallSyntax, FunctionSyntax, Signature, Span};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::FxHashMap;

pub(super) type EffectId = (usize, u8);
pub(super) enum Requirement {
    Local(EffectId, super::CallShape),
    Import(super::Import),
}
pub(super) type Candidate = (Signature, Vec<Requirement>);
struct Input<'a> {
    summary: &'a FunctionSyntax,
    callable: &'a ModuleBinding,
    parameter: &'a ModuleBinding,
}

pub(super) fn collect(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    declarations: &FxHashMap<Span, Option<&ModuleBinding>>,
    imports: &FxHashMap<usize, (&str, &str)>,
) -> FxHashMap<EffectId, Option<Candidate>> {
    let calls = syntax
        .effect_calls
        .iter()
        .map(|call| (call.builder, call))
        .collect();
    let mut dynamic_refs = syntax.dynamic_refs.clone();
    dynamic_refs.sort_unstable();
    let mut result = FxHashMap::default();
    for (&declaration, summaries) in &syntax.effect_functions {
        let Some(callable) = declarations.get(&declaration).copied().flatten() else {
            continue;
        };
        for summary in summaries.iter().flatten() {
            let Some(parameter) = declarations.get(&summary.parameter).copied().flatten() else {
                continue;
            };
            let required = dependencies(
                syntax,
                bindings,
                Input {
                    summary,
                    callable,
                    parameter,
                },
                &dynamic_refs,
                &calls,
                imports,
            );
            let mut signature = summary.signature;
            signature.returns_builder = summary.returned.is_some()
                && syntax
                    .effect_returns
                    .get(&summary.function)
                    .is_some_and(|values| {
                        !values.is_empty()
                            && values.iter().all(|reference| {
                                reference
                                    .and_then(|reference| bindings.owner(reference))
                                    .is_some_and(|owner| owner.id == parameter.id)
                            })
                    });
            result
                .entry((callable.id, summary.position))
                .and_modify(|entry| *entry = None)
                .or_insert(required.map(|required| (signature, required)));
        }
    }
    result
}
fn dependencies(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    input: Input<'_>,
    dynamic_refs: &[Span],
    calls: &FxHashMap<Span, &CallSyntax>,
    imports: &FxHashMap<usize, (&str, &str)>,
) -> Option<Vec<Requirement>> {
    if !declarations_safe(syntax, bindings, &input, dynamic_refs) {
        return None;
    }
    let Input {
        summary, parameter, ..
    } = input;
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
            let Some(dependency) = calls
                .get(&span)
                .and_then(|call| requirement(call, bindings, imports))
            else {
                return false;
            };
            required.push(dependency);
            true
        });
    valid.then_some(required)
}

fn declarations_safe(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    input: &Input<'_>,
    dynamic_refs: &[Span],
) -> bool {
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
        return false;
    }
    true
}

fn requirement(
    call: &CallSyntax,
    bindings: &Bindings<'_>,
    imports: &FxHashMap<usize, (&str, &str)>,
) -> Option<Requirement> {
    let callee = bindings.owner(call.callee)?;
    if !callee.imported {
        return Some(Requirement::Local((callee.id, call.position), call.shape));
    }
    if call.position != 0 {
        return None;
    }
    let (specifier, imported) = imports.get(&callee.id)?;
    Some(Requirement::Import(super::Import {
        specifier: (*specifier).to_string(),
        imported: (*imported).to_string(),
        shape: call.shape,
    }))
}
