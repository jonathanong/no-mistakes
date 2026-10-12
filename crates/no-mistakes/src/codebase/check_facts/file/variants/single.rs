//! Single-file adapter for shared standard and historical parser projections.
use super::{CheckFactVariant, CheckFileFacts};
use std::path::Path;
use std::sync::Arc;

pub(crate) fn collect(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &crate::codebase::check_facts::CheckFactPlan,
    playwright: Option<&crate::codebase::check_facts::PlaywrightFactPlan>,
    source: Arc<str>,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    let result = crate::ast::with_request_parse_cache(|| {
        super::collect_file_fact_variants_from_source_with_session(
            session,
            path,
            source,
            &[CheckFactVariant {
                root,
                plan,
                playwright,
            }],
        )
        .into_iter()
        .next()
        .flatten()
    });
    if !retain_parse {
        crate::ast::evict_request_parse_cache_path(path);
    }
    result
}
