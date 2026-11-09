fn folded_import_sites<'a>(
    inputs: &ExportEdgeInputs<'_>,
    facts: &'a crate::codebase::ts_source::facts::TsFileFacts,
    exported: &FxHashSet<NodeId>,
    edges: &mut Vec<Edge>,
    sites: &mut SelectorImportSites,
    imports: bool,
) -> FxHashSet<(&'a str, u32)> {
    let (Some(module), Some(source)) = (&facts.module_bindings, facts.source.as_deref()) else {
        return fx_set();
    };
    if module.folded_dynamic_imports.is_empty() {
        return fx_set();
    }
    let mock_spans: FxHashSet<_> = module
        .mock_import_spans
        .iter()
        .map(|span| (span.start, span.end))
        .collect();
    let mut folded_counts: FxHashMap<(&str, u32), usize> = fx_map();
    let reachable = reachable_function_scopes(facts);
    let mut originals: FxHashMap<(&str, u32), (usize, Option<&ExtractedImport>)> = fx_map();
    for import in &facts.imports {
        if import.computed && import.kind == ImportKind::Dynamic {
            let (count, representative) = originals
                .entry((&import.specifier, import.line))
                .or_insert((0, Some(import)));
            *count += 1;
            if representative.is_some_and(|first| {
                first.function_scope_id != import.function_scope_id
                    || first.function_scope != import.function_scope
                    || first.runtime_reachable != import.runtime_reachable
            }) {
                *representative = None;
            }
        }
    }
    let selectors: FxHashMap<_, _> = module
        .dynamic_import_selectors
        .iter()
        .map(|(start, names, calls)| (*start, (names, calls)))
        .collect();
    for folded in &module.folded_dynamic_imports {
        let load = &folded.load;
        let line =
            crate::codebase::ts_source::byte_offset_to_line(source, load.span.start as usize);
        if mock_spans.contains(&(load.span.start, load.span.end)) {
            *folded_counts
                .entry((folded.source.as_str(), line))
                .or_default() += 1;
            continue;
        }
        let specifier = if folded.relative_url {
            let Some(path) = url::Url::from_file_path(inputs.path)
                .ok()
                .and_then(|base| base.join(&load.specifier).ok())
                .filter(|url| !folded.pathname || !url.path().contains('%'))
                .and_then(|url| url.to_file_path().ok())
            else {
                let target = NodeId::module("<unresolved URL import>");
                push_selector_site(
                    inputs,
                    target.clone(),
                    SelectorImportSite {
                        line: line as usize,
                        specifier: load.specifier.clone(),
                        reason: "unresolved URL import",
                    },
                    sites,
                );
                edges.push((
                    NodeId::file_in(inputs.interner, inputs.path),
                    target,
                    EdgeKind::SymbolImportUnknown,
                ));
                continue;
            };
            path.to_string_lossy().into_owned()
        } else {
            load.specifier.clone()
        };
        *folded_counts
            .entry((folded.source.as_str(), line))
            .or_default() += 1;
        if let (Some(originals), Some(target)) = (
            originals.get(&(folded.source.as_str(), line)),
            selector_target(inputs, &specifier),
        ) {
            if let (_, Some(original)) = originals {
                let mut import = (*original).clone();
                import.computed = false;
                if let Some(kind) = imports
                    .then(|| graph_edge_kind_for_extracted_import(&import, facts, &reachable))
                    .flatten()
                {
                    edges.push((
                        NodeId::file_in(inputs.interner, inputs.path),
                        NodeId::file_in(inputs.interner, &target),
                        kind,
                    ));
                }
            } else {
                let target = NodeId::module("<ambiguous dynamic import occurrence>");
                push_selector_site(
                    inputs,
                    target.clone(),
                    SelectorImportSite {
                        line: line as usize,
                        specifier: specifier.clone(),
                        reason: "ambiguous dynamic import occurrence",
                    },
                    sites,
                );
                edges.push((
                    NodeId::file_in(inputs.interner, inputs.path),
                    target,
                    EdgeKind::SymbolImportUnknown,
                ));
            }
        }
        let names = selectors
            .get(&load.span.start)
            .filter(|(_, calls)| {
                calls.is_empty()
                    || selector_target(inputs, &specifier)
                        .is_some_and(|target| receiver_safe(inputs, &target, calls))
            })
            .map(|(names, _)| names.iter().map(String::as_str).collect::<Vec<_>>());
        if !names.as_ref().is_some_and(|names| {
            push_dynamic_selectors(
                inputs,
                &specifier,
                line as usize,
                names,
                exported,
                edges,
                sites,
            )
        }) {
            selector_unknown(
                inputs,
                &specifier,
                line as usize,
                "dynamic or require selector",
                edges,
                sites,
            );
        }
    }
    folded_counts
        .into_iter()
        .filter_map(|(site, count)| {
            originals
                .get(&site)
                .is_some_and(|(originals, _)| *originals == count)
                .then_some(site)
        })
        .collect()
}
