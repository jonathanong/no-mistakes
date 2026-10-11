use super::discovery::{ConfigFile, Runner};
use crate::config::v2::{ConfigView, NoMistakesConfig};
use crate::integration_tests::{project_config, types::Framework};
use std::path::{Path, PathBuf};

pub(super) fn config_files(
    root: &Path,
    config: &NoMistakesConfig,
    visible: &[PathBuf],
    prepared: bool,
) -> Vec<ConfigFile> {
    let selected = super::SelectedRunners::for_config(config);
    if !prepared {
        return super::discovery::config_files_from_visible(root, config, visible)
            .into_iter()
            .filter(|file| selected.includes(file.runner))
            .collect();
    }
    let mut files = Vec::new();
    if selected.includes(Runner::Jest) {
        files.extend(super::discovery::runner_config_files_from_visible(
            root,
            ConfigView::new(config).jest_configs(),
            Runner::Jest,
            visible,
        ));
    }
    if prepared && super::needs_vitest_configs(config) {
        files.extend(
            project_config::config_values_from_visible(
                root,
                Framework::Vitest,
                config.tests.vitest.configs.as_ref(),
                visible,
            )
            .into_iter()
            .map(|raw| ConfigFile {
                path: crate::codebase::ts_resolver::normalize_path(&root.join(raw)),
                runner: Runner::Vitest,
            }),
        );
    }
    files
}
