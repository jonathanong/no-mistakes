use super::*;

#[path = "worklist.rs"]
mod worklist;
pub(in super::super) use worklist::refine_value_calls;
pub(super) fn incoming_inputs(
    sites: &[ResolvedCallSite],
    constructors: &ConstructorRoutes,
    indexes: &CallableResolutionIndexes,
    facts: &dyn TsFactLookup,
) -> ParameterInputs {
    let mut incoming: ParameterInputs = fx_map();
    for site in sites {
        if site.invocation == InvocationKind::Membership {
            continue;
        }
        let Some(target) = &site.target_node else {
            continue;
        };
        let target = if site.invocation == InvocationKind::Construct {
            constructors::route(constructors, target).unwrap_or(target)
        } else {
            target
        };
        if let NodeId::Symbol {
            file,
            callable_id: Some(id),
            ..
        } = target
        {
            incoming
                .entry((file.to_path_buf(), *id))
                .or_default()
                .push(ParameterInput::from_site(site, indexes, facts));
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
    data: ValueInputs<'_>,
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
        incoming: data.incoming,
        supers: data.supers,
        used_parameters: fx_set(),
        parameters_enabled,
        initializing: Vec::new(),
        cache: data.cache,
        visiting: fx_set(),
        calls_visiting: fx_set(),
        default_origins: None,
        acquisition_context: None,
        invocation_read: None,
        invocation_start: None,
        executing_read: None,
        module_writer_phases: data.module_writer_phases,
        writer_dependencies: Default::default(),
        module_snapshot: None,
        repeated_initializer: None,
        member_read: None,
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
            let mut callee = &call.callee;
            while let ValueExpression::Member(base, _) = callee {
                callee = base;
            }
            if let ValueExpression::Named {
                name,
                scope: None | Some(0),
                ..
            } = callee
            {
                let head = name.split_once('.').map_or(name.as_str(), |(head, _)| head);
                if index.imported.get(head).is_some_and(|binding| binding.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace) { continue; }
            }
        }
        if call.bound {
            continue;
        }
        let value = flow.value(&site.file, &call.callee);
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
                FlowValue::Bound { target, .. } => *target,
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
        ) {
            site.typed_from.clear();
            site.returned_member_from.clear();
        }
        if matches!(value, FlowValue::Promise(_) | FlowValue::Uncallable) {
            site.target = ResolvedCallTarget::Unknown;
            continue;
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
    flow.merge_writer_dependencies();
    (sites, edges, flow.used_parameters)
}
