use super::finite_set_plan::PreparedFactDemand;
use no_mistakes::codebase::check_facts::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use no_mistakes::codebase::ts_source::SourceStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct CollectInput<'a> {
    pub(crate) session: &'a no_mistakes::codebase::analysis_session::AnalysisSession,
    pub(crate) root: &'a Path,
    pub(crate) discovered: Vec<PathBuf>,
    pub(crate) graph_files: Vec<PathBuf>,
    pub(crate) needs_shared_facts: bool,
    pub(crate) filesystem_rules_enabled: bool,
    pub(crate) fact_demand: &'a PreparedFactDemand,
    pub(crate) plan: CheckFactPlan,
    pub(crate) playwright_fact_plan: Option<PlaywrightFactPlan>,
    pub(crate) sources: Arc<SourceStore>,
}

pub(crate) fn collect(
    input: CollectInput<'_>,
) -> ((Vec<PathBuf>, CheckFactMap), std::time::Duration) {
    let CollectInput {
        session,
        root,
        discovered,
        graph_files,
        needs_shared_facts,
        filesystem_rules_enabled,
        fact_demand,
        plan,
        playwright_fact_plan,
        sources,
    } = input;
    let result = no_mistakes::diagnostics::measure_if_enabled(
        "parse",
        no_mistakes::diagnostics::TimingKind::Serial,
        || {
            if needs_shared_facts {
                let filesystem_files = if filesystem_rules_enabled {
                    discovered.clone()
                } else {
                    Vec::new()
                };
                let fact_files = fact_demand.primary_files(discovered);
                let supplemental_call_site_files =
                    fact_demand.supplemental_call_site_files(&fact_files, &graph_files);
                let facts = collect_check_facts(
                    session,
                    root,
                    (fact_files, graph_files),
                    plan.clone(),
                    playwright_fact_plan,
                    Arc::clone(&sources),
                );
                // Keep out-of-scope finite-set call facts addressable without
                // widening the primary filesystem or graph universes.
                let supplemental = collect_check_facts(
                    session,
                    root,
                    (supplemental_call_site_files, Vec::new()),
                    CheckFactPlan {
                        graph: no_mistakes::codebase::ts_source::facts::TsFactPlan {
                            call_sites: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    Arc::clone(&sources),
                );
                (
                    filesystem_files,
                    facts.fact_view_with_supplemental(&supplemental),
                )
            } else {
                (discovered, Default::default())
            }
        },
    );
    release_extract_programs(session);
    result
}

/// Drop request-scoped OXC programs after extract so later check phases do
/// not retain the full AST set. Records `parse.files` at this boundary so
/// tests can prove domain checks do not parse again.
pub(crate) fn release_extract_programs(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
) {
    if let Some(observer) = session.observer() {
        let parse_files = observer
            .snapshot()
            .work
            .get("parse.files")
            .copied()
            .unwrap_or(0);
        observer.increment("parse.files_after_extract", parse_files);
    }
    no_mistakes::ast::clear_request_parse_cache();
}

fn collect_check_facts(
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    root: &Path,
    file_scope: (Vec<PathBuf>, Vec<PathBuf>),
    plan: CheckFactPlan,
    playwright_fact_plan: Option<PlaywrightFactPlan>,
    sources: Arc<SourceStore>,
) -> CheckFactMap {
    no_mistakes::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
        session,
        root,
        file_scope,
        plan,
        playwright_fact_plan,
        sources,
    )
}
