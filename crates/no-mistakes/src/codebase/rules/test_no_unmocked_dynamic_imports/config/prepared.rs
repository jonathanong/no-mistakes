use super::discovery::config_files_from_visible;
use super::{ConfigSetupData, TestFilter};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::HashSet;
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

pub(in super::super) fn prepare_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<PreparedConfig> {
    let config_files = config_files_from_visible(root, config, visible_files);
    let visible_files = visible_files
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    Ok(PreparedConfig {
        test_filter: super::filter::test_filter_from_config_files_with_sources(
            root,
            config,
            &config_files,
            Some(sources),
        )?,
        setup_data: super::precompute_setup_data_from_config_files_from_visible(
            root,
            &config_files,
            &visible_files,
            sources,
        )?,
    })
}
