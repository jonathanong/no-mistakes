use super::*;

impl SharedTraversalContext {
    pub(crate) fn prepare(
        root: PathBuf,
        tsconfig_path: Option<&Path>,
        config_path: Option<&Path>,
        build_plan: graph::GraphBuildPlan,
    ) -> Result<Self> {
        Self::prepare_with_framework_plan(
            root,
            tsconfig_path,
            config_path,
            build_plan,
            crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(build_plan),
        )
    }
}
