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
                .filter(|call| {
                    call.invocation
                        != crate::codebase::dependencies::extract::InvocationKind::Membership
                        && !(call.is_callback
                            && call.invocation
                                == crate::codebase::dependencies::extract::InvocationKind::Construct)
                })
                .map(|call| (call.caller_id, call.offset, call.invocation))
                .collect::<std::collections::HashSet<_>>();
            let mut sites = file
                .function_calls
                .iter()
                .filter(|call| {
                    call.invocation
                        != crate::codebase::dependencies::extract::InvocationKind::Membership
                        && !(call.is_callback
                            && call.invocation
                                == crate::codebase::dependencies::extract::InvocationKind::Construct)
                })
                .map(|call| {
                    let resolved_callee = index
                        .resolve_alias(
                            call.caller.as_deref(),
                            call.callee_binding_scope,
                            &call.callee,
                        )
                        .or_else(|| {
                            (call.target_identity
                                == crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction)
                                .then(|| {
                                    index.resolve_class_binding(
                                        call.callee_binding_scope,
                                        &call.callee,
                                    )
                                })
                                .flatten()
                        })
                        .unwrap_or_else(|| ResolvedLocalCallee {
                            callee: call.callee.clone(),
                            callable_id: None,
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
                    let edge = match &resolved_target {
                        ResolvedCallTarget::RepositoryFunction { file, scope }
                        | ResolvedCallTarget::ModuleExport { repository_target: Some((file, scope)), .. } => Some((
                            source,
                            callable_node_for_call(
                                &edge_inputs.interner,
                                facts,
                                file,
                                scope,
                                resolved_callee.callable_id,
                            ),
                            EdgeKind::Call,
                        )),
                        _ => None,
                    };
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

fn callable_node_for_call(
    interner: &crate::codebase::analysis_session::PathInterner,
    facts: &dyn TsFactLookup,
    file: &std::path::Path,
    scope: &str,
    exact_id: Option<crate::codebase::dependencies::extract::CallableId>,
) -> NodeId {
    let id = exact_id.or_else(|| {
        facts.get_ts_facts(file).and_then(|file_facts| {
            // The resolved file and canonical target scope own this callable
            // identity. Importer lexical scopes and local aliases are unrelated
            // source files, and using either can select a same-spelled target
            // declaration instead of the export resolution's actual callable.
            let mut ids = file_facts
                .callable_scope_ids
                .iter()
                .filter_map(|(id, display)| (display == scope).then_some(*id));
            let first = ids.next()?;
            ids.next().is_none().then_some(first)
        })
    });
    id.map_or_else(
        || NodeId::symbol_in(interner, file, scope),
        |id| NodeId::callable_in(interner, file, scope, id),
    )
}
