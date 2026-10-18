use crate::codebase::check_facts::{
    CheckFactMap, CheckFactPlan, CheckFileFacts, PlaywrightFactPlan,
};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn collect_with_scope(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>, bool),
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<SourceStore>,
    precollected: FileIdMap<CheckFileFacts>,
) -> CheckFactMap {
    session.record_work("check.fact_collections", 1);
    let (files, graph_files, graph_files_complete) = file_scope;
    let files = plan.with_production_files(files);
    if plan.graph_context.visible_files.is_none() {
        let mut visible_files = files.clone();
        visible_files.extend(graph_files.iter().cloned());
        if let Some(playwright) = &playwright {
            visible_files.extend(playwright.source_files().iter().cloned());
        }
        plan.graph_context.set_visible_files(visible_files);
    }
    if let Some(playwright) = playwright {
        return crate::codebase::check_facts::staged_playwright::collect_with_sources_and_session(
            session,
            root,
            (files, graph_files, graph_files_complete),
            plan,
            playwright,
            sources,
            precollected,
        );
    }
    super::super::collect_check_facts_inner(
        session,
        root,
        (files, graph_files, graph_files_complete),
        plan,
        playwright,
        sources,
        precollected,
    )
}
