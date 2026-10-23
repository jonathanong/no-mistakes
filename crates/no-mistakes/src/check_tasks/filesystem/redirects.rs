use super::*;
use no_mistakes::codebase::rules::analysis_policy::AnalysisDiagnostics;

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    if !enabled {
        return Ok(AnalysisDiagnostics::default());
    }
    let mut diagnostics = AnalysisDiagnostics::default();
    let mut remaining;
    let config = if config.rule_configured(rules::NEXTJS_REDIRECT_DESTINATIONS) {
        // Consume both channels in the rule pass; finalization must not scan configs again.
        diagnostics = if let Some(resolver) = prepared.next_config_resolver {
            rules::nextjs_redirect_destinations::diagnostics_prepared(
                root,
                config,
                files,
                &prepared.sources,
                Some(prepared.snapshot),
                resolver,
                defer_suppression,
            )?
        } else {
            // Legacy standalone callers own missing preparation at this boundary.
            rules::nextjs_redirect_destinations::diagnostics_with_files_and_sources(
                root,
                config,
                files,
                &prepared.sources,
                Some(prepared.snapshot),
                defer_suppression,
            )?
        };
        remaining = config.clone();
        remaining
            .rules
            .retain(|rule| rule.rule != rules::NEXTJS_REDIRECT_DESTINATIONS);
        &remaining
    } else {
        config
    };
    let mut shell_remaining;
    let config = if config.rule_configured(rules::SHELLCHECK_RUNNER) {
        diagnostics.append(
            rules::shellcheck_runner::diagnostics_with_files_sources_and_snapshot(
                root,
                config,
                files,
                &prepared.sources,
                prepared.snapshot,
                defer_suppression,
            )?,
        );
        shell_remaining = config.clone();
        shell_remaining
            .rules
            .retain(|rule| rule.rule != rules::SHELLCHECK_RUNNER);
        &shell_remaining
    } else {
        config
    };
    let mut without_version_pins;
    let config = if config.rule_configured(rules::VERSION_PIN_CONSISTENCY) {
        diagnostics.append(
            rules::version_pin_consistency::diagnostics_with_files_and_sources(
                root,
                config,
                files,
                &prepared.sources,
                defer_suppression,
            )?,
        );
        without_version_pins = config.clone();
        without_version_pins
            .rules
            .retain(|rule| rule.rule != rules::VERSION_PIN_CONSISTENCY);
        &without_version_pins
    } else {
        config
    };
    let findings = if defer_suppression {
        rules::run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression(
            root, config, files, prepared, facts,
        )?
    } else {
        rules::run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
            root, config, files, prepared, facts,
        )?
    };
    diagnostics.findings.sort();
    diagnostics.findings.dedup();
    diagnostics.advisories.sort();
    diagnostics.advisories.dedup();
    diagnostics.findings.extend(findings);
    diagnostics.findings.sort();
    Ok(diagnostics)
}
