use super::*;

#[test]
fn graph_config_options_are_loaded_only_for_config_driven_plans() {
    let explicit =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan::imports_and_workspace()
    )
    .is_none());

    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            routes: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            queues: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
    assert!(graph_config_options_for_plan(
        &explicit,
        GraphBuildPlan {
            http: true,
            ..GraphBuildPlan::default()
        }
    )
    .is_some());
}
