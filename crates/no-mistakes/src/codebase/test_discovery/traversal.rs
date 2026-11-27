use super::{FrameworkPreparationPlan, NoMistakesConfig, Path, PathBuf, TestRunner};
use crate::codebase::dependencies::test_globs;
use crate::codebase::testing_analysis::{
    test_filter::TestFileFilter, FrameworkCatalog, FrameworkCatalogPreparation, PreparedProjects,
};
use crate::codebase::ts_resolver::TsConfig;
use anyhow::Result;

mod catalog;
mod filter;
mod globs;
mod projects;

pub(crate) use catalog::prepare_tsconfig_catalog_with_framework_projects;
pub(crate) use filter::{new_test_filter, test_filter_from_visible};
pub(crate) use globs::test_filters_from_prepared;

pub(crate) fn excluded_config_paths(
    plan: &FrameworkPreparationPlan,
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> std::collections::HashSet<PathBuf> {
    [TestRunner::Playwright, TestRunner::Vitest]
        .into_iter()
        .filter(|runner| !plan.contains(*runner))
        .flat_map(|runner| {
            let (configured, _) = super::projects::runner_config(config, runner);
            configured.map_or_else(
                || {
                    crate::integration_tests::project_config::discovered_config_paths(
                        root,
                        runner.framework(),
                        visible_paths,
                    )
                },
                crate::config::v2::schema::StringOrList::values,
            )
        })
        .map(|path| crate::codebase::ts_resolver::normalize_path(&root.join(path)))
        .filter(|path| !plan.retains_indexable_path(path))
        .collect()
}
