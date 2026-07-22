impl SharedTraversalContext {
    fn prepare_with_dataset_session_and_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        preparation: TraversalPreparationContext,
    ) -> Result<Self> {
        let TraversalPreparationContext {
            dataset,
            session,
            include_check_plan,
            mut framework_plan,
        } = preparation;
        session.record_work("analysis.requests", 1);
        let root_visible_paths = dataset.paths_for(&root);
        let config = (*session.config(&root, config_path)?).clone();
        let mut build_plan = build_plan;
        if include_check_plan {
            if let Some(check_plan) = crate::codebase::rules::canonical_graph_plan(&config) {
                build_plan.include(check_plan);
            }
        }
        framework_plan.include(
            crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(build_plan),
        );
        let excluded_configs =
            framework_plan.excluded_config_paths(&root, &config, &root_visible_paths);
        let graph_files = graph::GraphFiles::from_files_excluding_indexable(
            crate::codebase::ts_source::discover_files_from_visible(
                &root,
                &[],
                &root_visible_paths,
            ),
            &excluded_configs,
        );
        let tsconfig = session
            .tsconfig(&root, tsconfig_path)
            .map(|config| (*config).clone())
            .or_else(|error| {
                if tsconfig_path.is_some() {
                    Err(error)
                } else {
                    Ok(TsConfig {
                        dir: root.clone(),
                        paths: Vec::new(),
                        paths_dir: root.clone(),
                        base_url: None,
                    })
                }
            })?;
        let codebase_config =
            crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
        let workspace = dataset.workspace();
        let (tsconfig_catalog, prepared_test_projects) =
            prepare_tsconfig_catalog_with_framework_projects(FrameworkCatalogPreparation {
                root: &root,
                tsconfig_path,
                tsconfig: &tsconfig,
                config: &config,
                codebase_config: &codebase_config,
                workspace: &workspace,
                root_visible_paths: &root_visible_paths,
                visible_paths: dataset.visible_paths(),
                sources: dataset.sources_for(&root),
                build_plan,
                graph_files: &graph_files,
                collect_graph_facts: !include_check_plan,
                framework_plan: &framework_plan,
            })?;
        let tsconfig_build_diagnostics = tsconfig_catalog.diagnostics();
        let test_filter = crate::codebase::test_filter::TestFileFilter::from_prepared_projects(
            &root,
            &config,
            &root_visible_paths,
            prepared_test_projects.project_filters(),
        );
        let prepared_graph = graph::prepare_graph_config_with_test_filter_and_workspace(
            &root,
            build_plan,
            &codebase_config,
            &config,
            dataset.visible_paths(),
            test_filter.clone(),
            workspace,
        )?;
        let (fact_plan, mut fact_context) = graph::ts_fact_plan_and_context_for_plan_with_prepared(
            &root,
            build_plan,
            &prepared_graph,
        );
        fact_context.set_visible_files(graph_files.visible().iter().cloned());
        Ok(Self {
            session,
            root,
            tsconfig,
            tsconfig_catalog,
            tsconfig_build_diagnostics,
            graph_files,
            dataset,
            config,
            config_path: config_path.map(Path::to_path_buf),
            prepared_graph,
            build_plan,
            fact_plan,
            fact_context,
            facts: Some(prepared_test_projects.graph_facts().clone()),
            prepared_test_projects: Some(prepared_test_projects),
            test_filter,
            graph: None,
            graph_cache: SharedBuildCache::default(),
            symbol_index_cache: SharedBuildCache::default(),
            import_resolution_cache: Default::default(),
            traversal_results: Vec::new(),
            analysis_generation: 0,
            graph_builds: 0,
            symbol_index_builds: 0,
        })
    }
}
