use super::*;
use no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics;
use std::borrow::Cow;

type DiagnosticsRunner = fn(
    &Path,
    &NoMistakesConfig,
    &[PathBuf],
    &std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
    Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    bool,
) -> Result<AnalysisDiagnostics>;

const MANAGED: &[(&str, DiagnosticsRunner)] = &[
    (
        rules::POSTGRES_EXPLAIN_WRITE_ROLLBACK,
        rules::postgres_explain_write_rollback::diagnostics_with_files_sources_and_facts,
    ),
    (
        rules::POSTGRES_LOCK_REASON,
        rules::postgres_lock_reason::diagnostics_with_files_sources_and_facts,
    ),
];

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    sources: &std::sync::Arc<no_mistakes::codebase::ts_source::SourceStore>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    if !MANAGED.iter().any(|(rule, _)| config.rule_configured(rule)) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let mut remaining = config.clone();
    let mut diagnostics = AnalysisDiagnostics::default();
    for &(rule_id, runner) in MANAGED {
        if config.rule_configured(rule_id) {
            diagnostics.append(no_mistakes::perf_trace::filesystem_rule(rule_id, || {
                runner(root, config, files, sources, facts, defer_suppression)
            })?);
            super::handled::mark(&mut remaining, |id| id == rule_id);
        }
    }
    Ok((Cow::Owned(remaining), diagnostics))
}
