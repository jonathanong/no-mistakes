use crate::integration_tests::{
    config as integration_config, project_config, types::ConfigProject, types::Framework,
};
use std::path::Path;

/// Request-scoped Playwright projects parsed from visible config files once
/// for forbidden-call collection roots. Explicit `tests.playwright.projects`
/// entries merge the same way as Vitest explicit projects.
#[doc(hidden)]
pub struct PreparedPlaywrightProjectCatalog {
    config_projects: Result<Vec<ConfigProject>, String>,
    explicit_projects: Vec<ConfigProject>,
}

#[doc(hidden)]
pub fn prepare_playwright_project_catalog(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    visible_paths: &crate::codebase::ts_source::VisiblePathSnapshot,
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> PreparedPlaywrightProjectCatalog {
    let config_projects = if config_projects_required(root, config) {
        let root_visible_paths = visible_paths.paths_for(root);
        project_config::load_projects_from_visible_with_catalog(
            root,
            Framework::Playwright,
            config.tests.playwright.configs.as_ref(),
            &root_visible_paths,
            tsconfig_catalog,
        )
        .map_err(|error| format!("{error:#}"))
    } else {
        Ok(Vec::new())
    };
    PreparedPlaywrightProjectCatalog {
        config_projects,
        explicit_projects: explicit_playwright_projects(root, config),
    }
}

impl PreparedPlaywrightProjectCatalog {
    pub(crate) fn config_projects(&self) -> anyhow::Result<Vec<ConfigProject>> {
        self.config_projects.clone().map_err(anyhow::Error::msg)
    }

    pub(crate) fn matching_files(
        &self,
        root: &Path,
        names: &[String],
        files: &[std::path::PathBuf],
    ) -> anyhow::Result<Vec<std::path::PathBuf>> {
        let projects = super::runner_project_catalog::merge_explicit_projects(
            self.config_projects()?,
            self.explicit_projects.clone(),
        );
        super::runner_project_catalog::matching_catalog_files(
            root,
            &projects,
            names,
            files,
            "forbidden-calls Playwright root names an unknown project",
        )
    }
}

fn explicit_playwright_projects(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> Vec<ConfigProject> {
    config
        .tests
        .playwright
        .projects
        .iter()
        .filter_map(|(project_name, policy)| {
            integration_config::configured_project(root, project_name, policy)
        })
        .collect()
}

fn config_projects_required(root: &Path, config: &crate::config::v2::NoMistakesConfig) -> bool {
    super::runner_project_catalog::config_projects_required(
        root,
        config.tests.playwright.configs.as_ref(),
        &config.tests.playwright.projects,
    )
}
