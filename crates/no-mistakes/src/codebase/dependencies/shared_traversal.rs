pub(crate) struct SharedTraversalContext {
    root: PathBuf,
    tsconfig: TsConfig,
    graph_files: graph::GraphFiles,
    visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
    config: crate::config::v2::NoMistakesConfig,
    config_path: Option<PathBuf>,
    prepared_graph: graph::PreparedGraphConfig,
    build_plan: graph::GraphBuildPlan,
    fact_plan: crate::codebase::ts_source::facts::TsFactPlan,
    fact_context: crate::codebase::ts_source::facts::TsFactContext,
    prepared_test_projects: Option<crate::codebase::test_discovery::PreparedTestProjects>,
    test_filter: crate::codebase::test_filter::TestFileFilter,
    facts: Option<crate::codebase::ts_source::facts::TsFactMap>,
    graph: Option<graph::DepGraph>,
    pub(crate) graph_builds: usize,
}

impl SharedTraversalContext {
    pub(crate) fn prepare(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
    ) -> Result<Self> {
        let visible_paths = std::sync::Arc::new(
            crate::codebase::ts_source::VisiblePathSnapshot::new(&root),
        );
        Self::prepare_with_snapshot(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            visible_paths,
        )
    }

    pub(crate) fn prepare_with_snapshot(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
        visible_paths: std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot>,
    ) -> Result<Self> {
        let root_visible_paths = visible_paths.paths_for(&root);
        let graph_files = graph::GraphFiles::from_files(
            crate::codebase::ts_source::discover_files_from_visible(
                &root,
                &[],
                &root_visible_paths,
            ),
        );
        let config = crate::config::v2::load_v2_config_from_visible(
            &root,
            config_path,
            &root_visible_paths,
        )?;
        let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
            tsconfig_path,
            &root,
            &root_visible_paths,
        )?;
        let codebase_config =
            crate::codebase::config::config_from_loaded_v2(&root, config_path, &config);
        let preliminary_graph = graph::prepare_graph_config_with_test_filter(
            &root,
            build_plan,
            &codebase_config,
            &config,
            visible_paths.as_ref(),
            crate::codebase::test_filter::TestFileFilter::fallback_only(),
        )?;
        let (preliminary_fact_plan, preliminary_fact_context) =
            graph::ts_fact_plan_and_context_for_plan_with_prepared(
                &root,
                build_plan,
                &preliminary_graph,
            );
        let prepared_test_projects =
            crate::codebase::test_discovery::prepare_test_projects_from_visible(
                &root,
                &config,
                &root_visible_paths,
                &tsconfig,
                graph_files.indexable(),
                preliminary_fact_plan,
                preliminary_fact_context,
            );
        let test_filter = crate::codebase::test_filter::TestFileFilter::from_prepared_projects(
            &root,
            &config,
            &root_visible_paths,
            prepared_test_projects.project_filters(),
        );
        let prepared_graph = graph::prepare_graph_config_with_test_filter(
            &root,
            build_plan,
            &codebase_config,
            &config,
            visible_paths.as_ref(),
            test_filter.clone(),
        )?;
        let (fact_plan, mut fact_context) =
            graph::ts_fact_plan_and_context_for_plan_with_prepared(
                &root,
                build_plan,
                &prepared_graph,
            );
        fact_context.set_visible_files(graph_files.visible().iter().cloned());
        Ok(Self {
            root,
            tsconfig,
            graph_files,
            visible_paths,
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
            graph_builds: 0,
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

    pub(crate) fn visible_paths(
        &self,
    ) -> &crate::codebase::ts_source::VisiblePathSnapshot {
        self.visible_paths.as_ref()
    }

    pub(crate) fn visible_paths_arc(
        &self,
    ) -> std::sync::Arc<crate::codebase::ts_source::VisiblePathSnapshot> {
        self.visible_paths.clone()
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

    pub(crate) fn prepared_graph(&self) -> &graph::PreparedGraphConfig {
        &self.prepared_graph
    }

}
