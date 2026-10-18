use super::super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod scope;
mod supplemental;
use scope::collect_with_scope;
pub use supplemental::collect_check_facts_with_supplemental_call_sites;

pub fn collect_check_facts(root: &Path, files: Vec<PathBuf>, plan: CheckFactPlan) -> CheckFactMap {
    collect_check_facts_with_playwright(root, files, plan, None)
}

pub fn collect_check_facts_with_graph_files_and_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_check_facts_with_graph_files_playwright_and_session(
        &session,
        root,
        files,
        graph_files,
        plan,
        playwright,
    )
}

#[doc(hidden)]
pub fn collect_check_facts_with_graph_files_playwright_and_sources(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        root,
        (files, graph_files),
        plan,
        playwright,
        sources,
    )
}

#[doc(hidden)]
pub fn collect_check_facts_with_graph_files_playwright_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    let sources = session.existing_sources_for(root).unwrap_or_else(|| {
        super::request_sources(&files, &graph_files, &plan, playwright.as_ref())
    });
    collect_check_facts_with_graph_files_playwright_sources_and_session(
        session,
        root,
        (files, graph_files),
        plan,
        playwright,
        sources,
    )
}

#[doc(hidden)]
pub fn collect_check_facts_with_graph_files_playwright_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>),
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
) -> CheckFactMap {
    collect_check_facts_with_supplemental_call_sites(
        session,
        root,
        file_scope,
        Vec::new(),
        plan,
        playwright,
        sources,
    )
}

pub(crate) fn collect_check_facts_with_precollected_graph_facts(
    root: &Path,
    graph_files: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected_ts: crate::codebase::ts_source::facts::TsFactMap,
    precollected_routes: std::collections::BTreeMap<
        PathBuf,
        Vec<crate::playwright::integration_routes::RouteOccurrence>,
    >,
) -> CheckFactMap {
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = graph_files.clone();
        visible_files.extend(playwright.source_files().iter().cloned());
        plan.graph_context.set_visible_files(visible_files);
    }
    super::super::staged_playwright::collect_with_precollected_ts(
        root,
        Vec::new(),
        graph_files,
        true,
        plan,
        playwright,
        super::super::PrecollectedRouteFacts {
            ts: precollected_ts,
            routes: precollected_routes,
        },
    )
}

pub fn collect_check_facts_with_playwright(
    root: &Path,
    files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_check_facts_with_playwright_and_session(&session, root, files, plan, playwright)
}

#[doc(hidden)]
pub fn collect_check_facts_with_playwright_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    files: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
) -> CheckFactMap {
    let sources = super::request_sources(&files, &[], &plan, playwright.as_ref());
    collect_with_scope(
        session,
        root,
        (files, Vec::new(), false),
        plan,
        playwright,
        sources,
        Default::default(),
    )
}

pub(crate) fn collect_check_facts_with_precollected_file_facts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>),
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
    precollected: crate::codebase::ts_source::FileIdMap<super::super::CheckFileFacts>,
) -> CheckFactMap {
    collect_with_scope(
        session,
        root,
        (file_scope.0, file_scope.1, true),
        plan,
        playwright,
        sources,
        precollected,
    )
}
