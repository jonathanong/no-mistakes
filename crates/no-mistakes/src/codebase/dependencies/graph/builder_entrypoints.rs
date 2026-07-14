impl DepGraph {
    pub fn build(root: &Path, tsconfig: &TsConfig) -> Result<Self> {
        Self::build_with_plan(root, tsconfig, GraphBuildPlan::all())
    }

    pub fn build_with_plan(root: &Path, tsconfig: &TsConfig, plan: GraphBuildPlan) -> Result<Self> {
        let graph_files = GraphFiles::discover(root);
        Self::build_with_plan_and_files(root, tsconfig, plan, &graph_files)
    }

    pub fn build_with_plan_and_config(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        config_path: Option<&Path>,
    ) -> Result<Self> {
        let graph_files = GraphFiles::discover(root);
        Self::build_with_plan_and_files_config(
            root,
            tsconfig,
            plan,
            &graph_files,
            config_path,
        )
    }

    pub(crate) fn build_with_plan_and_files(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
    ) -> Result<Self> {
        Self::build_with_plan_and_files_config(root, tsconfig, plan, graph_files, None)
    }

    pub(crate) fn build_with_plan_and_files_config(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        config_path: Option<&Path>,
    ) -> Result<Self> {
        Self::build_with_plan_files_config_and_facts(
            root,
            tsconfig,
            plan,
            graph_files,
            config_path,
            None,
        )
    }

}
