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
    let graph_plan = signature_impact_graph_plan();
    // Discover files and parse imports+symbols facts once, then hand the same
    // `TsFactMap` to both the `DepGraph` build and `prepare_local_caller_context`
    // below, instead of letting the graph build parse internally and then having
    // `prepare_local_caller_context` parse the same file set again from scratch.
    // `signature_impact_graph_plan()`'s effective `TsFactPlan` (imports + workspace
    // + symbols) is always a superset of the `imports_and_symbols` shape
    // `local_caller_entries` needs, so one parse pass covers both consumers.
    let graph_files = GraphFiles::discover(&root);
    let (fact_plan, fact_context) =
        ts_fact_plan_and_context_for_plan_with_config(&root, graph_plan, args.config.as_deref());
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        graph_files.indexable(),
        fact_plan,
        &fact_context,
    );
    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        graph_plan,
        &graph_files,
        args.config.as_deref(),
        Some(&facts as &dyn TsFactLookup),
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
    let local_caller_context = prepare_local_caller_context(facts, graph_files.all(), &root);
    let production_extra_callers = local_caller_entries(
        &local_caller_context,
        &target_symbols,
        &root,
        &tsconfig,
        &test_filter,
        false,
    );
    let test_extra_callers = local_caller_entries(
        &local_caller_context,
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

/// Shared, `want_tests`-independent inputs for [`local_caller_entries`], computed once per
/// `signature-impact` run and reused for both the production and test passes.
struct LocalCallerContext {
    facts: crate::codebase::ts_source::facts::TsFactMap,
    workspace: crate::codebase::workspaces::WorkspaceMap,
}

/// Resolves the workspace map once and pairs it with the already-collected import/symbol
/// `facts` (built once in [`collect_report`] and shared with the `DepGraph` build), so
/// [`local_caller_entries`] can be called for both `want_tests` values without repeating the
/// file walk, parse pass, or workspace resolution.
///
/// `facts` must already cover `TsFactPlan::imports_and_symbols()` — `collect_report` builds it
/// from `signature_impact_graph_plan()`'s effective fact plan, a strict superset (imports +
/// workspace + symbols), so no re-parse happens here.
///
/// `discovery_files` must be `GraphFiles::all()` (the same `.gitignore`-aware walk
/// `GraphFiles::discover` already ran to seed `graph/builder.rs`'s own
/// `workspaces::load_from_files` call) — the caller passes it in rather than this function
/// re-discovering it, so a second `discover_files`/`git ls-files` pass doesn't happen. It must
/// specifically be `all()`, not `indexable()` (the narrower list backing `facts`): the graph
/// only indexes TS/JS file nodes, which never include `package.json`, so feeding that narrower
/// list into `load_from_files` would silently resolve zero workspace packages.
fn prepare_local_caller_context(
    facts: crate::codebase::ts_source::facts::TsFactMap,
    discovery_files: &[PathBuf],
    root: &Path,
) -> LocalCallerContext {
    let workspace =
        crate::codebase::workspaces::load_from_files(root, discovery_files).unwrap_or_default();
    LocalCallerContext { facts, workspace }
}

include!("impact_collect_exports.rs");

#[cfg(test)]
mod impact_collect_caller_tests;
#[cfg(test)]
mod impact_collect_caller_context_tests;
#[cfg(test)]
mod impact_collect_tests;
