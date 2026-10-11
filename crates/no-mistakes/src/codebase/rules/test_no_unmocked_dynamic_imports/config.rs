mod discovery;
mod filter;
mod prepared;
mod project_setups;
mod rule_targets;
mod runner_catalog;
mod runner_demand;
mod standalone;
mod vitest_setups;
pub(super) use standalone::prepare_standalone;

use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use discovery::{build_globset, build_regexes, extract_test_property_strings, ConfigFile};
use std::path::{Path, PathBuf};

pub(crate) use discovery::{extract_property_strings, extract_test_regexes};
pub(crate) use filter::test_filter_from_visible;
pub use filter::{test_filter, TestFilter};
pub(super) use prepared::prepare_with_runner_facts;
pub(super) use project_setups::{
    explicit_project_setup_data, setup_contexts_for_test_precomputed, SelectedRunners,
};

pub(crate) use runner_demand::{needs_playwright_configs, needs_vitest_configs};

pub struct ConfigSetupData {
    workspace: bool,
    filter: TestFilter,
    policy_filter: Option<TestFilter>,
    pub setup_files: Vec<PathBuf>,
    explicit_project: bool,
    runner: discovery::Runner,
    project_name: Option<String>,
}

impl ConfigSetupData {
    pub fn filter_matches(&self, rel_path: &str) -> bool {
        self.filter.is_match(rel_path)
            && self
                .policy_filter
                .as_ref()
                .is_none_or(|filter| filter.is_match(rel_path))
    }
}

fn precompute_setup_data_from_config_files_from_visible(
    root: &Path,
    config_files: &[ConfigFile],
    visible_files: &crate::fx::PathSet,
    sources: &crate::codebase::ts_source::SourceStore,
    runner_facts: Option<(
        &NoMistakesConfig,
        &crate::codebase::check_facts::RunnerConfigFacts,
    )>,
) -> Result<Vec<ConfigSetupData>> {
    precompute_setup_data_from_config_files_inner(
        root,
        config_files,
        Some(visible_files),
        Some(sources),
        runner_facts,
    )
}

fn precompute_setup_data_from_config_files_inner(
    root: &Path,
    config_files: &[ConfigFile],
    visible_files: Option<&crate::fx::PathSet>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    runner_facts: Option<(
        &NoMistakesConfig,
        &crate::codebase::check_facts::RunnerConfigFacts,
    )>,
) -> Result<Vec<ConfigSetupData>> {
    let mut result = Vec::new();
    for config_file in config_files {
        if config_file.runner == discovery::Runner::Vitest {
            if let Some((config, facts)) = runner_facts {
                result.extend(vitest_setups::from_facts(
                    config,
                    &config_file.path,
                    visible_files.expect("prepared Vitest configs require an inventory"),
                    facts,
                )?);
                continue;
            }
        }
        let source = match sources {
            Some(sources) => crate::codebase::rules::read_source(sources, &config_file.path)
                .ok_or(anyhow::anyhow!(
                    "failed to read {}",
                    config_file.path.display()
                ))?
                .to_string(),
            None => std::fs::read_to_string(&config_file.path)?,
        };
        let base = config_file.path.parent().unwrap_or(root);
        let includes = normalize_matcher_patterns(root, base, config_file.includes(&source));
        let excludes = normalize_matcher_patterns(
            root,
            base,
            extract_test_property_strings(&source, "exclude"),
        );
        let filter = TestFilter {
            alternatives: Vec::new(),
            include: build_globset(&includes)?,
            include_regex: build_regexes(&extract_test_regexes(&source))?,
            exclude: build_globset(&excludes)?,
        };
        let setup_files = setup_files_from_configs_inner(
            root,
            vec![config_file.path.clone()],
            visible_files,
            sources,
        );
        let setup_files = setup_files?;
        result.push(ConfigSetupData {
            workspace: false,
            filter,
            policy_filter: None,
            setup_files,
            explicit_project: false,
            runner: config_file.runner,
            project_name: None,
        });
    }
    Ok(result)
}

fn normalize_matcher_patterns(root: &Path, base: &Path, patterns: Vec<String>) -> Vec<String> {
    patterns
        .into_iter()
        .map(|pattern| normalize_matcher_pattern(root, base, pattern))
        .collect()
}

fn normalize_matcher_pattern(root: &Path, base: &Path, pattern: String) -> String {
    if pattern == "<rootDir>" {
        return crate::codebase::ts_source::relative_slash_path(root, base);
    }
    if let Some(rest) = pattern.strip_prefix("<rootDir>/") {
        return crate::codebase::ts_source::relative_slash_path(root, &base.join(rest));
    }
    if let Some(rest) = pattern.strip_prefix("./") {
        return crate::codebase::ts_source::relative_slash_path(root, &base.join(rest));
    }
    pattern
}

fn setup_files_from_configs_inner(
    root: &Path,
    config_files: Vec<PathBuf>,
    visible_files: Option<&crate::fx::PathSet>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for config_file in config_files {
        let source = match sources {
            Some(sources) => crate::codebase::rules::read_source(sources, &config_file)
                .ok_or(anyhow::anyhow!("failed to read {}", config_file.display()))?
                .to_string(),
            None => std::fs::read_to_string(&config_file)?,
        };
        let base = config_file.parent().unwrap_or(root);
        let mut setups = extract_test_property_strings(&source, "setupFiles");
        setups.extend(extract_property_strings(&source, "setupFiles"));
        setups.extend(extract_property_strings(&source, "setupFilesAfterEnv"));
        for setup in setups {
            let path = resolve_setup_file(base, &setup);
            let path = crate::codebase::ts_resolver::normalize_path(&path);
            if visible_files.map_or_else(|| path.exists(), |visible| visible.contains(&path)) {
                files.push(path);
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    files.retain(|path| seen.insert(path.clone()));
    Ok(files)
}

fn resolve_setup_file(base: &Path, setup: &str) -> PathBuf {
    if setup == "<rootDir>" {
        return base.to_path_buf();
    }
    if let Some(rest) = setup.strip_prefix("<rootDir>/") {
        return base.join(rest);
    }
    crate::config::resolve(base, Path::new(setup))
}

#[cfg(test)]
mod prepared_tests;
#[cfg(test)]
mod tests;
