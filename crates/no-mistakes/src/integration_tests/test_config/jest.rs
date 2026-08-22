use super::super::types::ConfigProject;
use crate::codebase::dependencies::VITEST_JEST_TEST_GLOBS;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::config::{
    extract_property_strings, extract_test_regexes,
};
use crate::codebase::ts_source::relative_slash_path;
use crate::integration_tests::project_config::prefix_globs;
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub(in crate::integration_tests) fn config_project(
    root: &Path,
    raw: &str,
    config_dir: &Path,
    source: &str,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<ConfigProject> {
    let scope = relative_slash_path(root, config_dir);
    Ok(ConfigProject {
        config: Some(raw.to_string()),
        workspace: false,
        policy_name: None,
        runner_project_arg: None,
        scope: (!scope.is_empty() && scope != ".").then_some(scope),
        include: include_globs(root, config_dir, source, visible_files)?,
        exclude: Vec::new(),
        vitest_setup: Vec::new(),
    })
}

fn include_globs(
    root: &Path,
    config_dir: &Path,
    source: &str,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<Vec<String>> {
    let test_match = extract_property_strings(source, "testMatch");
    let test_regex = extract_test_regexes(source);
    let configured = !test_match.is_empty() || !test_regex.is_empty();
    let mut include = prefix_globs(root, config_dir, &normalize_matcher_patterns(test_match));
    include.extend(regex_matched_files(root, &test_regex, visible_files)?);
    if include.is_empty() && !configured {
        include = prefix_globs(
            root,
            config_dir,
            &VITEST_JEST_TEST_GLOBS
                .iter()
                .map(|pattern| (*pattern).to_string())
                .collect::<Vec<_>>(),
        );
    }
    Ok(include)
}

fn regex_matched_files(
    root: &Path,
    patterns: &[String],
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<Vec<String>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let regexes = compile_regexes(patterns)?;
    let Some(visible_files) = visible_files else {
        return Ok(Vec::new());
    };
    let mut matched = visible_files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .filter(|rel| regexes.iter().any(|regex| regex.is_match(rel)))
        .collect::<Vec<_>>();
    matched.sort();
    matched.dedup();
    Ok(matched)
}

fn compile_regexes(patterns: &[String]) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| {
            Regex::new(pattern)
                .map_err(|error| anyhow::anyhow!("invalid Jest testRegex `{pattern}`: {error}"))
        })
        .collect()
}

fn normalize_matcher_patterns(patterns: Vec<String>) -> Vec<String> {
    patterns
        .into_iter()
        .map(normalize_matcher_pattern)
        .collect()
}

fn normalize_matcher_pattern(pattern: String) -> String {
    if pattern == "<rootDir>" {
        return ".".to_string();
    }
    if let Some(rest) = pattern.strip_prefix("<rootDir>/") {
        return rest.to_string();
    }
    if let Some(rest) = pattern.strip_prefix("./") {
        return rest.to_string();
    }
    pattern
}
