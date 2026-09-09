use super::Options;
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::{project_config, types::ConfigProject, types::Framework};
use anyhow::Result;
use std::path::Path;

pub(super) fn vitest_projects(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    catalog: Option<&super::super::PreparedVitestProjectCatalog>,
) -> Result<Vec<ConfigProject>> {
    if opts.explicit_projects_only {
        let projects = super::super::vitest_project_catalog::explicit_vitest_projects(root, config);
        if projects.is_empty() {
            anyhow::bail!(
                "vitest-project-mapping explicitProjectsOnly requires at least one tests.vitest.projects entry with include globs"
            );
        }
        return Ok(projects);
    }

    if let Some(catalog) = catalog {
        return catalog.merged_projects();
    }
    let projects = if super::super::vitest_project_catalog::config_projects_required(root, config) {
        project_config::load_projects(
            root,
            Framework::Vitest,
            config.tests.vitest.configs.as_ref(),
        )?
    } else {
        Vec::new()
    };
    Ok(
        super::super::vitest_project_catalog::merge_explicit_vitest_projects(
            projects,
            super::super::vitest_project_catalog::explicit_vitest_projects(root, config),
        ),
    )
}
