use super::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::react_traits;

pub(crate) fn run_react_check(
    root: &std::path::Path,
    enabled: bool,
    facts: &CheckFactMap,
    prepared: &react_traits::PreparedReactCheck,
    sources: &no_mistakes::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<CheckTask<Vec<react_traits::Violation>>> {
    let (((findings, react_suppression_targets), warning), duration) =
        no_mistakes::diagnostics::measure_if_enabled(
            "analysis.react",
            no_mistakes::diagnostics::TimingKind::Parallel,
            || {
                if enabled {
                    match react_traits::run_check_with_prepared_facts_for_aggregate(
                        root,
                        &[],
                        facts,
                        prepared,
                    ) {
                        Ok(mut findings) => {
                            if !defer_suppression {
                                // Ordinary checks suppress each component using
                                // the full local/inherited target sidecar.
                                crate::check_runner::results::suppression::suppress_react(
                                    root,
                                    sources,
                                    &mut findings.findings,
                                    &findings.suppression_targets,
                                    &mut Vec::new(),
                                );
                            }
                            ((findings.findings, findings.suppression_targets), None)
                        }
                        Err(err) => (
                            (Vec::new(), Vec::new()),
                            Some(format!("warning: react check skipped: {err:#}")),
                        ),
                    }
                } else {
                    ((Vec::new(), Vec::new()), None)
                }
            },
        );
    Ok(CheckTask {
        findings,
        react_suppression_targets,
        suppression_sources: Vec::new(),
        warning,
        duration,
    })
}
