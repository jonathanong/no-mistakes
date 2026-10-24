use anyhow::Result;
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::config::v2::{Effort, NoMistakesConfig};
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
    // Preserve the existing default-max dispatch without a config clone.
    if !enabled || !has_low_application(config)? {
        return super::redirects::run(
            root,
            config,
            enabled,
            files,
            prepared,
            facts,
            defer_suppression,
        );
    }
    let mut output = rules::github_actions_download_integrity::diagnostics_with_files_and_sources(
        root,
        config,
        files,
        &prepared.sources,
        defer_suppression,
    )?;
    // Consume both channels once at the rule boundary. The remaining dispatch
    // must not scan downloads again or rediscover advisories at finalization.
    let mut remaining = config.clone();
    remaining
        .rules
        .retain(|rule| rule.rule != rules::github_actions_download_integrity::RULE_ID);
    let rest = super::redirects::run(
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
    output.findings.sort();
    output.findings.dedup();
    output.advisories.sort();
    output.advisories.dedup();
    Ok(output)
}

fn has_low_application(config: &NoMistakesConfig) -> Result<bool> {
    for rule in config.rules.iter().filter(|rule| {
        rule.enabled && rule.rule == rules::github_actions_download_integrity::RULE_ID
    }) {
        if config.effective_rule_analysis(rule)?.effort == Effort::Low {
            return Ok(true);
        }
    }
    Ok(false)
}
