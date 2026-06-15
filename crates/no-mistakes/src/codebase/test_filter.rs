use crate::config::v2::NoMistakesConfig;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

type RunnerTestFilter =
    crate::codebase::rules::test_no_unmocked_dynamic_imports::config::TestFilter;

#[derive(Clone)]
pub struct TestFileFilter {
    /// Opt-in stub/mock test globs that are always treated as test files,
    /// even when a configured suite `exclude` would otherwise drop them.
    /// `None` when unconfigured, keeping the default path a no-op.
    always_include: Option<GlobSet>,
    config_filter: Option<RunnerTestFilter>,
    suites: Vec<TestSuiteFilter>,
}

#[derive(Clone)]
struct TestSuiteFilter {
    filter: crate::codebase::test_discovery::ProjectTestFilter,
}

impl TestFileFilter {
    pub fn new(root: &Path, config: &NoMistakesConfig) -> Self {
        Self {
            always_include: compile_optional_globset(&config.tests.impact.always_include_tests),
            config_filter:
                crate::codebase::rules::test_no_unmocked_dynamic_imports::config::test_filter(
                    root, config,
                )
                .ok(),
            suites: crate::codebase::test_discovery::project_filters(root, config)
                .into_iter()
                .map(|(_runner, filter)| TestSuiteFilter { filter })
                .collect(),
        }
    }

    pub fn is_match(&self, root: &Path, path: &Path) -> bool {
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        self.is_match_rel(&rel)
    }

    pub fn is_match_rel(&self, rel_path: &str) -> bool {
        // Opt-in stub/mock tests are always surfaced, bypassing suite excludes.
        if self
            .always_include
            .as_ref()
            .is_some_and(|set| set.is_match(rel_path))
        {
            return true;
        }
        if let Some(is_match) = self.configured_suite_match(rel_path) {
            return is_match;
        }
        if self
            .config_filter
            .as_ref()
            .is_some_and(|filter| filter.is_match(rel_path))
        {
            return true;
        }
        fallback_test_path(rel_path)
    }

    fn configured_suite_match(&self, rel_path: &str) -> Option<bool> {
        let mut excluded_by_matching_suite = false;
        for suite in &self.suites {
            if !suite.matches_include(rel_path) {
                continue;
            }
            if suite.matches_exclude(rel_path) {
                excluded_by_matching_suite = true;
            } else {
                return Some(true);
            }
        }
        excluded_by_matching_suite.then_some(false)
    }
}

impl TestSuiteFilter {
    fn matches_include(&self, rel_path: &str) -> bool {
        self.filter.includes(rel_path)
    }

    fn matches_exclude(&self, rel_path: &str) -> bool {
        self.filter.excludes(rel_path)
    }
}

fn fallback_test_path(rel_path: &str) -> bool {
    crate::codebase::test_discovery::fallback_test_path(rel_path)
}

/// Compile an opt-in glob list into a [`GlobSet`]. Returns `None` when the list
/// is empty (the unconfigured default) or when every pattern is malformed.
/// Malformed patterns are skipped so a single bad glob does not silently
/// disable the valid ones, and a panic is never possible.
fn compile_optional_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    let mut has_valid = false;
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
            has_valid = true;
        }
    }
    has_valid.then(|| builder.build().ok()).flatten()
}

#[cfg(test)]
mod tests;
