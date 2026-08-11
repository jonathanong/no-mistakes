use super::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::check_facts::CheckFactMap;
use no_mistakes::react_traits;

pub(crate) fn run_react_check(
    root: &std::path::Path,
    enabled: bool,
    facts: &CheckFactMap,
    prepared: &react_traits::PreparedReactCheck,
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
                        Ok(findings) => ((findings.findings, findings.suppression_targets), None),
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
