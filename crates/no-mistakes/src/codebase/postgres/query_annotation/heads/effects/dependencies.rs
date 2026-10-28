//! Complete parameter and callable census, shared by all calls to an owner.
use super::{CallSyntax, FunctionSyntax, Pending, Span};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::FxHashMap;

pub(super) struct Input<'a> {
    pub summary: &'a FunctionSyntax,
    pub callable: &'a ModuleBinding,
    pub parameter: &'a ModuleBinding,
}
pub(super) struct Dependencies {
    pub local: Vec<(usize, u8)>,
    pub imported: Option<Pending>,
    pub strict: bool,
}
pub(super) fn dependencies(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    input: Input<'_>,
    dynamic_refs: &[Span],
    calls: &FxHashMap<Span, &CallSyntax>,
    imports: &mut super::imported::Imports<'_>,
) -> Option<Dependencies> {
    if !eligible(syntax, bindings, &input, dynamic_refs) {
        return None;
    }
    let Input {
        summary,
        callable,
        parameter,
    } = input;
    let mut required = Vec::new();
    let mut imported = None;
    let mut strict = bindings.declaration_safe(summary.function)
        && callable.references.iter().all(|usage| {
            let span = (usage.span.start, usage.span.end);
            usage.runtime
                && imports.calls.contains(&span)
                && bindings
                    .owner(span)
                    .is_some_and(|owner| owner.id == callable.id)
        });
    let valid = parameter
        .references
        .iter()
        .filter(|usage| usage.runtime)
        .all(|usage| {
            let span = (usage.span.start, usage.span.end);
            let context = syntax.contexts.get(&span).copied().flatten();
            let own_context = context == Some(summary.function);
            strict &= own_context
                && bindings
                    .owner(span)
                    .is_some_and(|owner| owner.id == parameter.id);
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
                strict = false;
                return true;
            }
            let Some(call) = calls.get(&span) else {
                return false;
            };
            let Some(callee) = bindings.owner(call.callee) else {
                return false;
            };
            if callee.imported {
                if summary.position != 0 || imported.is_some() {
                    return false;
                }
                let Some(obligation) = imports.dependency(syntax, bindings, callee, call) else {
                    return false;
                };
                imported = Some(obligation);
                return true;
            }
            required.push((callee.id, call.position));
            true
        });
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.effect_parameter_censuses", 1);
    }
    valid.then_some(Dependencies {
        local: required,
        imported,
        strict,
    })
}

fn eligible(
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
    !(!callable.runtime
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
        }))
}
