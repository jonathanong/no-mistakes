use super::super::RunnerConfigFactPlan;
use std::sync::Arc;

pub(super) fn prepare(
    plan: &RunnerConfigFactPlan,
) -> [Arc<crate::codebase::ts_source::facts::TsFactContext>; 2] {
    let graph_plan = crate::codebase::check_facts::CheckFactPlan {
        graph: plan.graph_plan.graph,
        graph_context: plan.graph_plan.graph_context.clone(),
        integration_runner_configs: plan.graph_plan.integration_runner_configs.clone(),
        ..Default::default()
    };
    [
        Arc::new(crate::codebase::check_facts::ts_extract_context(
            &plan.root,
            &plan.primary_plan,
        )),
        Arc::new(crate::codebase::check_facts::ts_extract_context(
            &plan.root,
            &graph_plan,
        )),
    ]
}
