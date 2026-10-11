use super::super::prepared_plan::PreparedTestPlanRequest;
use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::DepGraph;

pub(super) fn prepared_graph<'a>(
    prepared: &'a PreparedTestPlanRequest,
    timing: Option<&mut crate::impacted_checks::timing::TimingTracker>,
) -> Result<&'a DepGraph> {
    if prepared.graph_is_initialized() {
        prepared.graph()
    } else if let Some(timing) = timing {
        timing.run_phase("graph", || prepared.graph())
    } else {
        prepared.graph()
    }
}
