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
    if !config.rule_configured(rules::POSTGRES_MATERIALIZED_VIEW_SHAPE) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let diagnostics =
        no_mistakes::perf_trace::filesystem_rule(rules::POSTGRES_MATERIALIZED_VIEW_SHAPE, || {
            rules::postgres_materialized_view_shape::diagnostics_with_sources_and_facts(
                root,
                config,
                files,
                &prepared.sources,
                facts,
                defer_suppression,
            )
        })?;
    let mut remaining = config.clone();
    super::handled::mark(&mut remaining, |id| {
        id == rules::POSTGRES_MATERIALIZED_VIEW_SHAPE
    });
    Ok((Cow::Owned(remaining), diagnostics))
}
