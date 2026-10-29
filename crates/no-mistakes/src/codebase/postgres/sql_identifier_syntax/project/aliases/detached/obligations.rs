use super::*;

pub(super) fn collect(
    profile: &EmbeddedSqlOptions,
    module: &TypeScriptModuleFacts,
    collector: &Collector<'_>,
    values: &FxHashMap<usize, Value>,
    selection: &mut Selection,
) {
    let mut direct = FxHashSet::default();
    for call in &collector.calls {
        let Some(id) = collector.owners.get(&call.root) else {
            continue;
        };
        let Some(value) = values.get(id) else {
            continue;
        };
        if value.kind == Kind::Executor && call.parts.len() == 1 {
            direct.insert(call.root);
        }
    }
    for member in &collector.members {
        if profile.executor_names.contains(&member.name)
            && collector
                .owners
                .get(&member.root)
                .and_then(|id| values.get(id))
                .is_some_and(|value| value.kind == Kind::Receiver)
            && !collector.direct_callees.contains(&member.span)
        {
            if collector.invoked.contains(&member.span) {
                selection.reference_obligations.insert(member.span);
            }
            selection.reference_receivers.insert(member.root);
        }
    }
    for binding in &module.bindings {
        if !values
            .get(&binding.id)
            .is_some_and(|value| value.kind == Kind::Executor)
        {
            continue;
        }
        for usage in binding.references.iter().filter(|usage| usage.runtime) {
            let at = (usage.span.start, usage.span.end);
            if !direct.contains(&at) && collector.invoked.contains(&at) {
                selection.reference_obligations.insert(at);
            }
        }
    }
}
