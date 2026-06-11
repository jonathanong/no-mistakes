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
        signature_impact_graph_plan(),
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
    let impact_edges = signature_impact_edges();
    let mut entries =
        graph.dependents_of_symbol_nodes(std::slice::from_ref(&target), None, Some(&impact_edges));
    let (exports, export_nodes) = export_paths(&graph, &target, symbol, &root, &definition);
    let target_symbols = signature_target_symbols(&target_file, symbol, &export_nodes);
    let file_import_edges = HashSet::from([EdgeKind::DynamicImport, EdgeKind::Require]);
    let mut file_roots: Vec<_> = export_nodes
        .iter()
        .filter_map(NodeId::as_file)
        .map(|path| NodeId::File(path.to_path_buf()))
        .collect();
    file_roots.push(NodeId::File(target_file.clone()));
    file_roots.sort();
    file_roots.dedup();
    let mut file_entry_target_symbols: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file_root in file_roots {
        let Some(root_file) = file_root.as_file() else {
            continue;
        };
        let symbols_for_root = target_symbols.get(root_file).cloned().unwrap_or_default();
        let file_entries = graph.dependents_of(
            std::slice::from_ref(&file_root),
            Some(1),
            Some(&file_import_edges),
        );
        if !symbols_for_root.is_empty() {
            for entry in &file_entries {
                if has_file_level_import_edge(&entry.via) {
                    if let Some(path) = entry.node.as_file() {
                        file_entry_target_symbols
                            .entry(relative_slash_path(&root, path))
                            .or_default()
                            .extend(symbols_for_root.iter().cloned());
                    }
                }
            }
        }
        entries.extend(file_entries);
    }
    let production_extra_callers =
        local_caller_entries(&graph, &target_symbols, &root, &tsconfig, &test_filter, false);
    let test_extra_callers = local_caller_entries(
        &graph,
        &target_symbols,
        &root,
        &tsconfig,
        &test_filter,
        true,
    );
    let suggested_entries = suggested_test_entries(
        &graph,
        &entries,
        &production_extra_callers,
        &root,
        &file_entry_target_symbols,
    );
    let suggested_tests = suggested_tests(
        &suggested_entries,
        &root,
        &test_filter,
        &test_extra_callers,
        &file_entry_target_symbols,
    );
    let caller_context = CallerEntriesContext {
        root: &root,
        test_filter: &test_filter,
        export_nodes: &export_nodes,
        file_target_symbols: &file_entry_target_symbols,
    };

    Ok(SignatureImpactReport {
        roots: vec![format!("{}#{}", args.files[0].display(), symbol)],
        symbol: symbol.to_string(),
        definition,
        production_callers: caller_entries(
            &entries,
            &caller_context,
            false,
            &production_extra_callers,
        ),
        test_callers: caller_entries(&entries, &caller_context, true, &test_extra_callers),
        warnings: warnings(&suggested_tests),
        exports,
        suggested_tests,
    })
}

include!("impact_collect_exports.rs");

#[cfg(test)]
mod impact_collect_caller_tests;
#[cfg(test)]
mod impact_collect_tests;
