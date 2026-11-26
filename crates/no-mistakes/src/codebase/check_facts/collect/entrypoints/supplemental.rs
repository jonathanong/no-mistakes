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
    let nested_control_paths = Arc::clone(&plan.nested_control_paths);
    let vitest_default_exclude_paths = Arc::clone(&plan.vitest_default_exclude_paths);
    let test_reliability_paths = Arc::clone(&plan.test_reliability_paths);
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
    // Every earlier inserter retains scoped demand, so a present key already
    // carries the requested call facts; only absent paths need the light pass.
    let missing: std::collections::BTreeSet<_> = supplemental_call_sites
        .iter()
        .chain(nested_control_paths.iter())
        .chain(vitest_default_exclude_paths.iter())
        .chain(test_reliability_paths.keys())
        .filter(|path| !facts.ts.contains_key(path))
        .cloned()
        .collect();
    if !missing.is_empty() {
        let mut plan = CheckFactPlan {
            nested_control_paths,
            vitest_default_exclude_paths,
            test_reliability_paths,
            ..Default::default()
        };
        plan.require_call_site_paths(&supplemental_call_sites);
        let supplemental = super::super::collect_fact_map_with_sources(
            session,
            root,
            &missing.into_iter().collect::<Vec<_>>(),
            &plan,
            None,
            &sources,
        );
        facts.stats.files_discovered += supplemental.len();
        facts.stats.files_parsed += supplemental.values().filter(|file| file.parsed).count();
        facts.stats.parse_errors += supplemental
            .values()
            .filter(|file| file.parse_error.is_some())
            .count();
        facts.ts.extend(supplemental.map_values(Arc::new));
    }
    facts
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "supplemental/tests.rs"]
mod tests;
