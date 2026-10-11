use crate::config::v2::schema::{NoMistakesConfig, StringOrList};
use crate::integration_tests::project_config;
use std::path::{Path, PathBuf};

/// Parent directories of literal configured runner paths.
#[doc(hidden)]
pub fn configured_runner_config_dirs(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    config_dirs(root, configured_values(config))
}

/// Resolve configured globs against the owning request inventory before
/// seeding importer-local TypeScript config roots. No discovery or reads occur.
#[doc(hidden)]
pub fn configured_runner_config_dirs_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Vec<PathBuf> {
    let visible = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect();
    let patterns = configured_values(config).collect::<Vec<_>>();
    config_dirs(
        root,
        project_config::expand_explicit_config_values(root, &patterns, &visible).into_iter(),
    )
}

fn configured_values(config: &NoMistakesConfig) -> impl Iterator<Item = String> + '_ {
    [
        config.tests.vitest.configs.as_ref(),
        config.tests.playwright.configs.as_ref(),
    ]
    .into_iter()
    .flatten()
    .flat_map(StringOrList::values)
}

fn config_dirs(root: &Path, values: impl Iterator<Item = String>) -> Vec<PathBuf> {
    values
        .filter_map(|path| {
            crate::codebase::ts_resolver::normalize_path(&root.join(path))
                .parent()
                .map(Path::to_path_buf)
        })
        .collect()
}
