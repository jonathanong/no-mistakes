use super::super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
    let sources = super::request_sources(&files, &graph_files, &plan, playwright.as_ref());
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
    collect_with_scope(
        session,
        root,
        (file_scope.0, file_scope.1, true),
        plan,
        playwright,
        sources,
        Default::default(),
    )
}

pub(crate) fn collect_check_facts_with_precollected_graph_facts(
    root: &Path,
    graph_files: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
    precollected_ts: crate::codebase::ts_source::facts::TsFactMap,
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
        precollected_ts,
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
    precollected: std::collections::HashMap<PathBuf, super::super::CheckFileFacts>,
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

fn collect_with_scope(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<crate::codebase::ts_source::SourceStore>,
    precollected: std::collections::HashMap<PathBuf, super::super::CheckFileFacts>,
) -> CheckFactMap {
    let (files, graph_files, graph_files_complete) = file_scope;
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = files.clone();
        visible_files.extend(graph_files.iter().cloned());
        if let Some(playwright) = &playwright {
            visible_files.extend(playwright.source_files().iter().cloned());
        }
        plan.graph_context.set_visible_files(visible_files);
    }
    if let Some(playwright) = playwright {
        return super::super::staged_playwright::collect_with_sources_and_session(
            session,
            root,
            (files, graph_files, graph_files_complete),
            plan,
            playwright,
            sources,
            precollected,
        );
    }
    super::collect_check_facts_inner(
        session,
        root,
        (files, graph_files, graph_files_complete),
        plan,
        playwright,
        sources,
        precollected,
    )
}
