use super::{CheckFactVariant, CheckFileFacts};
use std::path::Path;
use std::sync::Arc;

pub(crate) fn collect_file_fact_variants_from_source_with_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    source: Arc<str>,
    variants: &[CheckFactVariant<'_>],
) -> Vec<Option<CheckFileFacts>> {
    let contexts = variants
        .iter()
        .map(|variant| {
            super::super::prepared_ts_extract_context(session, variant.root, variant.plan)
        })
        .collect::<Vec<_>>();
    let contexts = contexts.iter().collect::<Vec<_>>();
    super::collect_file_fact_variants_from_source_with_session_and_contexts(
        session, path, source, variants, &contexts,
    )
}
