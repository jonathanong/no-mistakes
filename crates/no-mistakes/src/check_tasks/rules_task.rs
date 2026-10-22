use super::CheckTask;
use anyhow::Result;
use no_mistakes::codebase::rules::{self, RuleFinding};

pub(crate) fn run_rules_check(
    inputs: rules::PreparedRulesCheck<'_>,
    dependency_graph: Option<&no_mistakes::codebase::dependencies::graph::DepGraph>,
    sources: &no_mistakes::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let declared_plan = inputs
        .prepared_graph
        .map(|prepared| prepared.declared_plan());
    let (((findings, suppression_sources, advisories), warning), duration) =
        no_mistakes::diagnostics::measure_if_enabled(
            "analysis.rules",
            no_mistakes::diagnostics::TimingKind::Parallel,
            || match rules::run_check_with_declared_plan_and_suppression(
                inputs,
                dependency_graph,
                sources,
                defer_suppression,
                declared_plan,
            ) {
                Ok(findings) => (
                    (
                        findings.findings,
                        findings.suppression_sources,
                        findings.advisories,
                    ),
                    None,
                ),
                Err(err) => (
                    (Vec::new(), Vec::new(), Vec::new()),
                    Some(format!("warning: rules check skipped: {err:#}")),
                ),
            },
        );
    Ok(CheckTask {
        findings,
        react_suppression_targets: Vec::new(),
        suppression_sources,
        advisories,
        warning,
        duration,
    })
}
