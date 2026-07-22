impl SharedTraversalContext {
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn tsconfig(&self) -> &TsConfig {
        &self.tsconfig
    }

    pub(crate) fn tsconfig_catalog(&self) -> &crate::codebase::ts_resolver::TsConfigCatalog {
        &self.tsconfig_catalog
    }

    pub(crate) fn tsconfig_catalog_arc(
        &self,
    ) -> std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog> {
        std::sync::Arc::clone(&self.tsconfig_catalog)
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
