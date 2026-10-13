use super::*;

pub(in super::super) fn refine_value_calls(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
) {
    let base = pass.sites.clone();
    let mut incoming = incoming_inputs(&base);
    for _ in 0..=base.len() {
        let (sites, edges) = refine_iteration(
            inputs,
            facts,
            resolver,
            indexes,
            &base,
            incoming.clone(),
            true,
        );
        let next = incoming_inputs(&sites);
        if next == incoming {
            pass.sites = sites;
            pass.edges.extend(edges);
            return;
        }
        incoming = next;
    }
    // Recursive argument/callee cycles cannot prove a parameter binding.
    let (sites, edges) = refine_iteration(inputs, facts, resolver, indexes, &base, incoming, false);
    pass.sites = sites;
    pass.edges.extend(edges);
}
fn incoming_inputs(
    sites: &[ResolvedCallSite],
) -> FxHashMap<(std::path::PathBuf, CallableId), Vec<(std::path::PathBuf, u32)>> {
    let mut incoming: FxHashMap<_, Vec<_>> = fx_map();
    for site in sites {
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
    incoming
}
fn refine_iteration(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    base: &[ResolvedCallSite],
    incoming: FxHashMap<(std::path::PathBuf, CallableId), Vec<(std::path::PathBuf, u32)>>,
    parameters_enabled: bool,
) -> (Vec<ResolvedCallSite>, Vec<Edge>) {
    let mut flow = ValueFlowResolution {
        inputs,
        facts,
        resolver,
        indexes,
        incoming,
        parameters_enabled,
        parameters: fx_map(),
        visiting: fx_set(),
        calls_visiting: fx_set(),
    };
    let mut sites = base.to_vec();
    let mut edges = Vec::new();
    for site in &mut sites {
        if site.target_node.is_some() {
            continue;
        }
        let Some(index) = indexes.file(facts, &site.file) else {
            continue;
        };
        let Some(call) = index.values.calls.get(&site.offset) else {
            continue;
        };
        if matches!(site.target, ResolvedCallTarget::Unknown) {
            if let ValueExpression::Named {
                name,
                scope: None | Some(0),
                ..
            } = &call.callee
            {
                let head = name.split_once('.').map_or(name.as_str(), |(head, _)| head);
                if index.imported.get(head).is_some_and(|binding| binding.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace) { continue; }
            }
        }
        let value = flow.value(&site.file, &call.callee);
        let value = match value {
            FlowValue::Returned { factory, executor } => {
                site.returned_from = factory;
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
            edges.push((source, node.clone(), EdgeKind::Call));
            site.target_node = Some(node);
        }
        site.target = target;
    }
    (sites, edges)
}
