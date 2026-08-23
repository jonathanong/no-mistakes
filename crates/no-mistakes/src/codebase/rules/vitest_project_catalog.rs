use crate::integration_tests::{project_config, types::ConfigProject, types::Framework};
use std::path::Path;

/// Request-scoped Vitest projects parsed from visible config files once for
/// aggregate filesystem-rule fanout.
#[doc(hidden)]
pub struct PreparedVitestProjectCatalog {
    config_projects: Result<Vec<ConfigProject>, String>,
}

#[doc(hidden)]
pub fn prepare_vitest_project_catalog(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> PreparedVitestProjectCatalog {
    let config_projects = if config_projects_required(root, config) {
        let root_visible_paths = visible_paths.paths_for(root);
        project_config::load_projects_from_visible_with_catalog(
            root,
            Framework::Vitest,
            config.tests.vitest.configs.as_ref(),
            &root_visible_paths,
            tsconfig_catalog,
        )
        .map_err(|error| format!("{error:#}"))
    } else {
        Ok(Vec::new())
    };
    PreparedVitestProjectCatalog { config_projects }
}

impl PreparedVitestProjectCatalog {
    pub(crate) fn config_projects(&self) -> anyhow::Result<Vec<ConfigProject>> {
        self.config_projects.clone().map_err(anyhow::Error::msg)
    }
}

pub(crate) fn config_projects_required(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> bool {
    config.tests.vitest.configs.is_none()
        || config.tests.vitest.projects.is_empty()
        || config
            .tests
            .vitest
            .configs
            .as_ref()
            .is_some_and(|configs| configs.values().iter().any(|raw| root.join(raw).exists()))
        || config
            .tests
            .vitest
            .projects
            .values()
            .any(|policy| policy.include.is_empty())
}
