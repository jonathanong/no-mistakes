use super::*;
use no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics;
use std::borrow::Cow;

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    sources: &no_mistakes::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let managed = [
        rules::PACKAGE_JSON_REQUIRED_FIELDS,
        rules::PACKAGE_JSON_REGISTRY_ONLY,
        rules::PNPM_OVERRIDES_BAN,
        rules::PNPM_RELEASE_AGE_POLICY,
    ];
    if !managed.iter().any(|rule| config.rule_configured(rule)) {
        return Ok((Cow::Borrowed(config), AnalysisDiagnostics::default()));
    }
    let mut remaining = config.clone();
    let mut diagnostics = AnalysisDiagnostics::default();
    macro_rules! collect {
        ($module:ident) => {
            if config.rule_configured(rules::$module::RULE_ID) {
                diagnostics.append(rules::$module::diagnostics_with_files_and_sources(
                    root,
                    config,
                    files,
                    sources,
                    defer_suppression,
                )?);
                remaining
                    .rules
                    .retain(|rule| rule.rule != rules::$module::RULE_ID);
            }
        };
    }
    collect!(package_json_required_fields);
    collect!(package_json_registry_only);
    collect!(pnpm_overrides_ban);
    collect!(pnpm_release_age_policy);
    Ok((Cow::Owned(remaining), diagnostics))
}

#[cfg(test)]
mod tests;
