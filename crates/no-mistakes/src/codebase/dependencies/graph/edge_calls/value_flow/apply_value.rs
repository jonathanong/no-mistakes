use super::*;

pub(super) fn apply_value(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    indexes: &CallableResolutionIndexes,
    site: &mut ResolvedCallSite,
    value: FlowValue,
    replacement_read: bool,
) {
    let mut value = value;
    loop {
        value = match value {
            FlowValue::Provenance {
                executor,
                typed,
                members,
                ..
            } => {
                site.typed_from.extend(typed);
                site.returned_member_from.extend(members);
                *executor
            }
            FlowValue::Returned {
                factory, executor, ..
            } => {
                site.returned_from.extend(factory);
                *executor
            }
            FlowValue::Bound { target, .. }
            | FlowValue::TypedObject {
                executor: target, ..
            } => *target,
            FlowValue::Choice(values) => {
                site.target = ResolvedCallTarget::Unknown;
                site.target_node = None;
                site.alternatives = values
                    .into_iter()
                    .map(|value| {
                        let mut alternative = site.clone();
                        alternative.alternatives.clear();
                        apply_value(
                            inputs,
                            facts,
                            indexes,
                            &mut alternative,
                            value,
                            replacement_read,
                        );
                        alternative
                    })
                    .collect();
                return;
            }
            other => {
                value = other;
                break;
            }
        };
    }
    if matches!(
        value,
        FlowValue::Promise(_)
            | FlowValue::Uncallable
            | FlowValue::Object { .. }
            | FlowValue::Array { .. }
            | FlowValue::Undefined
            | FlowValue::Null
    ) {
        site.typed_from.clear();
        site.returned_member_from.clear();
    }
    if matches!(value, FlowValue::Promise(_) | FlowValue::Uncallable) {
        site.target = ResolvedCallTarget::Unknown;
        site.target_node = None;
        return;
    }
    if replacement_read
        || matches!(
            value,
            FlowValue::Target(ResolvedCallTarget::Alternatives(_))
        )
    {
        // A recognized repeat read owns the answer, including Unknown.
        site.target = match value {
            FlowValue::Target(target) => target,
            _ => ResolvedCallTarget::Unknown,
        };
        site.target_node = None;
        return;
    }
    let (target, exact_id) = match value {
        FlowValue::Callable { file, scope, id } => {
            let target = match &site.target {
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    ..
                } => ResolvedCallTarget::ModuleExport {
                    specifier: specifier.clone(),
                    export_path: export_path.clone(),
                    repository_target: Some((file, scope)),
                    callable_id: Some(id),
                },
                _ => ResolvedCallTarget::RepositoryFunction { file, scope },
            };
            (target, Some(id))
        }
        FlowValue::Target(target)
            if matches!(site.target, ResolvedCallTarget::Unknown)
                || matches!(
                    target,
                    ResolvedCallTarget::ModuleExport {
                        repository_target: Some(_),
                        ..
                    }
                ) =>
        {
            (target, None)
        }
        _ => return,
    };
    if let Some(node) = graph_call_target_node(&inputs.interner, facts, indexes, &target, exact_id)
    {
        site.target_node = Some(node);
    }
    site.target = target;
}
