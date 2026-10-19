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
    // Main collection can admit these paths through production imports,
    // Playwright sources, or runner helpers. Keep the richer entry when it does.
    let mut facts = super::collect_with_scope(
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
    );
    let missing = supplemental_call_sites
        .into_iter()
        .filter(|path| !facts.ts.contains_key(path))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        let supplemental = super::super::collect_fact_map_with_sources(
            session,
            root,
            &missing,
            &CheckFactPlan {
                graph: crate::codebase::ts_source::facts::TsFactPlan {
                    call_sites: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            &sources,
        );
        facts.stats.files_parsed += supplemental.values().filter(|file| file.parsed).count();
        facts.stats.parse_errors += supplemental
            .values()
            .filter(|file| file.parse_error.is_some())
            .count();
        facts.ts.extend(supplemental.map_values(Arc::new));
    }
    facts
}

#[cfg(test)]
#[path = "supplemental/tests.rs"]
mod tests;
