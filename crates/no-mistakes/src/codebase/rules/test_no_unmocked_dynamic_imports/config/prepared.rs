use super::discovery::config_files_from_visible;
use super::{ConfigSetupData, TestFilter};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(in super::super) struct PreparedConfig {
    test_filter: TestFilter,
    setup_data: Vec<ConfigSetupData>,
}

impl PreparedConfig {
    pub(in super::super) fn test_filter(&self) -> &TestFilter {
        &self.test_filter
    }

    pub(in super::super) fn setup_data(&self) -> &[ConfigSetupData] {
        &self.setup_data
    }
}

pub(in super::super) fn prepare_with_runner_facts(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::RunnerConfigFacts,
) -> Result<PreparedConfig> {
    prepare_inner(root, config, visible_files, sources, Some(facts))
}

pub(super) fn prepare_inner(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&crate::codebase::check_facts::RunnerConfigFacts>,
) -> Result<PreparedConfig> {
    let config_files = config_files_from_visible(root, config, visible_files);
    let visible_files = visible_files
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let mut setup_data = super::precompute_setup_data_from_config_files_from_visible(
        root,
        &config_files,
        &visible_files,
        sources,
        facts.map(|facts| (config, facts)),
    )?;
    setup_data.extend(super::explicit_project_setup_data(
        root,
        config,
        &visible_files,
    )?);
    setup_data.extend(super::rule_targets::playwright_setup_data(root, config)?);
    Ok(PreparedConfig {
        test_filter: super::filter::test_filter_from_config_files_with_sources(
            root,
            config,
            &config_files,
            Some(sources),
        )?,
        setup_data,
    })
}
