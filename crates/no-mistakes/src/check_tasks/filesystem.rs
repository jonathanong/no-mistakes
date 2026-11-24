use super::{rule_configured, CheckTask};
use anyhow::Result;
use no_mistakes::codebase::rules::{self, RuleFinding};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

mod ids;
pub(super) use ids::FILESYSTEM_RULE_IDS;

pub(crate) fn run_filesystem_rules_check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<CheckTask<Vec<RuleFinding>>> {
    let (findings, duration) = no_mistakes::diagnostics::measure_if_enabled(
        "analysis.filesystem_rules",
        no_mistakes::diagnostics::TimingKind::Parallel,
        || {
            vitest_project_policy::run(
                root,
                config,
                enabled,
                files,
                prepared,
                facts,
                defer_suppression,
            )
        },
    );
    let diagnostics = findings?;
    let diagnostics = if rules::ADVISORY_RULE_IDS
        .iter()
        .any(|rule| rule_configured(config, rule))
    {
        rules::advisory_rules::partition_findings(diagnostics)
    } else {
        diagnostics
    };
    Ok(CheckTask {
        findings: diagnostics.findings,
        pre_suppressed: diagnostics.suppressed,
        react_suppression_targets: Vec::new(),
        suppression_sources: Vec::new(),
        advisories: diagnostics.advisories,
        analysis_warnings: Vec::new(),
        warning: None,
        duration,
    })
}
mod download_integrity;
mod finite_sets;
mod github_actions;
mod handled;
mod manifests;
mod partial;
mod payload_compatibility;
mod redirects;
mod vitest_project_policy;
fn run_remaining_filesystem_rules(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics> {
    if enabled
        && prepared.session.is_some_and(|session| {
            facts.is_some_and(|facts| facts.stats.parse_errors > 0)
                || !session.check_failures().is_empty()
        })
    {
        return partial::run(root, config, files, prepared, facts, defer_suppression);
    }
    finite_sets::run(
        root,
        config,
        enabled,
        files,
        prepared,
        facts,
        defer_suppression,
    )
}
pub(crate) fn filesystem_rules_configured(config: &NoMistakesConfig) -> bool {
    FILESYSTEM_RULE_IDS
        .iter()
        .any(|rule_id| rule_configured(config, rule_id))
}
