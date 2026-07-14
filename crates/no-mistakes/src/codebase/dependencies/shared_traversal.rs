pub(crate) struct SharedTraversalContext {
    root: PathBuf,
    tsconfig: TsConfig,
    graph_files: graph::GraphFiles,
    dataset: crate::codebase::analysis_dataset::AnalysisDataset,
    config: crate::config::v2::NoMistakesConfig,
    config_path: Option<PathBuf>,
    prepared_graph: graph::PreparedGraphConfig,
    build_plan: graph::GraphBuildPlan,
    fact_plan: crate::codebase::ts_source::facts::TsFactPlan,
    fact_context: crate::codebase::ts_source::facts::TsFactContext,
    prepared_test_projects: Option<crate::codebase::test_discovery::PreparedTestProjects>,
    test_filter: crate::codebase::test_filter::TestFileFilter,
    facts: Option<crate::codebase::ts_source::facts::TsFactMap>,
    graph: Option<std::sync::Arc<graph::DepGraph>>,
    graph_cache: SharedBuildCache<EffectiveGraphPlanKey, graph::DepGraph>,
    symbol_index_cache: SharedBuildCache<GraphFileUniverseKey, graph::SymbolIndex>,
    import_resolution_cache: crate::codebase::ts_resolver::ImportResolutionCache,
    analysis_generation: u64,
    pub(crate) graph_builds: usize,
    pub(crate) symbol_index_builds: usize,
}

impl SharedTraversalContext {
    pub(crate) fn prepare(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
    ) -> Result<Self> {
        Self::prepare_with_dataset(
            root.clone(),
            tsconfig_path,
            config_path,
            build_plan,
            crate::codebase::analysis_dataset::AnalysisDataset::new(&root),
            false,
        )
    }

    pub(crate) fn prepare_with_snapshot_and_optional_check_plan(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
        include_check_plan: bool,
    ) -> Result<Self> {
        Self::prepare_with_dataset(
            root.clone(),
            tsconfig_path,
            config_path,
            build_plan,
            crate::codebase::analysis_dataset::AnalysisDataset::from_snapshot(&root, visible_paths),
            include_check_plan,
        )
    }

    fn prepare_with_dataset(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        dataset: crate::codebase::analysis_dataset::AnalysisDataset,
        include_check_plan: bool,
    ) -> Result<Self> {
        let root_visible_paths = dataset.paths_for(&root);
        let graph_files =
            graph::GraphFiles::from_files(crate::codebase::ts_source::discover_files_from_visible(
                &root,
                &[],
                &root_visible_paths,
            ));
        let config = (*dataset.config(config_path)?).clone();
        let mut build_plan = build_plan;
        if include_check_plan {
            if let Some(check_plan) = crate::codebase::rules::canonical_graph_plan(&config) {
                build_plan.include(check_plan);
            }
        }
        let tsconfig = (*dataset.tsconfig(tsconfig_path)?).clone();
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
        let preliminary_graph_files = if include_check_plan {
            &[][..]
        } else {
            graph_files.indexable()
        };
        let prepared_test_projects =
            crate::codebase::test_discovery::prepare_test_projects_from_visible_with_sources(
                &root,
                &config,
                &root_visible_paths,
                &tsconfig,
                (
                    preliminary_graph_files,
                    preliminary_fact_plan,
                    preliminary_fact_context,
                ),
                dataset.sources_for(&root),
                !include_check_plan,
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
            analysis_generation: 0,
            graph_builds: 0,
            symbol_index_builds: 0,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn tsconfig(&self) -> &TsConfig {
        &self.tsconfig
    }

    pub(crate) fn graph_files(&self) -> &graph::GraphFiles {
        &self.graph_files
    }

    pub(crate) fn visible_paths(&self) -> &crate::codebase::ts_source::VisiblePathSnapshot {
        self.dataset.visible_paths()
    }

    pub(crate) fn source_store(&self) -> std::sync::Arc<crate::codebase::ts_source::SourceStore> {
        self.dataset.sources_for(&self.root)
    }

    pub(crate) fn visible_paths_arc(
        &self,
    ) -> std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot> {
        self.dataset.visible_paths_arc()
    }

    pub(crate) fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    pub(crate) fn config(&self) -> &crate::config::v2::NoMistakesConfig {
        &self.config
    }

    pub(crate) fn build_plan(&self) -> graph::GraphBuildPlan {
        self.build_plan
    }

    pub(crate) fn canonical_graph(&mut self) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.request_graph(self.build_plan)
    }

    pub(crate) fn prepared_graph(&self) -> &graph::PreparedGraphConfig {
        &self.prepared_graph
    }
}
