mod nextjs;
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
        diagnostics = nextjs::run(root, config, files, &prepared, defer_suppression)?;
        remaining = config.clone();
        super::handled::mark(&mut remaining, |id| {
            id == rules::NEXTJS_REDIRECT_DESTINATIONS
        });
        &remaining
    } else {
        config
    };
    let mut shell_remaining;
    let config = if config.rule_configured(rules::SHELLCHECK_RUNNER) {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.shellcheck-runner",
            || {
                rules::shellcheck_runner::diagnostics_with_files_sources_and_snapshot(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    prepared.snapshot,
                    defer_suppression,
                )
            },
        )?);
        shell_remaining = config.clone();
        super::handled::mark(&mut shell_remaining, |id| id == rules::SHELLCHECK_RUNNER);
        &shell_remaining
    } else {
        config
    };
    let mut without_version_pins;
    let config = if config.rule_configured(rules::VERSION_PIN_CONSISTENCY) {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.version-pin-consistency",
            || {
                rules::version_pin_consistency::diagnostics_with_files_and_sources(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    defer_suppression,
                )
            },
        )?);
        without_version_pins = config.clone();
        super::handled::mark(&mut without_version_pins, |id| {
            id == rules::VERSION_PIN_CONSISTENCY
        });
        &without_version_pins
    } else {
        config
    };
    let mut without_script_expressions;
    let config = if config.rule_configured(rules::github_actions_script_expressions::RULE_ID) {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.github-actions-script-expressions",
            || {
                rules::github_actions_script_expressions::diagnostics_with_files_and_sources(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    defer_suppression,
                )
            },
        )?);
        without_script_expressions = config.clone();
        super::handled::mark(&mut without_script_expressions, |id| {
            id == rules::github_actions_script_expressions::RULE_ID
        });
        &without_script_expressions
    } else {
        config
    };
    let (config, workflow_diagnostics) =
        super::github_actions::run(root, config, files, &prepared.sources, defer_suppression)?;
    diagnostics.append(workflow_diagnostics);
    let config = config.as_ref();
    let mut without_manifest_fields;
    let config = if config.rule_configured(rules::PACKAGE_JSON_REQUIRED_FIELDS) {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.package-json-required-fields",
            || {
                rules::package_json_required_fields::diagnostics_with_files_and_sources(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    defer_suppression,
                )
            },
        )?);
        without_manifest_fields = config.clone();
        super::handled::mark(&mut without_manifest_fields, |id| {
            id == rules::PACKAGE_JSON_REQUIRED_FIELDS
        });
        &without_manifest_fields
    } else {
        config
    };
    let mut remaining_required_files;
    let config = if config
        .rules
        .iter()
        .any(|rule| rule.enabled && rules::file_diagnostics::handles(&rule.rule))
    {
        diagnostics.append(no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.file-diagnostics-combined",
            || {
                rules::file_diagnostics::diagnostics_with_files_and_sources(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    defer_suppression,
                )
            },
        )?);
        remaining_required_files = config.clone();
        super::handled::mark(
            &mut remaining_required_files,
            rules::file_diagnostics::handles,
        );
        &remaining_required_files
    } else {
        config
    };
    let (config, manifest_diagnostics) =
        super::manifests::run(root, config, files, &prepared.sources, defer_suppression)?;
    diagnostics.append(manifest_diagnostics);
    let config = config.as_ref();
    let (config, materialized_diagnostics) =
        super::materialized_views::run(root, config, files, &prepared, facts, defer_suppression)?;
    diagnostics.append(materialized_diagnostics);
    let config = config.as_ref();
    let findings = if defer_suppression {
        rules::run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression(
            root, config, files, prepared, facts,
        )?
    } else {
        rules::run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
            root, config, files, prepared, facts,
        )?
    };
    if let Some(facts) = facts {
        diagnostics.advisories.extend(
            rules::postgres_unanalyzable_schema::diagnostics(root, config, files, facts)?
                .advisories,
        );
    }
    diagnostics.findings.sort();
    diagnostics.findings.dedup();
    diagnostics.advisories.sort();
    diagnostics.advisories.dedup();
    diagnostics.findings.extend(findings);
    diagnostics.findings.sort();
    Ok(diagnostics)
}
