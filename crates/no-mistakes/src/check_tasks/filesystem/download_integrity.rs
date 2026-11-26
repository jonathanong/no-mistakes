use anyhow::Result;
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
    if !enabled || !config.rule_configured(rules::github_actions_download_integrity::RULE_ID) {
        return super::csp_allowed_origins::run(
            root,
            config,
            enabled,
            files,
            prepared,
            facts,
            defer_suppression,
        );
    }
    let mut output = no_mistakes::perf_trace::rule(
        "analysis.filesystem_rules.github-actions-download-integrity",
        || {
            rules::github_actions_download_integrity::diagnostics_with_files_and_sources(
                root,
                config,
                files,
                &prepared.sources,
                prepared.snapshot,
                defer_suppression,
            )
        },
    )?;
    // Consume both channels once at the rule boundary. The remaining dispatch
    // must not scan downloads again or rediscover advisories at finalization.
    let mut remaining = config.clone();
    super::handled::mark(&mut remaining, |id| {
        id == rules::github_actions_download_integrity::RULE_ID
    });
    let rest = super::csp_allowed_origins::run(
        root,
        &remaining,
        enabled,
        files,
        prepared,
        facts,
        defer_suppression,
    )?;
    output.append(rest);
    output.findings.sort();
    output.findings.dedup();
    output.advisories.sort();
    output.advisories.dedup();
    Ok(output)
}
