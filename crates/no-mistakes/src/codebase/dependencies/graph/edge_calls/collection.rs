fn collect_call_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
) -> (Vec<Edge>, Vec<ResolvedCallSite>) {
    use rayon::prelude::*;
    let indexes = CallableResolutionIndexes::default();
    edge_inputs
        .graph_files
        .indexable()
        .par_iter()
        .flat_map_iter(|path| {
            let Some(file) = facts.get_ts_facts(path) else {
                return Vec::new();
            };
            let Some(index) = indexes.file(facts, path) else { return Vec::new() };
            let mut sites = file
                .function_calls
                .iter()
                .map(|call| {
                    let resolved_callee = index
                        .resolve_alias(call.caller.as_deref(), &call.callee)
                        .unwrap_or_else(|| call.callee.clone());
                    let target_identity = call_target_identity(&index, call, &resolved_callee);
                    let target = match target_identity {
                        crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction => {
                            resolve_local_call_scope(call.caller.as_deref(), &resolved_callee, &index.known_scopes)
                                .map(|scope| (path.to_path_buf(), scope.to_string()))
                        }
                        crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport => {
                            resolve_imported_call_scope(edge_inputs, facts, resolver, path, &index, &resolved_callee, &indexes)
                        }
                        crate::codebase::dependencies::extract::CallTargetIdentity::Global
                        | crate::codebase::dependencies::extract::CallTargetIdentity::Unknown => None,
                    };
                    let source = call.caller.as_deref().map_or_else(
                        || NodeId::file_in(&edge_inputs.interner, path),
                        |caller| NodeId::symbol_in(&edge_inputs.interner, path, caller),
                    );
                    let resolved_target = match (target_identity, target) {
                        (crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction, Some((file, scope))) => {
                            ResolvedCallTarget::RepositoryFunction { file, scope }
                        }
                        (crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport, repository_target) => {
                            if repository_target.is_none()
                                && imported_call_targets_visible_module(
                                    edge_inputs,
                                    resolver,
                                    path,
                                    &index,
                                    &resolved_callee,
                                )
                            {
                                ResolvedCallTarget::Unknown
                            } else {
                                module_export_target(&index, &resolved_callee, repository_target)
                                    .unwrap_or(ResolvedCallTarget::Unknown)
                            }
                        }
                        (crate::codebase::dependencies::extract::CallTargetIdentity::Global, _) => {
                            ResolvedCallTarget::Global { name: call.callee.clone() }
                        }
                        _ => ResolvedCallTarget::Unknown,
                    };
                    let edge = match &resolved_target {
                        ResolvedCallTarget::RepositoryFunction { file, scope }
                        | ResolvedCallTarget::ModuleExport { repository_target: Some((file, scope)), .. } => Some((
                            source,
                            NodeId::symbol_in(&edge_inputs.interner, file, scope),
                            EdgeKind::Call,
                        )),
                        _ => None,
                    };
                    (
                        edge,
                        ResolvedCallSite {
                            file: path.to_path_buf(),
                            caller: call.caller.clone(),
                            source_callee: call.callee.clone(),
                            line: call.line,
                            offset: call.offset,
                            invocation: call.invocation,
                            target: resolved_target,
                        },
                    )
                })
                .collect::<Vec<_>>();
            sites.extend(file.unknown_calls.iter().map(|call| {
                (
                    None,
                    ResolvedCallSite {
                        file: path.to_path_buf(),
                        caller: call.caller.clone(),
                        source_callee: "<unknown>".to_string(),
                        line: call.line,
                        offset: call.offset,
                        invocation: call.invocation,
                        target: ResolvedCallTarget::Unknown,
                    },
                )
            }));
            sites
        })
        .fold(|| (Vec::new(), Vec::new()), |mut output, (edge, site)| {
            if let Some(edge) = edge {
                output.0.push(edge);
            }
            output.1.push(site);
            output
        })
        .reduce(|| (Vec::new(), Vec::new()), |mut left, mut right| {
            left.0.append(&mut right.0);
            left.1.append(&mut right.1);
            left
        })
}
