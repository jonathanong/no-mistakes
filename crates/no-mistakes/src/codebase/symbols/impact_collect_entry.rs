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
    let graph_plan = signature_impact_graph_plan();
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, args.config.as_deref(), &config);
    let dataset = session.dataset(&root);
    let workspace = dataset.workspace();
    let framework_plan = crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(
        graph_plan,
    );
    let (tsconfig_catalog, prepared_test_projects) =
        crate::codebase::dependencies::prepare_tsconfig_catalog_with_framework_projects(
            crate::codebase::dependencies::FrameworkCatalogPreparation {
                root: &root,
                tsconfig_path: args.tsconfig.as_deref(),
                tsconfig: &tsconfig,
                config: &config,
                codebase_config: &codebase_config,
                workspace: &workspace,
                root_visible_paths: &root_visible_paths,
                visible_paths: &visible_paths,
                sources: dataset.sources_for(&root),
                build_plan: graph_plan,
                graph_files: &graph_files,
                collect_graph_facts: false,
                framework_plan: &framework_plan,
            },
        )?;
    let test_filter = crate::codebase::test_filter::TestFileFilter::from_prepared_projects(
        &root,
        &config,
        &root_visible_paths,
        prepared_test_projects.project_filters(),
    );
    let prepared_graph =
        crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter_and_workspace(
            &root,
            graph_plan,
            &codebase_config,
            &config,
            &visible_paths,
            test_filter.clone(),
            workspace,
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
    crate::invocation::check_timeout()?;
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
        crate::codebase::dependencies::graph::PreparedGraphBuild {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: Some(&tsconfig_catalog),
            plan: graph_plan,
            graph_files: &graph_files,
            config_path: args.config.as_deref(),
            prepared: &prepared_graph,
            facts: Some(&facts),
            import_resolution_cache: None,
            dotnet_facts: None,
            swift_facts: None,
            visible_paths: Some(&visible_paths),
        },
        std::sync::Arc::clone(&session),
    )?;
    build_report_from_prepared(
        &PreparedReportContext {
            args,
            root: &root,
            tsconfig_catalog: &tsconfig_catalog,
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
        tsconfig_catalog,
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
            tsconfig_catalog,
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
