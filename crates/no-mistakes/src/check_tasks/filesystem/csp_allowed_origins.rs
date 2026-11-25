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
    if !enabled || !config.rule_configured(rules::CSP_ALLOWED_ORIGINS) {
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
    let mut output =
        no_mistakes::perf_trace::rule("analysis.filesystem_rules.csp-allowed-origins", || {
            rules::csp_allowed_origins::diagnostics_prepared(
                root,
                config,
                files,
                facts,
                &prepared.sources,
                prepared.session,
                defer_suppression,
            )
        })?;
    let mut remaining = config.clone();
    super::handled::mark(&mut remaining, |id| id == rules::CSP_ALLOWED_ORIGINS);
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
    output.suppressed.extend(rest.suppressed);
    output.findings.sort();
    output.findings.dedup();
    output.advisories.sort();
    output.advisories.dedup();
    Ok(output)
}
