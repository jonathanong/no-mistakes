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
        for target in site.target_nodes() {
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
    FxHashSet<(std::path::PathBuf, CallableId)>,
) {
    let mut flow = ValueFlowResolution {
        inputs,
        facts,
        resolver,
        indexes,
        incoming: data.incoming,
        supers: data.supers,
        replacements: data.replacements,
        replacement_read: false,
        used_parameters: fx_set(),
        parameters_enabled,
        initializing: Vec::new(),
        cache: data.cache,
        visiting: fx_set(),
        calls_visiting: fx_set(),
        default_origins: None,
        acquisition_context: None,
        invocation_read: None,
    };
    let mut sites = base.to_vec();
    for site in &mut sites {
        if site.invocation == InvocationKind::Construct {
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
        flow.replacement_read = false;
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
            site.target_node = None;
            continue;
        }
        if flow.replacement_read
            || matches!(
                value,
                FlowValue::Target(ResolvedCallTarget::Alternatives(_))
            )
        {
            // A recognized repeat read owns the answer, including Unknown.
            // Keeping the initial known target here would retain a stale edge.
            let target = match value {
                FlowValue::Target(target) => target,
                _ => ResolvedCallTarget::Unknown,
            };
            site.target = target;
            site.target_node = None;
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
            site.target_node = Some(node);
        }
        site.target = target;
    }
    (sites, flow.used_parameters)
}
