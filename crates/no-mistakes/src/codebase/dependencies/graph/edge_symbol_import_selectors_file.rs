fn collect_symbol_import_selectors_file(
    inputs: ExportEdgeInputs<'_>,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    export_edges: &[Edge],
    exported: &FxHashSet<NodeId>,
) -> (Vec<Edge>, SelectorImportSites) {
    let mut edges = Vec::new();
    let mut sites = fx_map();
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
    collect_selector_relay_edges(inputs, relay_edges, exported, &mut edges, &mut sites);
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
            selector_unknown(
                &inputs,
                &import.source,
                import.line as usize,
                "unresolved module selector",
                &mut edges,
                &mut sites,
            );
            continue;
        };
        // Assets cannot own any TS/JS production export in this catalog.
        if !is_indexable(&target) {
            continue;
        }
        if import.imported == "*" {
            // A type namespace used only as `ns.Member` is resolved to its members.
            if import.is_type_only
                && type_namespace_members(file_facts, &import.source, &import.local).is_some()
            {
                continue;
            }
            selector_unknown(
                &inputs,
                &import.source,
                import.line as usize,
                "namespace selector",
                &mut edges,
                &mut sites,
            );
        } else if alias_locals.get(&import.local).is_none_or(|aliases| {
            // Export specifiers themselves appear once in symbol references.
            referenced.get(import.local.as_str()).copied().unwrap_or(0) > *aliases
        }) {
            let symbol = NodeId::symbol_in(inputs.interner, &target, &import.imported);
            if !exported.contains(&symbol) {
                selector_unknown_symbol(
                    &inputs,
                    &target,
                    &import.imported,
                    SelectorImportSite {
                        line: import.line.max(1) as usize,
                        specifier: import.source.clone(),
                        reason: "unresolved named selector",
                    },
                    &mut edges,
                    &mut sites,
                );
            }
            edges.push((
                NodeId::file_in(inputs.interner, inputs.path),
                symbol,
                if import.is_type_only {
                    EdgeKind::SymbolTypeImport
                } else {
                    EdgeKind::SymbolImport
                },
            ));
        }
    }
    for export in &inputs.symbols.exports {
        if let ExportKind::ReExport { source, .. } = &export.kind {
            if selector_target(&inputs, source).is_none() {
                selector_unknown(
                    &inputs,
                    source,
                    export.line as usize,
                    "unresolved re-export selector",
                    &mut edges,
                    &mut sites,
                );
            }
        }
    }
    collect_type_import_selectors(&inputs, file_facts, exported, &mut edges, &mut sites);
    let mock_sites = typed_mock_import_sites(file_facts);
    let proven_sites = proven_dynamic_selectors(file_facts);
    for import in &file_facts.imports {
        if !import.computed
            && import.kind == ImportKind::Dynamic
            && mock_sites.contains(&(import.specifier.as_str(), import.line))
        {
            continue;
        }
        if !import.computed && import.kind == ImportKind::Dynamic {
            let proven = proven_sites.get(&(import.specifier.as_str(), import.line));
            if proven.is_some_and(|names| {
                push_dynamic_selectors(
                    &inputs,
                    &import.specifier,
                    import.line as usize,
                    names,
                    exported,
                    &mut edges,
                    &mut sites,
                )
            }) {
                continue;
            }
        }
        if import.computed
            || matches!(
                import.kind,
                crate::codebase::dependencies::extract::ImportKind::Dynamic
                    | crate::codebase::dependencies::extract::ImportKind::Require
            )
        {
            selector_unknown(
                &inputs,
                &import.specifier,
                import.line as usize,
                "dynamic or require selector",
                &mut edges,
                &mut sites,
            );
        }
    }
    (edges, sites)
}
