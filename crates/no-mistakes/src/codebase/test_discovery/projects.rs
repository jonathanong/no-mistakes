use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::types::TestRunner;

mod policies;
#[cfg(test)]
mod tests;

use policies::apply_explicit_policy_projects;
pub(super) use policies::explicit_policy_projects;

pub(super) fn runner_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Result<Vec<ConfigProject>> {
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig.clone(), None);
    runner_projects_from_visible_with_catalog(root, config, runner, visible_paths, &catalog)
}

pub(super) fn runner_projects_from_visible_with_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> Result<Vec<ConfigProject>> {
    if runner == TestRunner::Dotnet {
        return super::dotnet_projects::dotnet_projects_from_visible(root, config, visible_paths);
    }
    if runner == TestRunner::Swift {
        return Ok(super::swift_projects::swift_projects_from_visible(
            root,
            config,
            visible_paths,
        ));
    }
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects_from_visible_with_catalog(
            root,
            runner.framework(),
            configs,
            visible_paths,
            tsconfig_catalog,
        )?;
    apply_explicit_policy_projects(root, configs, policies, &mut projects);
    Ok(projects)
}

pub(super) fn runner_projects_lossy_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> Vec<ConfigProject> {
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig.clone(), None);
    runner_projects_lossy_from_visible_with_catalog(root, config, runner, visible_paths, &catalog)
}

pub(super) fn runner_projects_lossy_from_visible_with_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    visible_paths: &[PathBuf],
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> Vec<ConfigProject> {
    if runner == TestRunner::Dotnet {
        return super::dotnet_projects::dotnet_projects_lossy_from_visible(
            root,
            config,
            visible_paths,
        );
    }
    if runner == TestRunner::Swift {
        return super::swift_projects::swift_projects_from_visible(root, config, visible_paths);
    }
    let (configs, policies) = runner_config(config, runner);
    let mut projects =
        crate::integration_tests::project_config::load_projects_from_visible_with_catalog(
            root,
            runner.framework(),
            configs,
            visible_paths,
            tsconfig_catalog,
        )
        .unwrap_or_default();
    apply_explicit_policy_projects(root, configs, policies, &mut projects);
    projects
}

pub(super) fn runner_config(
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> (Option<&StringOrList>, &BTreeMap<String, TestProjectPolicy>) {
    match runner {
        TestRunner::Dotnet => unreachable!("dotnet projects are handled before runner_config"),
        TestRunner::Playwright => (
            config.tests.playwright.configs.as_ref(),
            &config.tests.playwright.projects,
        ),
        TestRunner::Vitest => (
            config.tests.vitest.configs.as_ref(),
            &config.tests.vitest.projects,
        ),
        TestRunner::Swift => (None, &config.tests.swift.projects),
    }
}
