use super::Options;
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::{
    config as integration_config, project_config, types::ConfigProject, types::Framework,
};
use anyhow::Result;
use std::path::Path;

pub(super) fn vitest_projects(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
) -> Result<Vec<ConfigProject>> {
    if opts.explicit_projects_only {
        let projects = explicit_vitest_projects(root, config);
        if projects.is_empty() {
            anyhow::bail!(
                "vitest-project-mapping explicitProjectsOnly requires at least one tests.vitest.projects entry with include globs"
            );
        }
        return Ok(projects);
    }

    let mut projects = if needs_config_projects(root, config) {
        project_config::load_projects(
            root,
            Framework::Vitest,
            config.tests.vitest.configs.as_ref(),
        )?
    } else {
        Vec::new()
    };
    for project in explicit_vitest_projects(root, config) {
        projects
            .retain(|existing| existing.policy_name.as_deref() != project.policy_name.as_deref());
        projects.push(project);
    }
    Ok(projects)
}

fn needs_config_projects(root: &Path, config: &NoMistakesConfig) -> bool {
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

fn explicit_vitest_projects(root: &Path, config: &NoMistakesConfig) -> Vec<ConfigProject> {
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
