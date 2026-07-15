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
    let session = crate::codebase::analysis_session::AnalysisSession::new(
        crate::diagnostics::current(),
    );
    let visible_paths = session.visible_paths(&root);
    let root_visible_paths = visible_paths.paths_for(&root);
    let tsconfig = session.tsconfig(&root, args.tsconfig.as_deref())?;
    let abs_files = resolve_input_files(&args.files, &root, &cwd);
    let target_file = crate::codebase::ts_resolver::normalize_path(&abs_files[0]);
    let mut graph_file_paths = root_visible_paths.as_ref().clone();
    if target_file.is_file() && !graph_file_paths.contains(&target_file) {
        graph_file_paths.push(target_file.clone());
    }
    let graph_files = GraphFiles::from_files(graph_file_paths);
    let config = session.config(&root, args.config.as_deref())?;
    let test_filter = TestFileFilter::new(&root, &config);
    let graph_plan = signature_impact_graph_plan();
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, args.config.as_deref(), &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        graph_plan,
        &codebase_config,
        &config,
        &visible_paths,
    )?;
    let (mut fact_plan, mut fact_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            &root,
            graph_plan,
            &prepared_graph,
        );
    fact_plan.imports = true;
    fact_plan.function_calls = true;
    fact_plan.symbols = true;
    fact_plan.source = true;
    fact_context.set_visible_files(graph_files.visible().iter().cloned());
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_session_and_context(
        &session,
        graph_files.indexable(),
        fact_plan,
        &fact_context,
    );
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_and_session(
        crate::codebase::dependencies::graph::PreparedGraphBuildRequest {
            root: &root,
            tsconfig: &tsconfig,
            plan: graph_plan,
            graph_files: &graph_files,
            config_path: args.config.as_deref(),
            prepared: &prepared_graph,
            facts: Some(&facts),
        },
        std::sync::Arc::clone(&session),
    )?;
    build_report_from_prepared(
        &PreparedReportContext {
            args,
            root: &root,
            tsconfig: &tsconfig,
            session: &session,
            graph_files: &graph_files,
            test_filter: &test_filter,
            workspace: prepared_graph.workspace(),
            graph: &graph,
            facts: &facts,
        },
        &target_file,
        symbol,
    )
}

pub(super) fn collect_report_with_prepared(
    args: &SymbolsArgs,
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    prepared: PreparedSignatureImpact<'_>,
) -> Result<SignatureImpactReport> {
    if args.files.len() != 1 {
        bail!("signature-impact mode requires exactly one file");
    }
    let Some(symbol) = args.symbol.as_deref().filter(|value| !value.is_empty()) else {
        bail!("signature-impact mode requires --symbol <SYMBOL>");
    };
    let PreparedSignatureImpact {
        session,
        graph_files,
        test_filter,
        workspace,
        graph,
        facts,
    } = prepared;
    let cwd = std::env::current_dir().context("reading current directory")?;
    let target_file = crate::codebase::ts_resolver::normalize_path(
        &resolve_input_files(&args.files, root, &cwd)[0],
    );
    build_report_from_prepared(
        &PreparedReportContext {
            args,
            root,
            tsconfig,
            session,
            graph_files,
            test_filter,
            workspace,
            graph,
            facts,
        },
        &target_file,
        symbol,
    )
}
