use super::Prerequisite;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::config::v2::NoMistakesConfig;

pub(super) fn without_value_flow_when_partial(
    config: &NoMistakesConfig,
    session: &AnalysisSession,
    shared: &CheckFactMap,
    graph_plan: Prerequisite<Option<GraphBuildPlan>>,
) -> Prerequisite<Option<GraphBuildPlan>> {
    let Prerequisite::Ready(Some(mut plan)) = graph_plan else {
        return graph_plan;
    };
    if !plan.ts_fact_plan().needs_value_flow() {
        return Prerequisite::Ready(Some(plan));
    }
    let fragment = crate::codebase::rules::POSTGRES_SQL_FRAGMENT_ESCAPE;
    if !session.partial_checks_enabled()
        || !session.check_consumer_ready(fragment)
        || !config.rule_configured(fragment)
        || !value_flow_is_incomplete(shared)
    {
        return Prerequisite::Ready(Some(plan));
    }
    plan.calls = false;
    plan.identifier_provenance = false;
    Prerequisite::Ready(Some(plan))
}

fn value_flow_is_incomplete(shared: &CheckFactMap) -> bool {
    shared
        .files()
        .iter()
        .chain(shared.graph_file_universe())
        .any(|file| {
            shared
                .ts
                .get(file)
                .is_some_and(|facts| facts.ts.value_flow.incomplete_reason.is_some())
        })
}
