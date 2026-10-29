use super::*;

pub(super) fn collect(
    profile: &EmbeddedSqlOptions,
    module: &TypeScriptModuleFacts,
    collector: &Collector<'_>,
    nodes: &[&Node],
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
    for node in nodes {
        if values
            .get(&node.id)
            .is_some_and(|value| value.kind == Kind::Executor)
        {
            selection.reference_obligations.extend(node.default_span);
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
            selection.reference_obligations.insert(member.span);
            selection.reference_receivers.insert(member.root);
        }
    }
    let mut exports = FxHashMap::<&str, Vec<_>>::default();
    for export in module
        .exports
        .iter()
        .filter(|export| !export.type_only && export.specifier.is_none())
    {
        exports
            .entry(export.local.as_str())
            .or_default()
            .push(export);
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
            if !direct.contains(&at) {
                selection.reference_obligations.insert(at);
            }
        }
        for export in exports
            .get(binding.name.as_str())
            .into_iter()
            .flatten()
            .filter(|_| binding.scope_id == 0)
        {
            let at = (export.span.start, export.span.end);
            if !binding
                .references
                .iter()
                .any(|usage| usage.runtime && at.0 <= usage.span.start && usage.span.end <= at.1)
            {
                selection.reference_obligations.insert(at);
            }
        }
    }
}
