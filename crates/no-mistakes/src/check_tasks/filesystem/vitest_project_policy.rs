use anyhow::{Context, Result};
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    if !enabled || !config.rule_configured(rules::vitest_project_policy::RULE_ID) {
        return super::run_remaining_filesystem_rules(
            root,
            config,
            enabled,
            files,
            prepared,
            facts,
            defer_suppression,
        );
    }
    let diagnostics = prepared
        .session
        .and_then(|session| session.prepared_vitest_project_policy())
        .context(
            "vitest-project-policy requires prepared policy diagnostics from the request session",
        )?;
    let mut output = AnalysisDiagnostics {
        findings: diagnostics.findings.clone(),
        ..Default::default()
    };
    let mut remaining = config.clone();
    super::handled::mark(&mut remaining, |id| {
        id == rules::vitest_project_policy::RULE_ID
    });
    let rest = super::run_remaining_filesystem_rules(
        root,
        &remaining,
        enabled,
        files,
        prepared,
        facts,
        defer_suppression,
    )?;
    output.findings.extend(rest.findings);
    output.advisories.extend(rest.advisories);
    output.suppressed.extend(rest.suppressed);
    output.findings.sort();
    output.findings.dedup();
    Ok(output)
}

#[cfg(test)]
mod tests;
