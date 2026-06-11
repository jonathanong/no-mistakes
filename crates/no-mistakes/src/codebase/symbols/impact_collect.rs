pub fn collect_report(args: &SymbolsArgs) -> Result<SignatureImpactReport> {
    if args.files.len() != 1 {
        bail!("signature-impact mode requires exactly one file");
    }
    let Some(symbol) = args.symbol.as_deref().filter(|value| !value.is_empty()) else {
        bail!("signature-impact mode requires --symbol <SYMBOL>");
    };

    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = resolve_root(args.root.as_deref(), &cwd);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = resolve_tsconfig(args.tsconfig.as_deref(), &root)?;
    let abs_files = resolve_input_files(&args.files, &root, &cwd);
    let target_file = crate::codebase::ts_resolver::normalize_path(&abs_files[0]);

    let config = load_v2_config(&root, args.config.as_deref())?;
    let test_filter = TestFileFilter::new(&root, &config);
    let graph = DepGraph::build_with_plan_and_config(
        &root,
        &tsconfig,
        GraphBuildPlan::all().with_symbols(true),
        args.config.as_deref(),
    )?;
    let target = NodeId::Symbol {
        file: target_file.clone(),
        symbol: symbol.to_string(),
    };
    let definition = if let Some(location) = export_location(&target_file, &root, symbol, false)? {
        location
    } else if graph.dependencies_of_node(&target).is_some()
        || graph.dependents_of_node(&target).is_some()
    {
        let Some(location) = export_location(&target_file, &root, symbol, true)? else {
            bail!(
                "`{}` is not exported by `{}`",
                symbol,
                args.files[0].display()
            );
        };
        location
    } else {
        bail!(
            "`{}` is not exported by `{}`",
            symbol,
            args.files[0].display()
        );
    };
    let entries = graph.dependents_of_symbol_nodes(std::slice::from_ref(&target), None, None);

    let (exports, export_nodes) = export_paths(&graph, &target, symbol, &root, &definition);
    let suggested_tests = suggested_tests(&entries, &root, &test_filter);

    Ok(SignatureImpactReport {
        roots: vec![format!("{}#{}", args.files[0].display(), symbol)],
        symbol: symbol.to_string(),
        definition,
        production_callers: caller_entries(&entries, &root, &test_filter, false, &export_nodes),
        test_callers: caller_entries(&entries, &root, &test_filter, true, &export_nodes),
        warnings: warnings(&suggested_tests),
        exports,
        suggested_tests,
    })
}

fn export_paths(
    graph: &DepGraph,
    target: &NodeId,
    target_symbol: &str,
    root: &Path,
    definition: &SymbolLocation,
) -> (Vec<SymbolLocation>, BTreeSet<NodeId>) {
    let mut exports = BTreeSet::from([definition.clone()]);
    let mut export_nodes = BTreeSet::from([target.clone()]);
    let mut frontier = vec![target.clone()];
    let mut seen = BTreeSet::from([target.clone()]);
    while let Some(node) = frontier.pop() {
        if let Some(neighbors) = graph.dependents_of_node(&node) {
            for (neighbor, _) in neighbors {
                let NodeId::Symbol { file, symbol } = neighbor else {
                    continue;
                };
                if seen.insert(neighbor.clone()) {
                    if let Some(location) = export_location(file, root, symbol, true).ok().flatten() {
                        let local_import_export = location.symbol == target_symbol
                            && symbol == target_symbol
                            && std::fs::read_to_string(file)
                                .ok()
                                .and_then(|source| {
                                    let is_tsx = file
                                        .extension()
                                        .and_then(|s| s.to_str())
                                        .is_some_and(|ext| {
                                            ext.eq_ignore_ascii_case("tsx")
                                                || ext.eq_ignore_ascii_case("jsx")
                                        });
                                    extract_symbols(&source, is_tsx).ok()
                                })
                                .and_then(|symbols| {
                                    let local = symbols.exports.iter().find_map(|export| {
                                        (!matches!(
                                            export.kind,
                                            ExportKind::ReExport { .. }
                                        ) && export.name == *symbol)
                                            .then_some(export.local.as_deref())
                                            .flatten()
                                    })?;
                                    symbols.imports.iter().any(|import| {
                                        import.local == local && import.imported == target_symbol
                                    }).then_some(())
                                })
                                .is_some();
                        if location.kind == "re-export" || local_import_export {
                            frontier.push(neighbor.clone());
                            exports.insert(location);
                            export_nodes.insert(neighbor.clone());
                        }
                    }
                }
            }
        }
    }
    (exports.into_iter().collect(), export_nodes)
}

fn caller_entries(
    entries: &[NodeEntry],
    root: &Path,
    test_filter: &TestFileFilter,
    want_tests: bool,
    export_nodes: &BTreeSet<NodeId>,
) -> Vec<CallerEntry> {
    let mut by_key: BTreeMap<(String, Option<String>), CallerEntry> = BTreeMap::new();
    for entry in entries {
        if export_nodes.contains(&entry.node) {
            continue;
        }
        let Some((file, symbol)) = caller_parts(&entry.node, root) else {
            continue;
        };
        let is_test = entry
            .node
            .as_file()
            .is_some_and(|path| test_filter.is_match(root, path));
        if is_test != want_tests {
            continue;
        }
        insert_caller(&mut by_key, entry, file, symbol);
    }
    let mut callers: Vec<_> = by_key.into_values().collect();
    callers.sort_by(|a, b| caller_sort_key(a).cmp(&caller_sort_key(b)));
    callers
}

fn insert_caller(
    by_key: &mut BTreeMap<(String, Option<String>), CallerEntry>,
    entry: &NodeEntry,
    file: String,
    symbol: Option<String>,
) {
    let via = via_strings(&entry.via);
    by_key
        .entry((file.clone(), symbol.clone()))
        .and_modify(|existing| {
            existing.depth = existing.depth.min(entry.depth);
            merge_via(&mut existing.via, &via);
        })
        .or_insert(CallerEntry {
            file,
            symbol,
            depth: entry.depth,
            via,
        });
}

fn caller_sort_key(caller: &CallerEntry) -> (usize, &str, &str) {
    (
        caller.depth,
        caller.file.as_str(),
        caller.symbol.as_deref().unwrap_or_default(),
    )
}

#[cfg(test)]
mod impact_collect_tests;
