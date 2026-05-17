mod discovery;

use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use discovery::{
    build_globset, build_regexes, config_files, extract_property_strings,
    extract_test_property_strings, extract_test_regexes,
};
use globset::GlobSet;
use regex::Regex;
use std::path::{Path, PathBuf};

pub struct TestFilter {
    include: GlobSet,
    include_regex: Vec<Regex>,
    exclude: GlobSet,
}

impl TestFilter {
    pub fn is_match(&self, rel_path: String) -> bool {
        let mut included = self.include.is_match(&rel_path);
        if !included {
            for regex in &self.include_regex {
                if regex.is_match(&rel_path) {
                    included = true;
                    break;
                }
            }
        }
        included && !self.exclude.is_match(&rel_path)
    }
}

pub fn test_filter(root: &Path, config: &NoMistakesConfig) -> Result<TestFilter> {
    let mut includes = project_rule_includes(config);
    if includes.is_empty() {
        includes = crate::codebase::dependencies::VITEST_JEST_TEST_GLOBS
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>();
    }
    let mut excludes = Vec::new();
    let mut include_regex = Vec::new();
    for config_file in config_files(root, config) {
        let source = std::fs::read_to_string(&config_file.path)?;
        includes.extend(extract_test_property_strings(&source, "include"));
        includes.extend(extract_property_strings(&source, "testMatch"));
        include_regex.extend(extract_test_regexes(&source));
        excludes.extend(extract_test_property_strings(&source, "exclude"));
    }
    Ok(TestFilter {
        include: build_globset(&includes)?,
        include_regex: build_regexes(&include_regex)?,
        exclude: build_globset(&excludes)?,
    })
}

fn project_rule_includes(config: &NoMistakesConfig) -> Vec<String> {
    let mut includes = Vec::new();
    for project in config.projects.values() {
        if !project.rules.iter().any(|rule| rule == super::RULE_ID) {
            continue;
        }
        let root = project.root.as_deref().unwrap_or(".").trim_matches('/');
        for include in &project.include {
            if root.is_empty() || root == "." {
                includes.push(include.to_string());
            } else {
                includes.push(format!(
                    "{}/{}",
                    root.trim_start_matches("./"),
                    include.trim_start_matches("./")
                ));
            }
        }
    }
    includes
}

#[cfg(test)]
pub fn setup_files(root: &Path, config: &NoMistakesConfig) -> Result<Vec<PathBuf>> {
    let config_files = config_files(root, config)
        .into_iter()
        .map(|config| config.path)
        .collect::<Vec<_>>();
    setup_files_from_configs(root, config_files)
}

pub fn setup_files_for_test(
    root: &Path,
    config: &NoMistakesConfig,
    rel_path: String,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for config_file in config_files(root, config) {
        let source = std::fs::read_to_string(&config_file.path)?;
        let includes = config_file.includes(&source);
        let excludes = extract_test_property_strings(&source, "exclude");
        let filter = TestFilter {
            include: build_globset(&includes)?,
            include_regex: build_regexes(&extract_test_regexes(&source))?,
            exclude: build_globset(&excludes)?,
        };
        if filter.is_match(rel_path.clone()) {
            files.extend(setup_files_from_configs(root, vec![config_file.path])?);
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn setup_files_from_configs(root: &Path, config_files: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for config_file in config_files {
        let source = std::fs::read_to_string(&config_file)?;
        let base = config_file.parent().unwrap_or(root);
        let mut setups = extract_test_property_strings(&source, "setupFiles");
        setups.extend(extract_property_strings(&source, "setupFiles"));
        setups.extend(extract_property_strings(&source, "setupFilesAfterEnv"));
        for setup in setups {
            let path = crate::config::resolve(base, Path::new(&setup));
            if path.exists() {
                files.push(crate::codebase::ts_resolver::normalize_path(&path));
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests;
