use crate::config::v2::NoMistakesConfig;
use std::path::Path;

type RunnerTestFilter =
    crate::codebase::rules::test_no_unmocked_dynamic_imports::config::TestFilter;

#[derive(Clone)]
pub struct TestFileFilter {
    config_filter: Option<RunnerTestFilter>,
    suites: Vec<TestSuiteFilter>,
}

#[derive(Clone)]
struct TestSuiteFilter {
    runner: crate::codebase::test_discovery::TestRunner,
    filter: crate::codebase::test_discovery::ProjectTestFilter,
}

impl TestFileFilter {
    pub fn new(root: &Path, config: &NoMistakesConfig) -> Self {
        Self {
            config_filter:
                crate::codebase::rules::test_no_unmocked_dynamic_imports::config::test_filter(
                    root, config,
                )
                .ok(),
            suites: crate::codebase::test_discovery::project_filters(root, config)
                .into_iter()
                .map(|(runner, filter)| TestSuiteFilter { runner, filter })
                .collect(),
        }
    }

    pub fn is_match(&self, root: &Path, path: &Path) -> bool {
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        self.is_match_rel(&rel)
    }

    pub fn is_match_rel(&self, rel_path: &str) -> bool {
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
                excluded_by_matching_suite =
                    excluded_by_matching_suite || suite.fallback_matches_runner(rel_path);
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

    fn fallback_matches_runner(&self, rel_path: &str) -> bool {
        crate::codebase::test_discovery::fallback_runner_match(self.runner, rel_path)
    }
}

fn fallback_test_path(rel_path: &str) -> bool {
    crate::codebase::test_discovery::fallback_test_path(rel_path)
}

#[cfg(test)]
mod tests;
