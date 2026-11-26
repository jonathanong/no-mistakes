use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::test_filter::TestFileFilter;
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::project_config::discovered_config_paths;
use crate::integration_tests::types::Framework;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub(in crate::queue) struct ConfiguredTestScope {
    filter: Option<Arc<TestFileFilter>>,
    playwright_include: Option<GlobSet>,
    playwright_exclude: Option<GlobSet>,
    include_tests: Option<GlobSet>,
}

impl ConfiguredTestScope {
    pub(in crate::queue) fn empty() -> Self {
        Self::default()
    }

    pub(in crate::queue) fn from_session(
        root: &Path,
        config: &NoMistakesConfig,
        visible_paths: &[PathBuf],
        session: &AnalysisSession,
    ) -> Self {
        if config
            .queues
            .include_tests
            .as_ref()
            .is_some_and(|include| include.is_all())
        {
            return Self::empty();
        }
        let filter = has_configured_suite_source(root, config, visible_paths)
            .then(|| session.test_file_filter_with_visible(root, config, Some(visible_paths)));
        let playwright_include = compile_globs(&config.tests.playwright.test_include);
        let playwright_exclude = compile_globs(&config.tests.playwright.test_exclude);
        let include_tests = config
            .queues
            .include_tests
            .as_ref()
            .and_then(|include| include.globset());
        Self {
            filter,
            playwright_include,
            playwright_exclude,
            include_tests,
        }
    }

    pub(in crate::queue) fn is_match(&self, root: &Path, path: &Path) -> bool {
        if self.include_tests.as_ref().is_some_and(|include| {
            include.is_match(crate::codebase::ts_source::relative_slash_path(root, path))
        }) {
            return false;
        }
        if self
            .filter
            .as_ref()
            .is_some_and(|filter| filter.is_configured_suite_match(root, path))
        {
            return true;
        }
        let Some(include) = &self.playwright_include else {
            return false;
        };
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        include.is_match(&rel)
            && self
                .playwright_exclude
                .as_ref()
                .is_none_or(|exclude| !exclude.is_match(&rel))
    }
}

fn has_configured_suite_source(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> bool {
    let tests = &config.tests;
    tests.vitest.configs.is_some()
        || !tests.vitest.projects.is_empty()
        || tests.playwright.configs.is_some()
        || !tests.playwright.projects.is_empty()
        || tests.jest.configs.is_some()
        || !tests.jest.projects.is_empty()
        || !tests.dotnet.solutions.is_empty()
        || !tests.dotnet.projects.is_empty()
        || !tests.swift.packages.is_empty()
        || !tests.swift.projects.is_empty()
        || !tests.python.packages.is_empty()
        || !tests.go.modules.is_empty()
        || !tests.rust.packages.is_empty()
        || !tests.rails.apps.is_empty()
        || !tests.php.apps.is_empty()
        || !tests.java.packages.is_empty()
        || !tests.kotlin.packages.is_empty()
        || !tests.elixir.apps.is_empty()
        || !tests.dart.packages.is_empty()
        || !discovered_config_paths(root, Framework::Vitest, visible_paths).is_empty()
        || !discovered_config_paths(root, Framework::Playwright, visible_paths).is_empty()
}

fn compile_globs(patterns: &[String]) -> Option<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut any = false;
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
            any = true;
        }
    }
    any.then(|| builder.build().ok()).flatten()
}
