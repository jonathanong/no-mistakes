use crate::config::v2::load_v2_config_from_visible;
use anyhow::Result;
use std::path::Path;

pub(crate) mod analysis;
mod calls;
mod checks;
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
pub(crate) use test_config::vitest::setup_resolution::resolve_setup_dependencies;

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

pub(crate) use checks::sort_findings;
use checks::{check_suites, check_suites_with_resolver, fail_on_dropped_files};
pub use runner_config::configured_runner_config_dirs;
pub use runner_config::prepare_runner_configs_with_catalog;

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
    let analyses: std::collections::BTreeMap<std::path::PathBuf, types::FileAnalysis> = shared
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

/// Request-scoped aggregate entry point with per-importer TypeScript resolution.
#[doc(hidden)]
pub fn check_with_prepared_facts_catalog_and_session(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    tsconfig_catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Result<Vec<IntegrationFinding>> {
    config::validate_config(config)?;

    let root_paths = visible_paths.paths_for(root);
    let runner_configs = runner_config::prepare_with_catalog_and_sources(
        root,
        config,
        &root_paths,
        std::sync::Arc::clone(&tsconfig_catalog),
        visible_paths.source_store_for(root),
    );
    let prepared_runner_configs =
        runner_config::ParsedRunnerConfigs::with_files(shared.integration_runner_configs.clone());
    if !prepared_runner_configs.covers(&runner_configs) {
        anyhow::bail!(
            "prepared integration runner facts are incomplete; collect shared facts with \
             CheckFactPlan.integration_runner_configs from prepare_runner_configs()"
        );
    }
    let suites = config::configured_suites_from_runner_configs(
        root,
        config,
        &runner_configs,
        &prepared_runner_configs,
    )?;
    if suites.is_empty() {
        return Ok(Vec::new());
    }

    fail_on_dropped_files(shared)?;
    let analyses: std::collections::BTreeMap<std::path::PathBuf, types::FileAnalysis> = shared
        .ts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .integration
                .as_ref()
                .map(|analysis| (path.clone(), analysis.clone()))
        })
        .collect();
    let visible_files = analyses.keys().cloned().collect();
    let import_resolver = crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
        &tsconfig_catalog,
        &visible_files,
        session,
    );
    check_suites_with_resolver(root, &suites, &analyses, &import_resolver)
}
