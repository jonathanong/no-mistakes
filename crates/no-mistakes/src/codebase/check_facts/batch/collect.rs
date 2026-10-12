use super::contexts::PreparedContexts;
use super::{BatchCheckFactRequest, CollectedDemand, FactDemand};
use std::path::PathBuf;

pub(super) fn collect_demand(
    session: &crate::codebase::analysis_session::AnalysisSession,
    requests: &[BatchCheckFactRequest],
    contexts: &PreparedContexts,
    path: PathBuf,
    demands: Vec<FactDemand>,
) -> Option<CollectedDemand> {
    crate::invocation::check_timeout().ok().map(|()| {
        crate::ast::with_request_parse_cache(|| {
            let variants = demands
                .iter()
                .map(|demand| {
                    let request = &requests[demand.request];
                    super::super::file::CheckFactVariant {
                        root: &request.root,
                        plan: &demand.plan,
                        playwright: request.playwright.as_ref(),
                    }
                })
                .collect::<Vec<_>>();
            let prepared_contexts = demands
                .iter()
                .map(|demand| {
                    contexts[demand.request][demand.context]
                        .as_ref()
                        .expect("demand context was prepared")
                })
                .collect::<Vec<_>>();
            let facts = match session.read_source(&path) {
                Ok(source) => {
                    super::super::file::collect_file_fact_variants_from_source_with_session_and_contexts(
                        session,
                        &path,
                        source,
                        &variants,
                        &prepared_contexts,
                    )
                }
                Err(_) => super::super::file::collect_file_fact_variants_with_session(
                    session, &path, &variants,
                ),
            };
            (path, demands, facts)
        })
    })
}
