use crate::codebase::test_discovery::{self, TestRunner};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::Path;

pub(super) fn selected_match(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    path: &Path,
) -> bool {
    let rel = relative_slash_path(root, path);
    runner_match(
        root,
        config,
        TestRunner::Vitest,
        &rule.tests.vitest,
        &rel,
        rule.tests
            .vitest
            .iter()
            .any(|name| config.tests.vitest.projects.contains_key(name)),
    ) || runner_match(
        root,
        config,
        TestRunner::Playwright,
        &rule.tests.playwright,
        &rel,
        rule.tests
            .playwright
            .iter()
            .any(|name| config.tests.playwright.projects.contains_key(name)),
    )
}

fn runner_match(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    names: &[String],
    rel: &str,
    allow_fallback: bool,
) -> bool {
    if names.is_empty() {
        return false;
    }
    let filters = test_discovery::named_project_filters(root, config, runner, names);
    if filters.is_empty() {
        return allow_fallback && test_discovery::fallback_runner_match(runner, rel);
    }
    filters.iter().any(|filter| filter.is_match(rel))
}
