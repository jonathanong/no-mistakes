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
    if !config.rule_configured(rules::REACT_NO_NESTED_BUTTONS_AND_LINKS) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let diagnostics =
        no_mistakes::perf_trace::filesystem_rule(rules::REACT_NO_NESTED_BUTTONS_AND_LINKS, || {
            rules::nested_controls::diagnostics_prepared(
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
    super::handled::mark(&mut remaining, |id| {
        id == rules::REACT_NO_NESTED_BUTTONS_AND_LINKS
    });
    Ok((Cow::Owned(remaining), diagnostics))
}
