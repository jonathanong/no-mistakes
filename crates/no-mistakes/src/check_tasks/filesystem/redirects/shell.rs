use super::*;
use std::borrow::Cow;

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let mut remaining = Cow::Borrowed(config);
    let mut diagnostics = AnalysisDiagnostics::default();
    if remaining.rule_configured(rules::SHELLCHECK_RUNNER) {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.shellcheck-runner",
            || {
                rules::shellcheck_runner::diagnostics_with_files_sources_and_snapshot(
                    root,
                    &remaining,
                    files,
                    &prepared.sources,
                    prepared.snapshot,
                    defer_suppression,
                )
            },
        )?);
        super::super::handled::mark(remaining.to_mut(), |id| id == rules::SHELLCHECK_RUNNER);
    }
    if remaining.rule_configured(rules::BASH_EMPTY_ARRAY_NOUNSET) {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.bash-empty-array-nounset",
            || {
                rules::shell_compatibility::bash_diagnostics_with_files_and_sources(
                    root,
                    &remaining,
                    files,
                    &prepared.sources,
                    defer_suppression,
                )
            },
        )?);
        super::super::handled::mark(remaining.to_mut(), |id| {
            id == rules::BASH_EMPTY_ARRAY_NOUNSET
        });
    }
    Ok((remaining, diagnostics))
}
