use super::{BatchCheckFactRequest, FactDemand};
use std::collections::BTreeMap;

pub(super) type PreparedContexts =
    Vec<[Option<crate::codebase::ts_source::facts::TsFactContext>; 2]>;

pub(super) fn prepare_contexts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    requests: &[BatchCheckFactRequest],
    demands: &BTreeMap<std::path::PathBuf, Vec<FactDemand>>,
) -> PreparedContexts {
    let mut contexts = requests
        .iter()
        .map(|_| [None, None])
        .collect::<PreparedContexts>();
    for demand in demands.values().flatten() {
        if contexts[demand.request][demand.context].is_some() {
            continue;
        }
        let request = &requests[demand.request];
        let graph_plan = super::super::CheckFactPlan {
            graph: request.plan.graph,
            graph_context: request.plan.graph_context.clone(),
            integration_runner_configs: request.plan.integration_runner_configs.clone(),
            ..super::super::CheckFactPlan::default()
        };
        let plan = if demand.context == 0 {
            &request.plan
        } else {
            &graph_plan
        };
        contexts[demand.request][demand.context] = Some(
            super::super::file::prepared_ts_extract_context(session, &request.root, plan),
        );
    }
    contexts
}
