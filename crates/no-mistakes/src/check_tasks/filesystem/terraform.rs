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
    if !enabled || !config.rule_configured(rules::TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS) {
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
    let mut output = no_mistakes::perf_trace::filesystem_rule(
        rules::TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS,
        || {
            rules::terraform_iam_no_undocumented_wildcards::diagnostics_prepared(
                root,
                config,
                files,
                &prepared.sources,
                prepared.session,
                defer_suppression,
            )
        },
    )?;
    let mut remaining = config.clone();
    super::handled::mark(&mut remaining, |id| {
        id == rules::TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS
    });
    output.append(super::redirects::run(
        root,
        &remaining,
        enabled,
        files,
        prepared,
        facts,
        defer_suppression,
    )?);
    output.findings.sort();
    output.findings.dedup();
    output.advisories.sort();
    output.advisories.dedup();
    Ok(output)
}
