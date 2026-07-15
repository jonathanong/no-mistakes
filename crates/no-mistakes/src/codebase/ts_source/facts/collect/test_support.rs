use super::super::{TsFactContext, TsFactPlan, TsFileFacts};
use std::path::Path;

pub(crate) fn collect_file_facts_with_sources(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<TsFileFacts> {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    super::collect_file_facts_with_sources_and_session(&session, path, plan, context, sources)
}
