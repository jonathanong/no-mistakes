use super::discovery::{
    build_globset, build_regexes, config_files, extract_property_strings,
    extract_test_property_strings, extract_test_regexes, ConfigFile,
};
use super::rule_targets::rule_test_project_globs;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::GlobSet;
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct TestFilter {
    pub(super) include: GlobSet,
    pub(super) include_regex: Vec<Regex>,
    pub(super) exclude: GlobSet,
}

impl TestFilter {
    pub fn is_match(&self, rel_path: &str) -> bool {
        let included = self.include.is_match(rel_path)
            || self
                .include_regex
                .iter()
                .any(|regex| regex.is_match(rel_path));
        included && !self.exclude.is_match(rel_path)
    }
}

pub fn test_filter(root: &Path, config: &NoMistakesConfig) -> Result<TestFilter> {
    test_filter_from_config_files(root, config, &config_files(root, config))
}

pub(crate) fn test_filter_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_files: &[PathBuf],
) -> Result<TestFilter> {
    test_filter_from_config_files(
        root,
        config,
        &super::discovery::config_files_from_visible(root, config, visible_files),
    )
}

pub(super) fn test_filter_from_config_files(
    root: &Path,
    config: &NoMistakesConfig,
    config_files: &[ConfigFile],
) -> Result<TestFilter> {
    let (mut includes, mut excludes) = rule_test_project_globs(root, config)?;
    let has_rule_target_includes = !includes.is_empty();
    let mut include_regex = Vec::new();
    let mut config_includes = Vec::new();
    for config_file in config_files {
        let source = std::fs::read_to_string(&config_file.path)?;
        let base = config_file.path.parent().unwrap_or(root);
        config_includes.extend(super::normalize_matcher_patterns(
            root,
            base,
            extract_test_property_strings(&source, "include"),
        ));
        config_includes.extend(super::normalize_matcher_patterns(
            root,
            base,
            extract_property_strings(&source, "testMatch"),
        ));
        include_regex.extend(extract_test_regexes(&source));
        excludes.extend(super::normalize_matcher_patterns(
            root,
            base,
            extract_test_property_strings(&source, "exclude"),
        ));
    }
    if has_rule_target_includes {
        include_regex.clear();
    } else if !config_includes.is_empty() || !include_regex.is_empty() {
        includes = config_includes;
    } else {
        includes = crate::codebase::dependencies::VITEST_JEST_TEST_GLOBS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
    }
    Ok(TestFilter {
        include: build_globset(&includes)?,
        include_regex: build_regexes(&include_regex)?,
        exclude: build_globset(&excludes)?,
    })
}
