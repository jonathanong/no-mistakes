use super::*;
use no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics;
use std::borrow::Cow;

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    sources: &std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let managed = [
        rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK,
        rules::POSTGRES_LOCK_REASON,
    ];
    if !managed.iter().any(|rule| config.rule_configured(rule)) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let mut remaining = config.clone();
    let mut diagnostics = AnalysisDiagnostics::default();
    if config.rule_configured(rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK) {
        diagnostics.append(no_mistakes::perf_trace::filesystem_rule(
            rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK,
            || {
                rules::postgres_explain_write_rollback::diagnostics_with_files_sources_and_facts(
                    root,
                    config,
                    files,
                    sources,
                    facts,
                    defer_suppression,
                )
            },
        )?);
        super::handled::mark(&mut remaining, |id| {
            id == rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK
        });
    }
    if config.rule_configured(rules::POSTGRES_LOCK_REASON) {
        diagnostics.append(no_mistakes::perf_trace::filesystem_rule(
            rules::POSTGRES_LOCK_REASON,
            || {
                rules::postgres_lock_reason::diagnostics_with_files_sources_and_facts(
                    root,
                    config,
                    files,
                    sources,
                    facts,
                    defer_suppression,
                )
            },
        )?);
        super::handled::mark(&mut remaining, |id| id == rules::POSTGRES_LOCK_REASON);
    }
    Ok((Cow::Owned(remaining), diagnostics))
}
