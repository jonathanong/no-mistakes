use crate::config::v2::schema::NoMistakesConfig;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::filters::ProjectTestFilter;
use super::projects;
use super::types::TestRunner;

pub(super) fn runner_reserved_tests_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    files: &[PathBuf],
    prepared_projects: Option<Vec<crate::integration_tests::types::ConfigProject>>,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> BTreeSet<PathBuf> {
    if runner != TestRunner::Vitest {
        return BTreeSet::new();
    }
    let playwright_projects = prepared_projects.unwrap_or_else(|| {
        projects::runner_projects_lossy_from_visible(
            root,
            config,
            TestRunner::Playwright,
            visible_paths,
            tsconfig,
        )
    });
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
