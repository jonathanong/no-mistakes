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
    // Default max retains its existing dispatch without cloning configuration.
    if !enabled || !has_low_application(config)? {
        return super::download_integrity::run(
            root,
            config,
            enabled,
            files,
            prepared,
            facts,
            defer_suppression,
        );
    }
    let mut output = rules::declared_payload_compatibility::diagnostics_with_files_and_sources(
        root,
        config,
        files,
        &prepared.sources,
        defer_suppression,
    )?;
    // Keep both channels from this evaluation; the remaining dispatch must not
    // evaluate the same contracts again to discover advisories at finalization.
    let mut remaining = config.clone();
    remaining
        .rules
        .retain(|rule| rule.rule != rules::declared_payload_compatibility::RULE_ID);
    let rest = super::download_integrity::run(
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
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == rules::declared_payload_compatibility::RULE_ID)
    {
        if config.effective_rule_analysis(rule)?.effort == Effort::Low {
            return Ok(true);
        }
    }
    Ok(false)
}
