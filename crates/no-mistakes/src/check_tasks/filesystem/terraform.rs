use anyhow::Result;
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::config::v2::NoMistakesConfig;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    if !config.rule_configured(rules::TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let diagnostics = no_mistakes::perf_trace::filesystem_rule(
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
    Ok((Cow::Owned(remaining), diagnostics))
}
