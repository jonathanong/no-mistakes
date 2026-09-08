#[allow(clippy::too_many_arguments)]
fn collect_exported_value_edges(
    root: &std::path::Path,
    path: &std::path::Path,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    graph_files: &GraphFiles,
    http_route_defs: &[(PathBuf, String)],
    interner: &PathInterner,
    exported_values: Vec<String>,
    caller_to_export: &HashMap<String, Vec<String>>,
    imported_symbols: &HashMap<String, ImportedSymbolTarget>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
    calls_by_caller: &HashMap<String, Vec<String>>,
    call_records_by_caller: &HashMap<String, Vec<FunctionCall>>,
    refs_by_caller: &HashMap<String, Vec<String>>,
    ordered_refs_by_caller: &HashMap<String, Vec<String>>,
    scoped_imports: &HashMap<String, Vec<(NodeId, EdgeKind)>>,
    local_scopes: &HashSet<String>,
    value_exports: &HashSet<String>,
    edges: &mut Vec<Edge>,
) {
    let scoped_http_route_defs = if file_facts.http_calls.is_empty() {
        &[][..]
    } else {
        http_route_defs
    };
    for exported_value in exported_values {
        let caller_exports = caller_to_export
            .get(&exported_value)
            .expect("exported value should have a symbol name")
            .clone();
        if let Some(imports) = scoped_imports.get("") {
            collect_file_scope_import_edges(
                path,
                &caller_exports,
                value_exports,
                imports,
                edges,
                interner,
            );
        }
        let mut visited = HashSet::new();
        let root_scope = exported_value.clone();
        let root_is_callable =
            exported_local_is_callable(symbols, &file_facts.exported_functions, &root_scope);
        let mut queue = VecDeque::from([exported_value]);
        while let Some(caller) = queue.pop_front() {
            if !visited.insert(caller.clone()) {
                continue;
            }
            collect_symbol_runtime_owner_file_edges(
                SymbolRuntimeEdgeInputs {
                    root,
                    path,
                    caller_exports: &caller_exports,
                    caller: &caller,
                    calls_by_caller: call_records_by_caller,
                    http_route_defs: scoped_http_route_defs,
                    process_spawns: &file_facts.process_spawns,
                    visible_files,
                    interner,
                },
                edges,
            );
            if let Some(imports) = scoped_imports.get(&caller) {
                for (target, kind) in imports {
                    for caller_export in &caller_exports {
                        edges.push((
                            NodeId::symbol_in(interner, path, caller_export.clone()),
                            target.clone(),
                            *kind,
                        ));
                    }
                }
            }
            let symbol_refs = refs_by_caller.get(&caller);
            let ordered_symbol_refs = ordered_refs_by_caller.get(&caller);
            for symbol_ref in symbol_refs.into_iter().flatten() {
                if namespace_import_member_reference_exists(
                    symbol_ref,
                    ordered_symbol_refs,
                    namespace_imports,
                ) {
                    continue;
                }
                if let Some((target, kind)) = resolve_imported_callee_with_graph_files(
                    symbol_ref,
                    imported_symbols,
                    namespace_imports,
                    ReexportResolutionInputs {
                        facts,
                        resolver,
                        workspace,
                        visible_files,
                        graph_files,
                        interner,
                    },
                ) {
                    for caller_export in &caller_exports {
                        edges.push((
                            NodeId::symbol_in(interner, path, caller_export.clone()),
                            target.clone(),
                            kind,
                        ));
                    }
                } else if let Some(scope) = resolve_local_scope(&caller, symbol_ref, local_scopes) {
                    let scope_is_callable = calls_by_caller.contains_key(&scope);
                    if !root_is_callable || !scope_is_callable {
                        queue.push_back(scope);
                    }
                }
            }
            for callee in calls_by_caller.get(&caller).into_iter().flatten() {
                if let Some(scope) = resolve_local_scope(&caller, callee, local_scopes) {
                    queue.push_back(scope);
                }
            }
        }
    }
}
