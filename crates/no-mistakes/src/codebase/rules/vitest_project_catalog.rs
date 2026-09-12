use crate::integration_tests::{
    config as integration_config, project_config, types::ConfigProject, types::Framework,
};
use std::path::Path;

/// Request-scoped Vitest projects parsed from visible config files once for
/// aggregate filesystem-rule fanout. Explicit `tests.vitest.projects` entries
/// are merged for forbidden-call roots the same way `vitest-project-mapping`
/// synthesizes ownership.
#[doc(hidden)]
pub struct PreparedVitestProjectCatalog {
    config_projects: Result<Vec<ConfigProject>, String>,
    explicit_projects: Vec<ConfigProject>,
}

#[doc(hidden)]
#[inline(never)]
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
    PreparedVitestProjectCatalog {
        config_projects,
        explicit_projects: explicit_vitest_projects(root, config),
    }
}

impl PreparedVitestProjectCatalog {
    #[inline(never)]
    pub(crate) fn config_projects(&self) -> anyhow::Result<Vec<ConfigProject>> {
        self.config_projects.clone().map_err(anyhow::Error::msg)
    }

    #[inline(never)]
    pub(crate) fn merged_projects(&self) -> anyhow::Result<Vec<ConfigProject>> {
        Ok(merge_explicit_vitest_projects(
            self.config_projects()?,
            self.explicit_projects.clone(),
        ))
    }

    /// Reuse parsed Vitest ownership for graph-backed call policy roots.
    #[inline(never)]
    pub(crate) fn matching_files(
        &self,
        root: &Path,
        names: &[String],
        files: &[std::path::PathBuf],
    ) -> anyhow::Result<Vec<std::path::PathBuf>> {
        super::runner_project_catalog::matching_catalog_files(
            root,
            &self.merged_projects()?,
            names,
            files,
            "forbidden-calls Vitest root names an unknown project",
        )
    }
}

#[inline(never)]
pub(crate) fn explicit_vitest_projects(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> Vec<ConfigProject> {
    config
        .tests
        .vitest
        .projects
        .iter()
        .filter_map(|(project_name, policy)| {
            integration_config::configured_project(root, project_name, policy)
        })
        .collect()
}

#[inline(never)]
pub(crate) fn merge_explicit_vitest_projects(
    projects: Vec<ConfigProject>,
    explicit: Vec<ConfigProject>,
) -> Vec<ConfigProject> {
    super::runner_project_catalog::merge_explicit_projects(projects, explicit)
}

#[inline(never)]
pub(crate) fn config_projects_required(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> bool {
    super::runner_project_catalog::config_projects_required(
        root,
        config.tests.vitest.configs.as_ref(),
        &config.tests.vitest.projects,
    )
}
