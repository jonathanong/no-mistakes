use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::FileIdMap;
use std::path::{Path, PathBuf};

pub(crate) struct PrecollectedRouteFacts {
    pub(crate) ts: crate::codebase::ts_source::facts::TsFactMap,
    pub(crate) routes: std::collections::BTreeMap<
        PathBuf,
        Vec<crate::playwright::integration_routes::RouteOccurrence>,
    >,
}

pub(crate) fn collect_with_precollected_ts(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    graph_files_complete: bool,
    plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected: PrecollectedRouteFacts,
) -> CheckFactMap {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let sources =
        super::super::collect::request_sources(&files, &graph_files, &plan, Some(&playwright));
    collect_precollected_route_facts(
        &session,
        root,
        (files, graph_files, graph_files_complete),
        plan,
        playwright,
        precollected,
        sources,
    )
}

pub(crate) fn collect_precollected_route_facts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected: PrecollectedRouteFacts,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    super::collect_with_precollected_ts_sources_and_session(
        session,
        root,
        file_scope,
        plan,
        playwright,
        super::PrecollectedFacts {
            ts: precollected.ts,
            files: FileIdMap::with_inventory(std::sync::Arc::clone(sources.inventory())),
            routes: precollected.routes,
        },
        sources,
    )
}
