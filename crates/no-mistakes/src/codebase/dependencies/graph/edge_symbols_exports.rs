struct ExportEdgeInputs<'a> {
    path: &'a Path,
    symbols: &'a crate::codebase::ts_symbols::FileSymbols,
    facts: &'a dyn TsFactLookup,
    resolver: &'a ImportResolver<'a>,
    workspace: &'a crate::codebase::workspaces::WorkspaceMap,
}

fn collect_export_edges(
    inputs: ExportEdgeInputs<'_>,
    exported_values: &mut Vec<String>,
    caller_to_export: &mut HashMap<String, Vec<String>>,
    edges: &mut Vec<Edge>,
) {
    collect_star_reexport_edges(&inputs, edges);
    for export in &inputs.symbols.exports {
        if export.name == "*" {
            continue;
        }
        let export_symbol = export_symbol_name(export);
        let local_symbol = export_local_name(export);
        if !matches!(export.kind, ExportKind::ReExport { .. }) {
            exported_values.push(local_symbol.clone());
            caller_to_export
                .entry(local_symbol)
                .or_default()
                .push(export_symbol.clone());
        }
        edges.push((
            NodeId::File(inputs.path.to_path_buf()),
            NodeId::Symbol {
                file: inputs.path.to_path_buf(),
                symbol: export_symbol.clone(),
            },
            symbol_edge_kind(export.is_type_only),
        ));
        collect_direct_reexport_edge(&inputs, export, &export_symbol, edges);
    }
}

fn collect_direct_reexport_edge(
    inputs: &ExportEdgeInputs<'_>,
    export: &crate::codebase::ts_symbols::Export,
    export_symbol: &str,
    edges: &mut Vec<Edge>,
) {
    let ExportKind::ReExport { source, imported } = &export.kind else {
        return;
    };
    if imported == "*" && export_symbol == "*" {
        return;
    }
    let from = NodeId::Symbol {
        file: inputs.path.to_path_buf(),
        symbol: export_symbol.to_string(),
    };
    if let Some(target) = inputs.resolver.resolve(source, inputs.path) {
        if imported == "*" {
            edges.push((from, NodeId::File(target), symbol_edge_kind(export.is_type_only)));
            return;
        }
        if !is_indexable(&target) {
            edges.push((from, NodeId::File(target), EdgeKind::AssetImport));
            return;
        }
        let kind = symbol_edge_kind(
            export.is_type_only || target_export_is_type(&target, imported, inputs.facts),
        );
        edges.push((
            from,
            NodeId::Symbol {
                file: target,
                symbol: imported.clone(),
            },
            kind,
        ));
    } else if let Some(target) = inputs.workspace.resolve_specifier_from(source, inputs.path) {
        if imported == "*" {
            edges.push((from, NodeId::File(target), EdgeKind::WorkspaceImport));
            return;
        }
        edges.push((
            from,
            NodeId::Symbol {
                file: target,
                symbol: imported.clone(),
            },
            EdgeKind::WorkspaceImport,
        ));
    } else if let Some(node) = bare_module_node(source) {
        edges.push((from, node, symbol_edge_kind(export.is_type_only)));
    }
}

fn collect_export_reference_edges(
    inputs: ExportEdgeInputs<'_>,
    imported_symbols: &HashMap<String, ImportedSymbolTarget>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
    edges: &mut Vec<Edge>,
) {
    for export in &inputs.symbols.exports {
        if matches!(export.kind, ExportKind::ReExport { .. }) || export.name == "*" {
            continue;
        }
        let local_symbol = export_local_name(export);
        let resolved = namespace_imports
            .get(&local_symbol)
            .map(namespace_file_node)
            .or_else(|| {
                resolve_imported_callee(
            &local_symbol,
            imported_symbols,
            namespace_imports,
            inputs.facts,
            inputs.resolver,
            inputs.workspace,
                )
            });
        if let Some((target, kind)) = resolved {
            edges.push((
                NodeId::Symbol {
                    file: inputs.path.to_path_buf(),
                    symbol: export_symbol_name(export),
                },
                target,
                kind,
            ));
        }
    }
}
