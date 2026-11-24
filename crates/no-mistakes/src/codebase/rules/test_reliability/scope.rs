use super::options::RetryOptions;
use super::{configured, PLAYWRIGHT_NO_SERIAL, TEST_NO_RETRIES};
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::rules::selected_rule_files_with_sources;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{is_test_file, relative_slash_path, SourceStore};
use crate::config::v2::{schema::StringOrList, NoMistakesConfig};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
pub struct SourceDemand {
    pub(crate) vitest_config: bool,
    pub(crate) playwright_config: bool,
    pub(crate) playwright_factory: bool,
    pub(crate) standard_tests: bool,
    pub(crate) playwright_tests: bool,
    pub(crate) vitest_globals: bool,
}

impl SourceDemand {
    #[doc(hidden)]
    pub fn includes_playwright_tests(self) -> bool {
        self.playwright_tests
    }

    pub(crate) fn include(&mut self, other: Self) {
        self.vitest_config |= other.vitest_config;
        self.playwright_config |= other.playwright_config;
        self.playwright_factory |= other.playwright_factory;
        self.standard_tests |= other.standard_tests;
        self.playwright_tests |= other.playwright_tests;
        self.vitest_globals |= other.vitest_globals;
    }
    pub(super) fn includes(self, origin: Origin) -> bool {
        match origin {
            Origin::Settings => true,
            Origin::PlaywrightFactory => self.playwright_factory,
            Origin::Standard => self.standard_tests,
            Origin::VitestGlobal => self.standard_tests && self.vitest_globals,
            Origin::Playwright => self.playwright_tests,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Origin {
    Settings,
    PlaywrightFactory,
    Standard,
    VitestGlobal,
    Playwright,
}

pub(crate) struct ConfigPaths {
    vitest: Option<BTreeSet<PathBuf>>,
    playwright: Option<BTreeSet<PathBuf>>,
    workflows: BTreeSet<PathBuf>,
    playwright_test_paths: super::super::RuleGlobFilter,
    playwright_custom_paths: bool,
}

impl ConfigPaths {
    pub(crate) fn new(root: &Path, config: &NoMistakesConfig) -> Result<Self> {
        let playwright = &config.tests.playwright;
        Ok(Self {
            vitest: configured_paths(root, config.tests.vitest.configs.as_ref()),
            playwright: configured_paths(root, playwright.configs.as_ref()),
            workflows: config
                .ci
                .workflow_dirs
                .iter()
                .map(|dir| normalize_path(&root.join(dir)))
                .collect(),
            playwright_test_paths: super::super::RuleGlobFilter::new(
                PLAYWRIGHT_NO_SERIAL,
                &playwright.test_include,
                &playwright.test_exclude,
            )?,
            playwright_custom_paths: !playwright.test_include.is_empty(),
        })
    }

    pub(crate) fn workflow(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
            && path
                .parent()
                .is_some_and(|parent| self.workflows.contains(parent))
    }

    pub(crate) fn source(
        &self,
        root: &Path,
        path: &Path,
        rule: &crate::config::v2::schema::RuleDef,
        factory_files: &GlobMatcher,
    ) -> Option<SourceDemand> {
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("js" | "jsx" | "ts" | "tsx" | "mjs" | "mts" | "cjs" | "cts")
        ) {
            return None;
        }
        let vitest_config = rule.rule == TEST_NO_RETRIES
            && is_config(path, self.vitest.as_ref(), &["vitest", "vite"]);
        let playwright_config = is_config(path, self.playwright.as_ref(), &["playwright"]);
        let relative = relative_slash_path(root, path);
        let playwright_factory = factory_files.is_match(&relative);
        let test_source = !vitest_config && !playwright_config;
        let default_test_name = is_test_file(&relative) || relative.starts_with("__tests__/");
        let explicit_scope = !rule.include.is_empty();
        let standard_tests =
            rule.rule == TEST_NO_RETRIES && test_source && (default_test_name || explicit_scope);
        let playwright_tests = test_source
            && (default_test_name || explicit_scope || self.playwright_custom_paths)
            && self.playwright_test_paths.includes(&relative);
        (vitest_config
            || playwright_config
            || playwright_factory
            || standard_tests
            || playwright_tests)
            .then_some(SourceDemand {
                vitest_config,
                playwright_config,
                playwright_factory,
                standard_tests,
                playwright_tests,
                vitest_globals: !rule.tests.vitest.is_empty(),
            })
    }
}

fn configured_paths(root: &Path, values: Option<&StringOrList>) -> Option<BTreeSet<PathBuf>> {
    values.map(|value| {
        value
            .values()
            .into_iter()
            .map(|path| normalize_path(&root.join(path)))
            .collect()
    })
}

fn is_config(path: &Path, configured: Option<&BTreeSet<PathBuf>>, basenames: &[&str]) -> bool {
    if let Some(configured) = configured {
        return configured.contains(path);
    }
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_suffix(".config"))
        .is_some_and(|stem| basenames.contains(&stem))
}

/// Declare test reliability facts on the ordinary request-wide TS parse plan.
#[doc(hidden)]
pub fn required_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Arc<BTreeMap<PathBuf, SourceDemand>>> {
    let mut selected = BTreeMap::<_, SourceDemand>::new();
    if !configured(config) {
        return Ok(Arc::new(selected));
    }
    let paths = ConfigPaths::new(root, config)?;
    for id in [TEST_NO_RETRIES, PLAYWRIGHT_NO_SERIAL] {
        for rule in config.rule_applications(id) {
            let options: RetryOptions = if id == TEST_NO_RETRIES {
                rule.rule_options()
            } else {
                RetryOptions::default()
            };
            let factory_files = options.factory_files()?;
            for path in selected_rule_files_with_sources(root, config, rule, files, sources)? {
                if let Some(demand) = paths.source(root, &path, rule, &factory_files) {
                    selected.entry(path).or_default().include(demand);
                }
            }
        }
    }
    Ok(Arc::new(selected))
}
