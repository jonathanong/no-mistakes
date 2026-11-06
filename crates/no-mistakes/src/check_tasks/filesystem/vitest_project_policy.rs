use anyhow::Result;
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    enabled: bool,
    files: &[PathBuf],
    prepared: rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    facts: Option<&no_mistakes::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    if !enabled || !config.rule_configured(rules::vitest_project_policy::RULE_ID) {
        return super::run_remaining_filesystem_rules(
            root,
            config,
            enabled,
            files,
            prepared,
            facts,
            defer_suppression,
        );
    }
    let evaluate = || {
        no_mistakes::perf_trace::rule("analysis.rules.vitest-project-policy", || {
            rules::vitest_project_policy::diagnostics_with_files_and_sources(
                root,
                config,
                files,
                &prepared.sources,
                defer_suppression,
            )
        })
    };
    let diagnostics = match prepared.session {
        Some(session) => match session.prepared_vitest_project_policy() {
            Some(diagnostics) => Some(diagnostics),
            None => session
                .capture_check_result(rules::vitest_project_policy::RULE_ID, evaluate())?
                .map(std::sync::Arc::new),
        },
        None => Some(std::sync::Arc::new(evaluate()?)),
    };
    let mut output = AnalysisDiagnostics::default();
    if let Some(diagnostics) = diagnostics {
        output.findings = diagnostics.findings.clone();
        if let Some(session) = prepared.session {
            for failure in &diagnostics.incomplete {
                session.record_check_failure(failure.clone());
            }
        }
    }
    let mut remaining = config.clone();
    super::handled::mark(&mut remaining, |id| {
        id == rules::vitest_project_policy::RULE_ID
    });
    let rest = super::run_remaining_filesystem_rules(
        root,
        &remaining,
        enabled,
        files,
        prepared,
        facts,
        defer_suppression,
    )?;
    output.findings.extend(rest.findings);
    output.advisories.extend(rest.advisories);
    output.suppressed.extend(rest.suppressed);
    output.findings.sort();
    output.findings.dedup();
    Ok(output)
}
