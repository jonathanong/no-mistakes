fn collect_symbol_import_selectors_file(
    inputs: ExportEdgeInputs<'_>,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    export_edges: &[Edge],
    exported: &FxHashSet<NodeId>,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let imported_symbols = imported_symbol_map(
        inputs.path,
        inputs.symbols,
        inputs.resolver,
        inputs.workspace,
        inputs.visible_files,
        inputs.graph_files,
        inputs.interner,
    );
    let namespaces = namespace_import_map(
        inputs.path,
        inputs.symbols,
        inputs.resolver,
        inputs.workspace,
        inputs.visible_files,
        inputs.graph_files,
        inputs.interner,
    );
    let mut relay_edges = export_edges.to_vec();
    collect_export_reference_edges(inputs, &imported_symbols, &namespaces, &mut relay_edges);
    for (from, to, _) in relay_edges {
        if !matches!(from, NodeId::Symbol { .. }) {
            continue;
        }
        if let NodeId::Symbol { file, symbol, .. } = &to {
            if !exported.contains(&to) {
                selector_unknown_symbol(&inputs, file.as_ref(), symbol, &mut edges);
            }
            edges.push((from, to, EdgeKind::SymbolReexport));
        } else if matches!(to, NodeId::File(_)) {
            edges.push((
                NodeId::file_in(inputs.interner, inputs.path),
                to,
                EdgeKind::SymbolImportUnknown,
            ));
        }
    }
    let mut alias_locals = fx_map();
    for export in &inputs.symbols.exports {
        if !matches!(export.kind, ExportKind::ReExport { .. }) {
            *alias_locals
                .entry(export_local_name(export))
                .or_insert(0usize) += 1;
        }
    }
    let mut referenced = fx_map();
    for reference in file_facts
        .function_calls
        .iter()
        .chain(&file_facts.symbol_references)
    {
        let root = reference
            .callee
            .split('.')
            .next()
            .unwrap_or(&reference.callee);
        *referenced.entry(root).or_insert(0usize) += 1;
    }
    for import in &inputs.symbols.imports {
        let Some(target) = selector_target(&inputs, &import.source) else {
            selector_unknown(&inputs, &import.source, &mut edges);
            continue;
        };
        // Assets cannot own any TS/JS production export in this catalog.
        if !is_indexable(&target) {
            continue;
        }
        if import.imported == "*" {
            selector_unknown(&inputs, &import.source, &mut edges);
        } else if alias_locals.get(&import.local).is_none_or(|aliases| {
            // Export specifiers themselves appear once in symbol references.
            referenced.get(import.local.as_str()).copied().unwrap_or(0) > *aliases
        }) {
            let symbol = NodeId::symbol_in(inputs.interner, &target, &import.imported);
            if !exported.contains(&symbol) {
                selector_unknown_symbol(&inputs, &target, &import.imported, &mut edges);
            }
            edges.push((
                NodeId::file_in(inputs.interner, inputs.path),
                symbol,
                EdgeKind::SymbolImport,
            ));
        }
    }
    for export in &inputs.symbols.exports {
        if let ExportKind::ReExport { source, .. } = &export.kind {
            if selector_target(&inputs, source).is_none() {
                selector_unknown(&inputs, source, &mut edges);
            }
        }
    }
    let named_sources: HashSet<_> = inputs
        .symbols
        .imports
        .iter()
        .map(|import| (import.source.as_str(), import.line))
        .collect();
    for import in &file_facts.imports {
        let unbound_type_reference = matches!(
            import.kind,
            crate::codebase::dependencies::extract::ImportKind::Type
        ) && !import.re_export
            && !named_sources.contains(&(import.specifier.as_str(), import.line));
        if import.computed
            || unbound_type_reference
            || matches!(
                import.kind,
                crate::codebase::dependencies::extract::ImportKind::Dynamic
                    | crate::codebase::dependencies::extract::ImportKind::Require
            )
        {
            selector_unknown(&inputs, &import.specifier, &mut edges);
        }
    }
    edges
}
