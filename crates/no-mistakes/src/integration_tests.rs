use crate::config::v2::load_v2_config_from_visible;
use anyhow::Result;
use std::path::Path;

pub(crate) mod analysis;
mod calls;
pub(crate) mod config;
mod enforce;
pub(crate) mod project_config;
mod resolve;
pub(crate) mod runner_config;
mod standalone;
mod test_config;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_errors;
#[cfg(test)]
mod tests_resolution;
#[cfg(test)]
mod tests_review;
pub(crate) mod types;

#[doc(hidden)]
pub use runner_config::PreparedIntegrationRunnerConfigs;
pub use types::IntegrationFinding;

#[doc(hidden)]
pub fn prepare_runner_configs(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &[std::path::PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> PreparedIntegrationRunnerConfigs {
    runner_config::prepare(root, config, visible_paths, tsconfig)
}

pub fn check(root: &Path, config_path: Option<&Path>) -> Result<Vec<IntegrationFinding>> {
    standalone::check(root, config_path)
}

pub fn check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<IntegrationFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let config = load_v2_config_from_visible(root, config_path, &visible_paths)?;
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    check_with_prepared_facts(root, &config, shared, &tsconfig, &snapshot)
}

/// Shared-config entry point used by the aggregate `check` command.
#[doc(hidden)]
pub fn check_with_config_and_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<IntegrationFinding>> {
    let snapshot =
        crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, shared.files());
    let visible_paths = snapshot.paths_for(root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    check_with_prepared_facts(root, config, shared, &tsconfig, &snapshot)
}

/// Request-scoped entry point used by the aggregate `check` command.
#[doc(hidden)]
pub fn check_with_prepared_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Result<Vec<IntegrationFinding>> {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    check_with_prepared_facts_and_session(root, config, shared, tsconfig, visible_paths, &session)
}

#[doc(hidden)]
pub fn check_with_prepared_facts_and_session(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<Vec<IntegrationFinding>> {
    config::validate_config(config)?;

    let root_paths = visible_paths.paths_for(root);
    let runner_configs = runner_config::prepare(root, config, &root_paths, tsconfig);
    let prepared_runner_configs =
        runner_config::ParsedRunnerConfigs::with_files(shared.integration_runner_configs.clone());
    if !prepared_runner_configs.covers(&runner_configs) {
        anyhow::bail!(
            "prepared integration runner facts are incomplete; collect shared facts with \
             CheckFactPlan.integration_runner_configs from prepare_runner_configs()"
        );
    }
    let parsed_runner_configs = prepared_runner_configs;
    let suites = config::configured_suites_from_runner_configs(
        root,
        config,
        &runner_configs,
        &parsed_runner_configs,
    )?;
    if suites.is_empty() {
        return Ok(Vec::new());
    }

    fail_on_dropped_files(shared)?;
    let analyses = shared
        .ts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .integration
                .as_ref()
                .map(|analysis| (path.clone(), analysis.clone()))
        })
        .collect();
    check_suites(root, &suites, tsconfig, &analyses, session)
}

fn fail_on_dropped_files(shared: &crate::codebase::check_facts::CheckFactMap) -> Result<()> {
    for (file, facts) in &shared.ts {
        if let Some(error) = &facts.parse_error {
            anyhow::bail!(
                "failed to parse integration file {}: {error}",
                file.display()
            );
        }
    }
    Ok(())
}

fn check_suites(
    root: &Path,
    suites: &[types::Suite],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    analyses: &std::collections::BTreeMap<std::path::PathBuf, types::FileAnalysis>,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<Vec<IntegrationFinding>> {
    let function_index = resolve::build_function_index(analyses);
    let export_index = resolve::build_export_index(analyses);
    let visible_files = analyses.keys().cloned().collect();
    let import_resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        tsconfig,
        Some(&visible_files),
        session,
    );
    let resolver = resolve::ImportResolution {
        analyses,
        export_index: &export_index,
        resolver: &import_resolver,
    };

    let mut findings = Vec::new();
    for suite in suites {
        let include = project_config::build_globset(&suite.include)?;
        let exclude = project_config::build_globset(&suite.exclude)?;
        for (file, file_analysis) in analyses {
            let rel = crate::codebase::ts_source::relative_slash_path(root, file);
            if !include.is_match(&rel) || exclude.is_match(&rel) {
                continue;
            }
            for test in &file_analysis.tests {
                let integrations =
                    resolve::resolved_integrations(&test.function_key, &function_index, &resolver);
                findings.extend(enforce::enforce_policy(root, suite, test, &integrations));
            }
        }
    }
    sort_findings(&mut findings);
    Ok(findings)
}

fn sort_findings(findings: &mut Vec<IntegrationFinding>) {
    findings.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.framework.cmp(&b.framework))
            .then(a.suite.cmp(&b.suite))
    });
    findings.dedup_by(|a, b| {
        a.framework == b.framework
            && a.suite == b.suite
            && a.file == b.file
            && a.line == b.line
            && a.message == b.message
    });
}
