use super::discovery::config_files_from_visible;
use super::filter::test_filter_from_config_files;
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
) -> Result<PreparedConfig> {
    let config_files = config_files_from_visible(root, config, visible_files);
    let visible_files = visible_files
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    Ok(PreparedConfig {
        test_filter: test_filter_from_config_files(root, config, &config_files)?,
        setup_data: super::precompute_setup_data_from_config_files_from_visible(
            root,
            &config_files,
            &visible_files,
        )?,
    })
}
