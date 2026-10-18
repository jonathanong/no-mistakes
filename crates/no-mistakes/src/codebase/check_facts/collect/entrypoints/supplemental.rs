use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Collect call sites outside the ordinary file and graph scopes in the same
/// request fact pass. Their paths remain addressable without widening either scope.
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
    let supplemental = if supplemental_call_sites.is_empty() {
        FileIdMap::with_inventory(Arc::clone(sources.inventory()))
    } else {
        super::super::collect_fact_map_with_sources(
            session,
            root,
            &supplemental_call_sites,
            &CheckFactPlan {
                graph: crate::codebase::ts_source::facts::TsFactPlan {
                    call_sites: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            &sources,
        )
    };
    super::collect_with_scope(
        session,
        root,
        (file_scope.0, file_scope.1, true),
        plan,
        playwright,
        sources,
        supplemental,
    )
}
