use crate::config::v2::schema::NoMistakesConfig;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::filters::ProjectTestFilter;
use super::projects;
use super::types::TestRunner;

pub(super) fn runner_reserved_tests(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    files: &[PathBuf],
) -> BTreeSet<PathBuf> {
    if runner != TestRunner::Vitest {
        return BTreeSet::new();
    }
    let playwright_projects = projects::runner_projects_lossy(root, config, TestRunner::Playwright);
    if playwright_projects.is_empty() {
        return BTreeSet::new();
    }
    let playwright_filters = playwright_projects
        .into_iter()
        .filter_map(ProjectTestFilter::from_project)
        .collect::<Vec<_>>();
    files
        .iter()
        .filter(|path| {
            let rel = crate::codebase::ts_source::relative_slash_path(root, path);
            playwright_filters
                .iter()
                .any(|filter| filter.is_match(&rel))
        })
        .cloned()
        .collect()
}
