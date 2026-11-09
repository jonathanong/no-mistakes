use crate::codebase::check_facts::{
    file, file_program_with_discovery, CheckFactPlan, CheckFactVariant, CheckFileFacts,
    PlaywrightFactPlan,
};
use std::path::Path;
use std::sync::Arc;

pub(crate) fn collect_file_facts_with_session_and_sources(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    file::collect_file_facts_with_request(
        session.into(),
        root,
        path,
        plan,
        playwright,
        sources,
        retain_parse,
    )
}

pub(crate) fn collect_file_facts_from_program(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    owned_source: Option<Arc<str>>,
) -> CheckFileFacts {
    file_program_with_discovery(
        &CheckFactVariant {
            root,
            plan,
            playwright,
            collect_tests: true,
        },
        path,
        source,
        program,
        owned_source,
    )
}
