fn collect_call_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    callable_export_resolutions: &mut FxHashMap<
        (std::path::PathBuf, String),
        ExportedCallableResolution,
    >,
) -> (Vec<Edge>, Vec<ResolvedCallSite>) {
    use rayon::prelude::*;
    let indexes = CallableResolutionIndexes::default();
    let output = edge_inputs
        .graph_files
        .indexable()
        .par_iter()
        .flat_map_iter(|path| {
            let Some(file) = facts.get_ts_facts(path) else {
                return Vec::new();
            };
            let Some(index) = indexes.file(facts, path) else { return Vec::new() };
            let call_offsets = file
                .function_calls
                .iter()
                .filter(|call| is_traversable_call(&index, call))
                .map(|call| (call.caller_id, call.offset, call.invocation))
                .collect::<FxHashSet<_>>();
            let mut sites = file
                .function_calls
                .iter()
                .filter(|call| is_traversable_call(&index, call))
                .map(|call| {
                    let resolved_callee = index
                        .resolve_alias(
                            call.caller.as_deref(),
                            call.callee_binding_scope,
                            &call.callee,
                            call.offset,
                            call.caller_id,
                            call.invocation,
                        )
                        .or_else(|| {
                            (call.target_identity
                                == crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction)
                                .then(|| {
                                    index.resolve_class_binding(
                                        call.callee_binding_scope,
                                        &call.callee,
                                        call.invocation,
                                    )
                                })
                                .flatten()
                        })
                        .unwrap_or_else(|| ResolvedLocalCallee {
                            callee: call.callee.clone(),
                            callable_id: None,
                        });
                    let callable_id = resolved_callee.callable_id.or_else(|| {
                        index.resolve_local_callable_id(
                            call.callee_binding_scope,
                            &resolved_callee.callee,
                        )
                    });
                    let target_identity = call_target_identity(&index, call, &resolved_callee.callee);
                    let target = match target_identity {
                        crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction => {
                            resolve_local_call_scope(
                                call.caller.as_deref(),
                                call.callee_binding_scope,
                                &resolved_callee.callee,
                                &index.known_scopes,
                                &index.class_scopes,
                            )
                                .map(|scope| (path.to_path_buf(), scope.to_string()))
                        }
                        crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport => None,
                        crate::codebase::dependencies::extract::CallTargetIdentity::Global
                        | crate::codebase::dependencies::extract::CallTargetIdentity::Unknown => None,
                    };
                    let source = call.caller.as_deref().map_or_else(
                        || NodeId::file_in(&edge_inputs.interner, path),
                        |caller| call.caller_id.map_or_else(
                            || NodeId::symbol_in(&edge_inputs.interner, path, caller),
                            |id| NodeId::callable_in(&edge_inputs.interner, path, caller, id),
                        ),
                    );
                    let resolved_target = match (target_identity, target) {
                        (crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction, Some((file, scope))) => {
                            ResolvedCallTarget::RepositoryFunction { file, scope }
                        }
                        (crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport, _) => {
                            resolve_imported_call_target(
                                edge_inputs,
                                facts,
                                resolver,
                                path,
                                &index,
                                &resolved_callee.callee,
                                &indexes,
                            )
                            .unwrap_or(ResolvedCallTarget::Unknown)
                        }
                        (crate::codebase::dependencies::extract::CallTargetIdentity::Global, _) => {
                            ResolvedCallTarget::Global { name: call.callee.clone() }
                        }
                        _ => ResolvedCallTarget::Unknown,
                    };
                    let edge = graph_call_target_node(
                        &edge_inputs.interner,
                        facts,
                        &indexes,
                        &resolved_target,
                        callable_id,
                    )
                    .map(|target| (source, target, EdgeKind::Call));
                    (
                        edge,
                        ResolvedCallSite {
                            file: path.to_path_buf(),
                            caller: call.caller.clone(),
                            caller_id: call.caller_id,
                            source_callee: call.callee.clone(),
                            line: call.line,
                            offset: call.offset,
                            invocation: call.invocation,
                            target: resolved_target,
                        },
                    )
                })
                .collect::<Vec<_>>();
            sites.extend(file.unknown_calls.iter().filter(|call| {
                !call_offsets.contains(&(call.caller_id, call.offset, call.invocation))
            }).map(|call| {
                (
                    None,
                    ResolvedCallSite {
                        file: path.to_path_buf(),
                        caller: call.caller.clone(),
                        caller_id: call.caller_id,
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
        });
    populate_callable_export_resolutions(
        edge_inputs,
        facts,
        resolver,
        &indexes,
        callable_export_resolutions,
    );
    output
}
