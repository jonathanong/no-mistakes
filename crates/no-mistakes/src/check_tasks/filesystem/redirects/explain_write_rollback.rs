use super::*;
use no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics;
use std::borrow::Cow;

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    if !config.rule_configured(rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let diagnostics =
        no_mistakes::perf_trace::filesystem_rule(rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK, || {
            rules::postgres_explain_write_rollback::diagnostics_with_files_sources_and_facts(
                root,
                config,
                files,
                &prepared.sources,
                facts,
                defer_suppression,
            )
        })?;
    let mut remaining = config.clone();
    super::super::handled::mark(&mut remaining, |id| {
        id == rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK
    });
    Ok((Cow::Owned(remaining), diagnostics))
}
