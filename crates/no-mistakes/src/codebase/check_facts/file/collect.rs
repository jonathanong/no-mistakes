use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::path::Path;
use std::sync::Arc;

#[cfg(test)]
pub(crate) fn collect_file_facts_with_session_and_sources(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    let context = super::prepared_ts_extract_context(session, root, plan);
    super::collect_file_facts_with_session_and_sources_and_context(
        session,
        root,
        path,
        plan,
        playwright,
        sources,
        retain_parse,
        &context,
    )
}

pub(crate) fn collect_file_facts_with_session_and_sources_and_context(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
    context: &crate::codebase::ts_source::facts::TsFactContext,
) -> Option<CheckFileFacts> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(err) => {
            let parse_error = format!("failed to read {}: {err}", path.display());
            return Some(CheckFileFacts {
                ts: Arc::new(crate::codebase::ts_source::facts::TsFileFacts {
                    operational_error: Some(parse_error.clone()),
                    parse_error: Some(parse_error.clone()),
                    ..Default::default()
                }),
                parse_error: Some(parse_error),
                server_route_client_boundary: plan
                    .server_route_client_boundary
                    .then(Default::default),
                ..CheckFileFacts::default()
            });
        }
    };
    if plan.production_import_paths.contains(path) {
        return super::production_imports::collect(
            session,
            root,
            path,
            plan,
            playwright,
            source,
            retain_parse,
            context,
        );
    }
    super::collect_file_facts_from_source(
        session,
        root,
        path,
        plan,
        playwright,
        source,
        retain_parse,
        context,
    )
}
