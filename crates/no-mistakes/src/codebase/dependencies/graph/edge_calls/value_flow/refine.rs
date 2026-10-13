use super::*;


pub(in super::super) fn refine_value_calls(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
) {
    let mut incoming: FxHashMap<_, Vec<_>> = fx_map();
    for site in &pass.sites {
        if let Some(NodeId::Symbol {
            file,
            callable_id: Some(id),
            ..
        }) = &site.target_node
        {
            incoming
                .entry((file.to_path_buf(), *id))
                .or_default()
                .push((site.file.clone(), site.offset));
        }
    }
    let mut flow = ValueFlowResolution {
        inputs,
        facts,
        resolver,
        indexes,
        incoming,
        parameters: fx_map(),
        visiting: fx_set(),
        calls_visiting: fx_set(),
    };
    for site in &mut pass.sites {
        if site.target_node.is_some() {
            continue;
        }
        let Some(index) = indexes.file(facts, &site.file) else {
            continue;
        };
        let Some(call) = index.values.calls.get(&site.offset) else {
            continue;
        };
        let value = flow.value(&site.file, &call.callee);
        let value = match value {
            FlowValue::Returned { factory, executor } => {
                site.returned_from = Some(factory);
                *executor
            }
            value => value,
        };
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
            _ => continue,
        };
        if let Some(node) =
            graph_call_target_node(&inputs.interner, facts, indexes, &target, exact_id)
        {
            let source = site.caller.as_deref().map_or_else(
                || NodeId::file_in(&inputs.interner, &site.file),
                |caller| NodeId::scoped_in(&inputs.interner, &site.file, caller, site.caller_id),
            );
            pass.edges.push((source, node.clone(), EdgeKind::Call));
            site.target_node = Some(node);
        }
        site.target = target;
    }
}
