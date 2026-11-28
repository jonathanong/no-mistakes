use super::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::codebase::rules::{suppress_domain_findings_with_sources, SuppressionTarget};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::config::v2::NoMistakesConfig;
use no_mistakes::queue::{format_aggregate_warning, is_check_warning, CheckFinding};

pub fn run_queue_check(
    root: &std::path::Path,
    prepared_tsconfig_catalog: &std::sync::Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    enabled: bool,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    session: &no_mistakes::codebase::analysis_session::AnalysisSession,
    sources: &SourceStore,
) -> Result<CheckTask<Vec<CheckFinding>>> {
    let (result, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.queues",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || -> Result<_> {
            Ok(if enabled {
                let report =
                    no_mistakes::queue::analyze_project_with_prepared_facts_and_catalog_and_session(
                        root,
                        prepared_tsconfig_catalog,
                        config,
                        &[],
                        facts,
                        session,
                    )?;
                let mut warnings = report
                    .diagnostics
                    .into_iter()
                    .filter(is_check_warning)
                    .collect();
                let suppressed = suppress_domain_findings_with_sources(
                    root,
                    &mut warnings,
                    sources,
                    |diagnostic| SuppressionTarget {
                        domain: "queues",
                        rule: "queues-check",
                        file: &diagnostic.file,
                        line: Some(diagnostic.line),
                        reason: &diagnostic.message,
                        identity: None,
                    },
                );
                let analysis_warnings = warnings.iter().map(format_aggregate_warning).collect();
                (report.check, analysis_warnings, suppressed)
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            })
        },
    );
    let (findings, analysis_warnings, pre_suppressed) = result?;
    Ok(CheckTask {
        findings,
        pre_suppressed,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: Vec::new(),
        analysis_warnings,
        warning: None,
        duration,
    })
}
