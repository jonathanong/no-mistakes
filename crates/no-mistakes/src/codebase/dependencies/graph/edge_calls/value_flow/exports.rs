use super::*;

pub(in super::super) fn resolve_value_exports(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &CallPass,
    exports: &mut FxHashMap<(std::path::PathBuf, String), ExportedCallableResolution>,
) {
    let incoming = refine::incoming_inputs(&pass.sites, &pass.constructor_routes, indexes, facts);
    let resolved: Vec<_> = exports
        .par_iter()
        .filter_map(|(key, export)| {
            let ExportedCallableResolution::Value(file, local) = export else {
                return None;
            };
            let mut flow = ValueFlowResolution {
                inputs,
                facts,
                resolver,
                indexes,
                incoming: &incoming,
                supers: &pass.super_constructors,
                replacements: None,
                replacement_read: false,
                foreign_acquisition: false,
                used_parameters: fx_set(),
                parameters_enabled: true,
                initializing: Vec::new(),
                cache: &pass.parameter_cache,
                visiting: fx_set(),
                calls_visiting: fx_set(),
                default_origins: None,
                acquisition_context: None,
                invocation_read: None,
                executing_read: None,
                member_read: None,
            };
            let mut value = flow.named(file, local, Some(0), u32::MAX, None, None);
            loop {
                value = match value {
                    FlowValue::Returned { executor, .. } => *executor,
                    FlowValue::Bound { target, .. } => *target,
                    other => {
                        value = other;
                        break;
                    }
                };
            }
            let (target, exact) = match value {
                FlowValue::Callable { file, scope, id } => (
                    ResolvedCallTarget::RepositoryFunction { file, scope },
                    Some(id),
                ),
                FlowValue::Target(target) => (target, None),
                _ => return None,
            };
            let NodeId::Symbol {
                file,
                symbol,
                callable_id,
            } = graph_call_target_node(&inputs.interner, facts, indexes, &target, exact)?
            else {
                return None;
            };
            Some((
                key.clone(),
                ExportedCallableResolution::Callable(
                    file.to_path_buf(),
                    symbol.to_string(),
                    callable_id,
                ),
            ))
        })
        .collect();
    exports.extend(resolved);
}
