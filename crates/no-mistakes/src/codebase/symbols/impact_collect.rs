include!("impact_collect_entry.rs");

struct PreparedReportContext<'a> {
    args: &'a SymbolsArgs,
    root: &'a Path,
    tsconfig: &'a crate::codebase::ts_resolver::TsConfig,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
    graph_files: &'a GraphFiles,
    test_filter: &'a TestFileFilter,
    graph: &'a DepGraph,
    facts: &'a TsFactMap,
}

fn build_report_from_prepared(
    context: &PreparedReportContext<'_>,
    target_file: &Path,
    symbol: &str,
) -> Result<SignatureImpactReport> {
    let PreparedReportContext {
        args,
        root,
        tsconfig,
        session,
        graph_files,
        test_filter,
        graph,
        facts,
    } = context;
    let workspace = crate::codebase::workspaces::load_from_files_with_session(
        root,
        graph_files.all(),
        Some(session),
    )
    .unwrap_or_default();
    let visible_files = graph_files.visible().clone();
    let target = NodeId::Symbol {
        file: target_file.to_path_buf(),
        symbol: symbol.to_string(),
    };
    let definition = if let Some(location) =
        export_location(facts, target_file, root, symbol, false)?
    {
        location
    } else if graph.dependencies_of_node(&target).is_some()
        || graph.dependents_of_node(&target).is_some()
    {
        let Some(location) = export_location(facts, target_file, root, symbol, true)? else {
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
    let (exports, export_nodes) =
        export_paths(graph, facts, &target, symbol, root, &definition);
    let target_symbols = signature_target_symbols(
        target_file,
        symbol,
        &export_nodes,
        &visible_files,
        facts,
    );
    let file_import_edges = HashSet::from([EdgeKind::DynamicImport, EdgeKind::Require]);
    let mut file_roots: Vec<_> = export_nodes
        .iter()
        .filter_map(NodeId::as_file)
        .map(|path| NodeId::File(path.to_path_buf()))
        .collect();
    file_roots.push(NodeId::File(target_file.to_path_buf()));
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
                            .entry(relative_slash_path(root, path))
                            .or_default()
                            .extend(symbols_for_root.iter().cloned());
                    }
                }
            }
        }
        entries.extend(file_entries);
    }
    let local_caller_context = prepare_local_caller_context(facts, &workspace, &visible_files);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        tsconfig,
        Some(local_caller_context.visible_files),
        session,
    );
    let production_extra_callers = local_caller_entries(
        &local_caller_context,
        &target_symbols,
        root,
        &resolver,
        test_filter,
        false,
    );
    let test_extra_callers = local_caller_entries(
        &local_caller_context,
        &target_symbols,
        root,
        &resolver,
        test_filter,
        true,
    );
    let suggested_entries = suggested_test_entries(
        graph,
        &entries,
        &production_extra_callers,
        root,
        &file_entry_target_symbols,
        facts,
    );
    let suggested_tests = suggested_tests(
        &suggested_entries,
        root,
        test_filter,
        &test_extra_callers,
        &file_entry_target_symbols,
        facts,
    );
    let caller_context = CallerEntriesContext {
        root,
        test_filter,
        export_nodes: &export_nodes,
        file_target_symbols: &file_entry_target_symbols,
        facts,
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
struct LocalCallerContext<'a> {
    facts: &'a TsFactMap,
    workspace: &'a crate::codebase::workspaces::WorkspaceMap,
    visible_files: &'a HashSet<PathBuf>,
}

/// Resolves the workspace map once and pairs it with the already-collected import/symbol
/// `facts` (built once in [`collect_report`] and shared with the `DepGraph` build), so
/// [`local_caller_entries`] can be called for both `want_tests` values without repeating the
/// file walk, parse pass, or workspace resolution.
///
/// `facts`, `workspace`, and `visible_files` are all request-scoped values
/// prepared by the caller. Keeping this helper read-only prevents a second
/// parser pass or repository discovery while serving production and test callers.
fn prepare_local_caller_context<'a>(
    facts: &'a TsFactMap,
    workspace: &'a crate::codebase::workspaces::WorkspaceMap,
    visible_files: &'a HashSet<PathBuf>,
) -> LocalCallerContext<'a> {
    LocalCallerContext {
        facts,
        workspace,
        visible_files,
    }
}

include!("impact_collect_exports.rs");

#[cfg(test)]
mod impact_collect_caller_tests;
#[cfg(test)]
mod impact_collect_caller_context_tests;
#[cfg(test)]
mod impact_collect_tests;
