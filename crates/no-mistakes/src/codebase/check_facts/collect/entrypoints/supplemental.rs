use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[doc(hidden)]
pub fn collect_check_facts_with_supplemental_call_sites(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>),
    supplemental_call_sites: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<SourceStore>,
) -> CheckFactMap {
    collect_with_request(
        session.into(),
        root,
        file_scope,
        supplemental_call_sites,
        plan,
        playwright,
        sources,
    )
}
#[doc(hidden)]
pub fn collect_prepared_check_facts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>),
    supplemental_call_sites: Vec<PathBuf>,
    plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<SourceStore>,
) -> CheckFactMap {
    let request = crate::codebase::check_facts::FactRequest::prepared(session, &plan);
    collect_with_request(
        request,
        root,
        file_scope,
        supplemental_call_sites,
        plan,
        playwright,
        sources,
    )
}

/// Collect call sites outside the ordinary file and graph scopes in the same
/// request fact pass. Their paths remain addressable without widening either scope.
#[doc(hidden)]
fn collect_with_request(
    session: crate::codebase::check_facts::FactRequest<'_>,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>),
    supplemental_call_sites: Vec<PathBuf>,
    mut plan: CheckFactPlan,
    playwright: Option<PlaywrightFactPlan>,
    sources: Arc<SourceStore>,
) -> CheckFactMap {
    // Overlapping richer entries must receive call facts in their first pass,
    // without requesting call summaries from unrelated primary/graph files.
    plan.require_call_site_paths(&supplemental_call_sites);
    super::collect_with_scope(
        session,
        root,
        (file_scope.0, file_scope.1, true),
        plan,
        playwright,
        Arc::clone(&sources),
        crate::codebase::check_facts::SupplementalFactInput {
            facts: FileIdMap::with_inventory(Arc::clone(sources.inventory())),
            call_sites: &supplemental_call_sites,
        },
    )
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "supplemental/tests.rs"]
mod tests;
