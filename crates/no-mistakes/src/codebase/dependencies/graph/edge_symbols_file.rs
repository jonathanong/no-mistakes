fn collect_symbol_edges_for_file(input: SymbolFileEdgeInputs<'_>) -> Vec<Edge> {
    let SymbolFileEdgeInputs {
        root,
        path,
        facts,
        resolver,
        workspace,
        visible_files,
        graph_files,
        http_route_defs,
        interner,
    } = input;
    let mut edges = Vec::new();
    let Some(file_facts) = facts.get_ts_facts(path) else {
        return edges;
    };
    let Some(symbols) = file_facts.symbols.as_ref() else {
        return edges;
    };

    let mut exported_values = Vec::new();
    let mut caller_to_export = HashMap::new();
    let exports = ExportEdgeInputs {
        path,
        symbols,
        facts,
        resolver,
        workspace,
        visible_files,
        graph_files,
        interner,
    };
    collect_export_edges(
        exports,
        &mut exported_values,
        &mut caller_to_export,
        &mut edges,
    );

    let imported_symbols = imported_symbol_map(
        path,
        symbols,
        resolver,
        workspace,
        visible_files,
        graph_files,
        interner,
    );
    let namespace_imports = namespace_import_map(
        path,
        symbols,
        resolver,
        workspace,
        visible_files,
        graph_files,
        interner,
    );
    for imported in fallback_imported_symbols(
        symbols.exports.is_empty(),
        &file_facts.function_calls,
        &file_facts.symbol_references,
        &imported_symbols,
        interner,
    ) {
        let (node, kind) = target_node(imported, interner);
        edges.push((NodeId::file_in(interner, path), node, kind));
    }
    for (node, kind) in fallback_namespace_symbols(
        &file_facts.function_calls,
        &file_facts.symbol_references,
        &namespace_imports,
        interner,
    ) {
        edges.push((NodeId::file_in(interner, path), node, kind));
    }
    collect_export_reference_edges(exports, &imported_symbols, &namespace_imports, &mut edges);

    // Function-call facts now retain unresolved/shadowed occurrences for call
    // policy reporting. Legacy symbol reachability remains a resolved-local
    // projection, so those occurrences cannot attribute imports by spelling.
    let resolved_calls = file_facts
        .function_calls
        .iter()
        .filter(|call| {
            call.target_identity
                == crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction
        })
        .cloned()
        .collect::<Vec<_>>();
    let calls_by_caller = local_call_graph(&resolved_calls);
    let call_records_by_caller = local_call_records(&resolved_calls);
    let refs_by_caller = local_call_graph(&file_facts.symbol_references);
    let ordered_refs_by_caller = local_ordered_call_graph(&file_facts.symbol_references);
    let scoped_imports = scoped_import_map_with_graph_files(
        &file_facts.imports,
        path,
        resolver,
        workspace,
        graph_files,
        interner,
    );
    let local_scopes = local_scope_names(&calls_by_caller, &refs_by_caller, &scoped_imports);
    exported_values.sort();
    exported_values.dedup();
    let value_exports = value_export_symbol_names(symbols);
    collect_top_level_imported_edges(
        path,
        &caller_to_export,
        &file_facts.function_calls,
        &imported_symbols,
        &value_exports,
        &mut edges,
        interner,
    );
    collect_exported_value_edges(
        ExportedValueEdgeInputs {
            root,
            path,
            file_facts,
            symbols,
            facts,
            resolver,
            workspace,
            visible_files,
            graph_files,
            http_route_defs,
            interner,
            exported_values,
            caller_to_export: &caller_to_export,
            imported_symbols: &imported_symbols,
            namespace_imports: &namespace_imports,
            calls_by_caller: &calls_by_caller,
            call_records_by_caller: &call_records_by_caller,
            refs_by_caller: &refs_by_caller,
            ordered_refs_by_caller: &ordered_refs_by_caller,
            scoped_imports: &scoped_imports,
            local_scopes: &local_scopes,
            value_exports: &value_exports,
        },
        &mut edges,
    );
    edges
}
