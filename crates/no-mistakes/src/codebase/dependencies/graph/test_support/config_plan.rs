pub(super) fn graph_config_options_for_plan(
    root: &Path,
    plan: GraphBuildPlan,
) -> Option<GraphConfigOptions> {
    if graph_plan_needs_config(plan) {
        graph_config_options(root)
    } else {
        None
    }
}
