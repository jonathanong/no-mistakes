fn collect_symbol_edges(
    root: &Path,
    files: &[PathBuf],
    all_files: &[PathBuf],
    facts: &dyn TsFactLookup,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let http_route_defs = collect_symbol_http_route_defs(root, all_files, facts, config_options);
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
        let call_records_by_caller = local_call_records(&file_facts.function_calls);
        let refs_by_caller = local_call_graph(&file_facts.symbol_references);
        let ordered_refs_by_caller = local_ordered_call_graph(&file_facts.symbol_references);
        let scoped_imports = scoped_import_map(&file_facts.imports, path, resolver, workspace);
        let local_scopes = local_scope_names(&calls_by_caller, &refs_by_caller, &scoped_imports);
        exported_values.sort();
        exported_values.dedup();
        let scoped_http_route_defs = if file_facts.http_calls.is_empty() {
            &[][..]
        } else {
            http_route_defs.as_slice()
        };
        for exported_value in exported_values {
            let caller_exports = caller_to_export
                .get(&exported_value)
                .expect("exported value should have a symbol name")
                .clone();
            if let Some(imports) = scoped_imports.get("") {
                for export_symbol in &caller_exports {
                    for (target, kind) in imports {
                        edges.push((
                            NodeId::Symbol {
                                file: path.clone(),
                                symbol: export_symbol.clone(),
                            },
                            target.clone(),
                            *kind,
                        ));
                    }
                }
            }
            let mut visited = HashSet::new();
            let root_scope = exported_value.clone();
            let root_is_callable =
                exported_local_is_callable(symbols, &file_facts.exported_functions, &root_scope);
            let mut queue = VecDeque::from([exported_value]);
            while let Some(caller) = queue.pop_front() {
                if visited.insert(caller.clone()) {
                    collect_symbol_runtime_owner_file_edges(
                        SymbolRuntimeEdgeInputs {
                            root,
                            path,
                            caller_exports: &caller_exports,
                            caller: &caller,
                            calls_by_caller: &call_records_by_caller,
                            http_route_defs: scoped_http_route_defs,
                            process_spawns: &file_facts.process_spawns,
                        },
                        &mut edges,
                    );
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
                    let symbol_refs = refs_by_caller.get(&caller);
                    let ordered_symbol_refs = ordered_refs_by_caller.get(&caller);
                    for symbol_ref in symbol_refs.into_iter().flatten() {
                        if namespace_import_member_reference_exists(
                            symbol_ref,
                            ordered_symbol_refs,
                            &namespace_imports,
                        ) {
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
                        } else {
                            if let Some(scope) =
                                resolve_local_scope(&caller, symbol_ref, &local_scopes)
                            {
                                let scope_is_callable = calls_by_caller.contains_key(&scope);
                                if !root_is_callable || !scope_is_callable {
                                    queue.push_back(scope);
                                }
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

fn namespace_import_member_reference_exists(
    symbol_ref: &str,
    symbol_refs: Option<&Vec<String>>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
) -> bool {
    namespace_imports.contains_key(symbol_ref)
        && symbol_refs.is_some_and(|refs| {
            let prefix = format!("{symbol_ref}.");
            let bare_index = refs.iter().position(|candidate| candidate == symbol_ref);
            let member_index = refs
                .iter()
                .position(|candidate| candidate.starts_with(&prefix));
            match (bare_index, member_index) {
                (Some(bare_index), Some(member_index)) => member_index < bare_index,
                _ => false,
            }
        })
}
