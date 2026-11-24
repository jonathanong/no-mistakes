use super::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::queue::{
    format_aggregate_warning, is_worker_rate_limit_unresolved_diagnostic, CheckFinding,
};

pub(crate) fn run_queue_check(
    root: &std::path::Path,
    prepared_tsconfig_catalog: &std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    enabled: bool,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
) -> Result<CheckTask<Vec<CheckFinding>>> {
    let (result, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.queues",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || -> Result<_> {
            Ok(if enabled {
                let mut report =
                    no_mistakes::queue::analyze_project_with_prepared_facts_and_catalog_and_session(
                        root,
                        prepared_tsconfig_catalog,
                        config,
                        &[],
                        facts,
                        session,
                    )?;
                let analysis_warnings = if super::configured::queue_topology_configured(config) {
                    report
                        .diagnostics
                        .iter()
                        .filter(|diagnostic| is_worker_rate_limit_unresolved_diagnostic(diagnostic))
                        .map(format_aggregate_warning)
                        .collect()
                } else {
                    report.check.retain(CheckFinding::is_opt_in_policy);
                    Vec::new()
                };
                (report.check, analysis_warnings)
            } else {
                (Vec::new(), Vec::new())
            })
        },
    );
    let (findings, analysis_warnings) = result?;
    Ok(CheckTask {
        findings,
        pre_suppressed: Vec::new(),
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: Vec::new(),
        analysis_warnings,
        warning: None,
        duration,
    })
}
