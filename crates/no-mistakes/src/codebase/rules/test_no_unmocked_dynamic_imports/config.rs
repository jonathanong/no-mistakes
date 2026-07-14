mod discovery;
mod filter;
mod prepared;
mod rule_targets;

use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use discovery::{
    build_globset, build_regexes, config_files, extract_property_strings,
    extract_test_property_strings, extract_test_regexes, ConfigFile,
};
use std::path::{Path, PathBuf};

pub(crate) use filter::test_filter_from_visible;
pub use filter::{test_filter, TestFilter};
pub(super) use prepared::prepare_from_visible;

pub struct ConfigSetupData {
    filter: TestFilter,
    pub setup_files: Vec<PathBuf>,
}

impl ConfigSetupData {
    pub fn filter_matches(&self, rel_path: &str) -> bool {
        self.filter.is_match(rel_path)
    }
}

/// Pre-compute per-config filter and setup files once, so the per-test loop can skip
/// re-reading and re-parsing config files on every iteration.
pub fn precompute_setup_data(
    root: &Path,
    config: &NoMistakesConfig,
) -> Result<Vec<ConfigSetupData>> {
    precompute_setup_data_from_config_files(root, &config_files(root, config))
}

fn precompute_setup_data_from_config_files(
    root: &Path,
    config_files: &[ConfigFile],
) -> Result<Vec<ConfigSetupData>> {
    precompute_setup_data_from_config_files_inner(root, config_files, None)
}

fn precompute_setup_data_from_config_files_from_visible(
    root: &Path,
    config_files: &[ConfigFile],
    visible_files: &std::collections::HashSet<PathBuf>,
) -> Result<Vec<ConfigSetupData>> {
    precompute_setup_data_from_config_files_inner(root, config_files, Some(visible_files))
}

fn precompute_setup_data_from_config_files_inner(
    root: &Path,
    config_files: &[ConfigFile],
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Result<Vec<ConfigSetupData>> {
    let mut result = Vec::new();
    for config_file in config_files {
        let source = std::fs::read_to_string(&config_file.path)?;
        let base = config_file.path.parent().unwrap_or(root);
        let includes = normalize_matcher_patterns(root, base, config_file.includes(&source));
        let excludes = normalize_matcher_patterns(
            root,
            base,
            extract_test_property_strings(&source, "exclude"),
        );
        let filter = TestFilter {
            include: build_globset(&includes)?,
            include_regex: build_regexes(&extract_test_regexes(&source))?,
            exclude: build_globset(&excludes)?,
        };
        let setup_files =
            setup_files_from_configs_inner(root, vec![config_file.path.clone()], visible_files)?;
        result.push(ConfigSetupData {
            filter,
            setup_files,
        });
    }
    Ok(result)
}

pub fn setup_files_for_test_precomputed(
    rel_path: &str,
    config_data: &[ConfigSetupData],
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for data in config_data {
        if data.filter_matches(rel_path) {
            files.extend(data.setup_files.iter().cloned());
        }
    }
    files.sort();
    files.dedup();
    files
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
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for config_file in config_files {
        let source = std::fs::read_to_string(&config_file)?;
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
    files.sort();
    files.dedup();
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
