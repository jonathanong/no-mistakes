/// Export names proven for a dynamic `import()` site, keyed by specifier and
/// line. Imports carry no byte offsets, so a site where any same-line,
/// same-specifier runtime occurrence is unproven stays unknown.
fn proven_dynamic_selectors(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> FxHashMap<(&str, u32), Vec<&str>> {
    let (Some(module), Some(source)) = (&facts.module_bindings, facts.source.as_deref()) else {
        return fx_map();
    };
    let mut sites: FxHashMap<(&str, u32), Option<Vec<&str>>> = fx_map();
    for load in module
        .loads
        .iter()
        .filter(|load| load.kind == "dynamicImport")
    {
        let line =
            crate::codebase::ts_source::byte_offset_to_line(source, load.span.start as usize);
        let proven = module
            .dynamic_import_selectors
            .iter()
            .find(|(start, _)| *start == load.span.start)
            .map(|(_, names)| names.iter().map(String::as_str).collect::<Vec<_>>());
        let entry = sites
            .entry((load.specifier.as_str(), line))
            .or_insert(Some(Vec::new()));
        *entry = match (entry.take(), proven) {
            (Some(mut all), Some(names)) => {
                all.extend(names);
                Some(all)
            }
            _ => None,
        };
    }
    sites
        .into_iter()
        .filter_map(|(site, names)| Some((site, names?)))
        .collect()
}

/// Records each proven name of a dynamic import as a runtime symbol consumer.
/// Returns false when the specifier does not resolve to an indexable file, so
/// the caller keeps its unknown-selector advisory.
fn push_dynamic_selectors(
    inputs: &ExportEdgeInputs<'_>,
    specifier: &str,
    line: usize,
    names: &[&str],
    exported: &FxHashSet<NodeId>,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
) -> bool {
    let Some(target) = selector_target(inputs, specifier).filter(|path| is_indexable(path)) else {
        return false;
    };
    for name in names {
        let symbol = NodeId::symbol_in(inputs.interner, &target, name);
        if !exported.contains(&symbol) {
            let site = SelectorImportSite {
                line,
                specifier: specifier.to_string(),
                reason: "unresolved named selector",
            };
            selector_unknown_symbol(inputs, &target, name, site, edges, sites);
        }
        edges.push((
            NodeId::file_in(inputs.interner, inputs.path),
            symbol,
            EdgeKind::SymbolImport,
        ));
    }
    true
}
