fn collect_symbol_edges(
    files: &[PathBuf],
    facts: &dyn TsFactLookup,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    for path in files {
        let Some(file_facts) = facts.get_ts_facts(path) else {
            continue;
        };
        let Some(symbols) = file_facts.symbols.as_ref() else { continue };

        let mut exported_values = Vec::new();
        let mut caller_to_export = HashMap::new();
        collect_export_edges(
            ExportEdgeInputs {
                path,
                symbols,
                facts,
                resolver,
                workspace,
            },
            &mut exported_values,
            &mut caller_to_export,
            &mut edges,
        );

        let imported_symbols = imported_symbol_map(path, symbols, resolver, workspace);
        let namespace_imports = namespace_import_map(path, symbols, resolver, workspace);
        for imported in fallback_imported_symbols(
            symbols.exports.is_empty(),
            &file_facts.function_calls,
            &file_facts.symbol_references,
            &imported_symbols,
        ) {
            let (node, kind) = target_node(imported);
            edges.push((NodeId::File(path.clone()), node, kind));
        }
        for (node, kind) in fallback_namespace_symbols(
            &file_facts.function_calls,
            &file_facts.symbol_references,
            &namespace_imports,
        ) {
            edges.push((NodeId::File(path.clone()), node, kind));
        }
        collect_export_reference_edges(
            ExportEdgeInputs {
                path,
                symbols,
                facts,
                resolver,
                workspace,
            },
            &imported_symbols,
            &namespace_imports,
            &mut edges,
        );

        let calls_by_caller = local_call_graph(&file_facts.function_calls);
        let refs_by_caller = local_call_graph(&file_facts.symbol_references);
        let scoped_imports = scoped_import_map(&file_facts.imports, path, resolver, workspace);
        let local_scopes = local_scope_names(&calls_by_caller, &refs_by_caller, &scoped_imports);
        exported_values.sort();
        exported_values.dedup();
        for exported_value in exported_values {
            let caller_exports = caller_to_export
                .get(&exported_value)
                .expect("exported value should have a symbol name")
                .clone();
            let mut visited = HashSet::new();
            let root_scope = exported_value.clone();
            let root_is_callable =
                exported_local_is_callable(symbols, &file_facts.exported_functions, &root_scope);
            let mut queue = VecDeque::from([exported_value]);
            while let Some(caller) = queue.pop_front() {
                if visited.insert(caller.clone()) {
                    if let Some(imports) = scoped_imports.get(&caller) {
                        for (target, kind) in imports {
                            for caller_export in &caller_exports {
                                edges.push((
                                    NodeId::Symbol {
                                        file: path.clone(),
                                        symbol: caller_export.clone(),
                                    },
                                    target.clone(),
                                    *kind,
                                ));
                            }
                        }
                    }
                    for symbol_ref in refs_by_caller.get(&caller).into_iter().flatten() {
                        let binding = symbol_ref
                            .split_once('.')
                            .map_or(symbol_ref.as_str(), |(binding, _)| binding);
                        if file_facts.local_type_declarations.contains(binding) {
                            continue;
                        }
                        if let Some((target, kind)) = resolve_imported_callee(
                            symbol_ref,
                            &imported_symbols,
                            &namespace_imports,
                            facts,
                            resolver,
                            workspace,
                        ) {
                            for caller_export in &caller_exports {
                                edges.push((
                                    NodeId::Symbol {
                                        file: path.clone(),
                                        symbol: caller_export.clone(),
                                    },
                                    target.clone(),
                                    kind,
                                ));
                            }
                        } else if caller == root_scope && !root_is_callable {
                            if let Some(scope) =
                                resolve_local_scope(&caller, symbol_ref, &local_scopes)
                            {
                                queue.push_back(scope);
                            }
                        }
                    }
                    for callee in calls_by_caller.get(&caller).into_iter().flatten() {
                        if let Some(scope) = resolve_local_scope(&caller, callee, &local_scopes) {
                            queue.push_back(scope);
                        }
                    }
                }
            }
        }
    }
    edges
}
