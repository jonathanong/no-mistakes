use super::*;
use crate::codebase::rules::test_no_unmocked_dynamic_imports::config as dynamic_config;

pub(crate) fn new_test_filter(root: &Path, config: &NoMistakesConfig) -> TestFileFilter {
    TestFileFilter::with_filters(
        dynamic_config::test_filter(root, config).ok(),
        super::super::project_filters(root, config),
    )
}

pub(crate) fn test_filter_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> TestFileFilter {
    let projects = super::super::project_filters_from_visible_paths(root, config, visible_paths);
    TestFileFilter::with_filters(
        dynamic_config::test_filter_from_visible(root, config, visible_paths).ok(),
        projects,
    )
}
