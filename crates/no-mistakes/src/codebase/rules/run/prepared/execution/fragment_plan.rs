use super::Prerequisite;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) const CALL_GRAPH: &str = "codebase.canonical-graph-calls";

pub(super) fn note_call_graph<'a>(
    session: &AnalysisSession,
    calls: bool,
    graph: &'a DepGraph,
) -> Option<&'a DepGraph> {
    if calls {
        session.set_check_prerequisite(CALL_GRAPH, Prerequisite::Ready(()));
    }
    Some(graph)
}

/// Files for the shared graph. An incomplete value-flow file is omitted so the
/// full call plan can still build. The fragment rule gaps that file itself.
pub(super) fn shared_graph_files(
    config: &NoMistakesConfig,
    session: &AnalysisSession,
    shared: &CheckFactMap,
    graph_plan: &Prerequisite<Option<GraphBuildPlan>>,
) -> Vec<PathBuf> {
    let files = shared.graph_file_universe().to_vec();
    let Prerequisite::Ready(Some(plan)) = graph_plan else {
        return files;
    };
    let fragment = crate::codebase::rules::POSTGRES_SQL_FRAGMENT_ESCAPE;
    if !plan.ts_fact_plan().needs_value_flow()
        || !session.partial_checks_enabled()
        || !session.check_consumer_ready(fragment)
        || !config.rule_configured(fragment)
    {
        return files;
    }
    files
        .into_iter()
        .filter(|file| value_flow_complete(shared, file))
        .collect()
}

fn value_flow_complete(shared: &CheckFactMap, file: &Path) -> bool {
    shared
        .ts
        .get(file)
        .is_none_or(|facts| facts.ts.value_flow.incomplete_reason.is_none())
}
