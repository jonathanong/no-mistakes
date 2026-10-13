use super::*;

#[path = "worklist.rs"]
mod worklist;
pub(in super::super) use worklist::refine_value_calls;
fn incoming_inputs(sites: &[ResolvedCallSite]) -> ParameterInputs {
    let mut incoming: FxHashMap<_, Vec<_>> = fx_map();
    let constructors: FxHashMap<_, _> = sites
        .iter()
        .filter(|site| {
            site.invocation == InvocationKind::Membership && site.source_callee == "constructor"
        })
        .filter_map(|site| {
            let caller = site.caller_id?;
            let NodeId::Symbol {
                file,
                callable_id: Some(id),
                ..
            } = site.target_node.as_ref()?
            else {
                return None;
            };
            Some(((site.file.clone(), caller), (file.to_path_buf(), *id)))
        })
        .collect();
    for site in sites {
        if site.invocation == InvocationKind::Membership {
            continue;
        }
        if let Some(NodeId::Symbol {
            file,
            callable_id: Some(id),
            ..
        }) = &site.target_node
        {
            let key = if site.invocation == InvocationKind::Construct {
                constructors
                    .get(&(file.to_path_buf(), *id))
                    .cloned()
                    .unwrap_or((file.to_path_buf(), *id))
            } else {
                (file.to_path_buf(), *id)
            };
            incoming.entry(key).or_default().push((
                site.file.clone(),
                site.offset,
                site.source_callee.clone(),
            ));
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
    incoming: &ParameterInputs,
    parameters_enabled: bool,
) -> (
    Vec<ResolvedCallSite>,
    Vec<Edge>,
    FxHashSet<(std::path::PathBuf, CallableId)>,
) {
    let mut flow = ValueFlowResolution {
        inputs,
        facts,
        resolver,
        indexes,
        incoming,
        used_parameters: fx_set(),
        parameters_enabled,
        parameters: fx_map(),
        visiting: fx_set(),
        calls_visiting: fx_set(),
    };
    let mut sites = base.to_vec();
    let mut edges = Vec::new();
    for site in &mut sites {
        if site.target_node.is_some() || site.invocation == InvocationKind::Construct {
            continue;
        }
        let Some(index) = indexes.file(facts, &site.file) else {
            continue;
        };
        let call = index
            .values
            .sites
            .get(&site.offset)
            .and_then(|sites| sites.get(site.source_callee.as_str()))
            .and_then(|key| index.values.calls.get(key));
        let Some(call) = call else {
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
            FlowValue::Returned {
                factory, executor, ..
            } => {
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
    (sites, edges, flow.used_parameters)
}
