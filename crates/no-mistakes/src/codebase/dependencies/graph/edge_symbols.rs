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

        let mut exported_values = Vec::new();
        let mut caller_to_export = HashMap::new();

        for export in &symbols.exports {
            if export.name == "*" {
                if let ExportKind::ReExport { source, imported } = &export.kind {
                    if imported == "*" {
                        if let Some(target) = resolver.resolve(source, path) {
                            if let Some(target_symbols) =
                                facts.get(&target).and_then(|facts| facts.symbols.as_ref())
                            {
                                for target_export in &target_symbols.exports {
                                    if target_export.name == "*" {
                                        continue;
                                    }
                                    let reexported_symbol = export_symbol_name(target_export);
                                    if reexported_symbol == "default" {
                                        continue;
                                    }
                                    edges.push((
                                        NodeId::File(path.clone()),
                                        NodeId::Symbol {
                                            file: path.clone(),
                                            symbol: reexported_symbol.clone(),
                                        },
                                        symbol_edge_kind(export.is_type_only),
                                    ));
                                    edges.push((
                                        NodeId::Symbol {
                                            file: path.clone(),
                                            symbol: reexported_symbol.clone(),
                                        },
                                        NodeId::Symbol {
                                            file: target.clone(),
                                            symbol: reexported_symbol,
                                        },
                                        symbol_edge_kind(export.is_type_only),
                                    ));
                                }
                            }
                        }
                    }
                }
                continue;
            }
            let export_symbol = export_symbol_name(export);
            let local_symbol = export_local_name(export);
            exported_values.push(local_symbol.clone());
            caller_to_export.insert(local_symbol.clone(), export_symbol.clone());
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
        let namespace_imports = namespace_import_map(path, symbols, resolver);
        for export in &symbols.exports {
            if matches!(export.kind, ExportKind::ReExport { .. }) || export.name == "*" {
                continue;
            }
            let local_symbol = export_local_name(export);
            if let Some((target, imported, is_type_only)) =
                resolve_imported_callee(&local_symbol, &imported_symbols, &namespace_imports)
            {
                edges.push((
                    NodeId::Symbol {
                        file: path.clone(),
                        symbol: export_symbol_name(export),
                    },
                    NodeId::Symbol {
                        file: target,
                        symbol: imported,
                    },
                    symbol_edge_kind(is_type_only),
                ));
            }
        }

        let calls_by_caller = local_call_graph(&file_facts.function_calls);
        exported_values.sort();
        exported_values.dedup();
        for exported_value in exported_values {
            let caller_export = caller_to_export
                .get(&exported_value)
                .expect("exported value should have a symbol name")
                .clone();
            let mut visited = HashSet::new();
            let mut queue = VecDeque::from([exported_value]);
            while let Some(caller) = queue.pop_front() {
                if visited.insert(caller.clone()) {
                    let Some(callees) = calls_by_caller.get(&caller) else {
                        continue;
                    };
                    for callee in callees {
                        if let Some((target, imported, is_type_only)) = resolve_imported_callee(
                            callee,
                            &imported_symbols,
                            &namespace_imports,
                        )
                        {
                            edges.push((
                                NodeId::Symbol {
                                    file: path.clone(),
                                    symbol: caller_export.clone(),
                                },
                                NodeId::Symbol {
                                    file: target,
                                    symbol: imported,
                                },
                                symbol_edge_kind(is_type_only),
                            ));
                        } else if calls_by_caller.contains_key(callee) {
                            queue.push_back(callee.clone());
                        }
                    }
                }
            }
        }
    }
    edges
}
