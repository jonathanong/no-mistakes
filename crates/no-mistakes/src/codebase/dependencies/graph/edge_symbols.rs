fn collect_symbol_edges(
    files: &[PathBuf],
    facts: &TsFactMap,
    resolver: &ImportResolver<'_>,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    for path in files {
        let Some(file_facts) = facts.get(path) else {
            continue;
        };
        let Some(symbols) = file_facts.symbols.as_ref() else {
            continue;
        };

        let mut exported_values = HashSet::new();
        let mut caller_to_export = HashMap::new();

        for export in &symbols.exports {
            if export.name == "*" {
                continue;
            }
            let export_symbol = export_symbol_name(export);
            exported_values.insert(export.name.clone());
            caller_to_export.insert(export.name.clone(), export_symbol.clone());
            edges.push((
                NodeId::File(path.clone()),
                NodeId::Symbol {
                    file: path.clone(),
                    symbol: export_symbol.clone(),
                },
                symbol_edge_kind(export.is_type_only),
            ));

            if let ExportKind::ReExport { source, imported } = &export.kind {
                if imported != "*" {
                    if let Some(target) = resolver.resolve(source, path) {
                        edges.push((
                            NodeId::Symbol {
                                file: path.clone(),
                                symbol: export_symbol.clone(),
                            },
                            NodeId::Symbol {
                                file: target,
                                symbol: imported.clone(),
                            },
                            symbol_edge_kind(export.is_type_only),
                        ));
                    }
                }
            }
        }

        let imported_symbols = imported_symbol_map(path, symbols, resolver);
        for call in &file_facts.function_calls {
            let Some(caller) = &call.caller else {
                continue;
            };
            if !exported_values.contains(caller) {
                continue;
            }
            if let Some((target, imported, is_type_only)) = imported_symbols.get(&call.callee) {
                let caller_export = caller_to_export
                    .get(caller)
                    .expect("exported caller should have a symbol name")
                    .clone();
                edges.push((
                    NodeId::Symbol {
                        file: path.clone(),
                        symbol: caller_export,
                    },
                    NodeId::Symbol {
                        file: target.clone(),
                        symbol: imported.clone(),
                    },
                    symbol_edge_kind(*is_type_only),
                ));
            }
        }
    }
    edges
}
