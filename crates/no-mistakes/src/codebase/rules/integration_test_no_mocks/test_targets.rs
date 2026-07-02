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
    runner_match(root, config, TestRunner::Vitest, &rule.tests.vitest, &rel)
        || runner_match(
            root,
            config,
            TestRunner::Playwright,
            &rule.tests.playwright,
            &rel,
        )
}

fn runner_match(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    names: &[String],
    rel: &str,
) -> bool {
    if names.is_empty() {
        return false;
    }
    let filters = test_discovery::named_project_filters(root, config, runner, names);
    if filters.is_empty() {
        return test_discovery::fallback_runner_match(runner, rel);
    }
    filters.iter().any(|filter| filter.is_match(rel))
}
