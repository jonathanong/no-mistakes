impl SharedTraversalContext {
    fn prepare_with_dataset_session_and_framework_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        dataset: std::sync::Arc<crate::codebase::analysis_dataset::AnalysisDataset>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
        include_check_plan: bool,
        mut framework_plan: crate::codebase::test_discovery::FrameworkPreparationPlan,
    ) -> Result<Self> {
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
        let tsconfig = (*session.tsconfig(&root, tsconfig_path)?).clone();
        let codebase_config =
            crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
        let workspace = dataset.workspace();
        let preliminary_graph = graph::prepare_graph_config_with_test_filter_and_workspace(
            &root,
            build_plan,
            &codebase_config,
            &config,
            dataset.visible_paths(),
            crate::codebase::test_filter::TestFileFilter::fallback_only(),
            std::sync::Arc::clone(&workspace),
        )?;
        let (preliminary_fact_plan, preliminary_fact_context) =
            graph::ts_fact_plan_and_context_for_plan_with_prepared(
                &root,
                build_plan,
                &preliminary_graph,
            );
        let collect_graph_facts = !include_check_plan;
        let preliminary_graph_files = if collect_graph_facts {
            graph_files.indexable()
        } else {
            &[]
        };
        let prepared_test_projects =
            crate::codebase::test_discovery::prepare_test_projects_from_visible_with_sources_and_plan(
                &root,
                &config,
                &root_visible_paths,
                &tsconfig,
                crate::codebase::test_discovery::PreparedTestProjectRequest {
                    graph: (
                        preliminary_graph_files,
                        preliminary_fact_plan,
                        preliminary_fact_context,
                    ),
                    sources: dataset.sources_for(&root),
                    collect_graph_facts,
                    preparation_plan: &framework_plan,
                },
            );
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
