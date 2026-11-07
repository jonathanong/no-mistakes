use super::*;
use crate::fx::FxHashSet;

pub(super) fn proven(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    flow: &ValueFlowFacts,
    reference: (&str, usize, u32, &[String]),
) -> bool {
    let (name, scope, offset, members) = reference;
    let Some(context) = flow.calls.iter().find_map(|value_call| {
        ((value_call.offset, value_call.end) == call.span).then_some(value_call.context)
    }) else {
        return false;
    };
    let mut reference = (name, scope, offset, context);
    let mut seen = FxHashSet::default();
    loop {
        let (name, scope, offset, context) = reference;
        if flow.finite_identifier_guards.iter().any(|guard| {
            guard.name == name
                && (if matches!(guard.kind, GuardKind::Lookup) {
                    bindings
                        .value(&call.file, scope, name, offset)
                        .is_some_and(|binding| bounded(flow, binding, members, true))
                } else {
                    guard.members.as_slice() == members
                })
                && guard.context == context
                && offset >= guard.from
                && offset < guard.until
                && guard_subject_holds(flow, bindings, &call.file, name, scope, offset, guard)
                && !written_since(flow, guard, offset)
                && guard_applies(graph, opts, facts, bindings, &call.file, guard)
        }) {
            return true;
        }
        if !members.is_empty() {
            return false;
        }
        let Some(binding) = bindings.value(&call.file, scope, name, offset) else {
            return false;
        };
        if !binding.immutable
            || !seen.insert((binding.scope, binding.name.as_str(), binding.declared_at))
        {
            return false;
        }
        let ValueExpression::Named {
            name,
            scope,
            offset,
            context,
        } = &binding.value
        else {
            return false;
        };
        reference = (name, scope.unwrap_or(0), *offset, *context);
    }
}

#[cfg(test)]
#[path = "guard_aliases_tests.rs"]
mod tests;
