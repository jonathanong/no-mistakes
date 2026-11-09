use super::*;

pub(crate) fn collect_file_facts_with_session_and_sources(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    collect_file_facts_with_request(
        session.into(),
        root,
        path,
        plan,
        playwright,
        sources,
        retain_parse,
    )
}
