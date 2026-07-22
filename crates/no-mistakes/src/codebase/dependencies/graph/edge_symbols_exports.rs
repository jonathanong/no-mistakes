struct ExportEdgeInputs<'a> {
    path: &'a Path,
    symbols: &'a crate::codebase::ts_symbols::FileSymbols,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &'a HashSet<PathBuf>,
    graph_files: &'a GraphFiles,
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
        let Some(target) = inputs.graph_files.visible_path(&target) else {
            return;
        };
        if !is_indexable(target) {
            edges.push((from, NodeId::File(target.to_path_buf()), EdgeKind::AssetImport));
            return;
        }
        if imported == "*" {
            edges.push((
                from,
                NodeId::File(target.to_path_buf()),
                symbol_edge_kind(export.is_type_only),
            ));
            return;
        }
        let kind = symbol_edge_kind(
            export.is_type_only || target_export_is_type(target, imported, inputs.facts),
        );
        edges.push((
            from,
            NodeId::Symbol {
                file: target.to_path_buf(),
                symbol: imported.clone(),
            },
            kind,
        ));
    } else if let Some(target) = inputs.workspace.resolve_specifier_from_file_visible(
        source,
        inputs.path,
        inputs.visible_files,
    ) {
        let Some(target) = inputs.graph_files.visible_path(&target) else { return; };
        if imported == "*" {
            edges.push((from, NodeId::File(target.to_path_buf()), EdgeKind::WorkspaceImport));
            return;
        }
        edges.push((
            from,
            NodeId::Symbol {
                file: target.to_path_buf(),
                symbol: imported.clone(),
            },
            EdgeKind::WorkspaceImport,
        ));
    } else if !inputs
        .workspace
        .recognizes_specifier_from(source, inputs.path)
    {
        if let Some(node) = bare_module_node(source) {
            edges.push((from, node, symbol_edge_kind(export.is_type_only)));
        }
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
                resolve_imported_callee_with_graph_files(
                    &local_symbol,
                    imported_symbols,
                    namespace_imports,
                    ReexportResolutionInputs {
                        facts: inputs.facts,
                        resolver: inputs.resolver,
                        workspace: inputs.workspace,
                        visible_files: inputs.visible_files,
                        graph_files: inputs.graph_files,
                    },
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
