use crate::config::v2::schema::NoMistakesConfig;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn dotnet_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Result<Vec<ConfigProject>> {
    let (projects, missing) = collect_projects(root, config, visible_paths);
    if let Some(project) = missing.first() {
        bail!(
            "configured dotnet project `{}` at `{}` could not be resolved",
            project.name,
            project.project
        );
    }
    Ok(projects)
}

pub(super) fn dotnet_projects_lossy_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Vec<ConfigProject> {
    collect_projects(root, config, visible_paths).0
}

fn collect_projects(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> (
    Vec<ConfigProject>,
    Vec<crate::codebase::dotnet::DotnetConfigProject>,
) {
    let all_files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        visible_paths,
    );
    let configured = crate::codebase::dotnet::configured_projects(root, &config.tests.dotnet);
    let facts = crate::codebase::dotnet::collect_dotnet_facts(root, &all_files, &configured);
    let mut projects = Vec::new();
    let mut missing = Vec::new();
    for configured_project in configured {
        let project_path =
            crate::codebase::ts_resolver::normalize_path(&root.join(&configured_project.project));
        let Some(project_facts) = facts.projects.get(&project_path) else {
            missing.push(configured_project);
            continue;
        };
        if !project_facts.is_test && !configured_project.test {
            continue;
        }
        projects.push(ConfigProject {
            config: Some(configured_project.project.clone()),
            policy_name: Some(configured_project.name.clone()),
            runner_project_arg: Some(project_facts.root_namespace.clone()),
            scope: project_scope(root, &project_path),
            include: project_includes(root, &facts, &configured_project, &project_path),
            exclude: prefix_globs(root, root, &configured_project.exclude),
        });
    }
    (projects, missing)
}

fn project_scope(root: &Path, project_path: &Path) -> Option<String> {
    project_path
        .parent()
        .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
}

fn project_includes(
    root: &Path,
    facts: &crate::codebase::dotnet::DotnetFactMap,
    project: &crate::codebase::dotnet::DotnetConfigProject,
    project_path: &PathBuf,
) -> Vec<String> {
    if !project.include.is_empty() {
        return prefix_globs(root, root, &project.include);
    }
    let mut files = facts
        .files
        .values()
        .filter(|file| file.project.as_ref() == Some(project_path))
        .filter(|file| file.has_xunit_tests)
        .map(|file| {
            crate::codebase::test_discovery::literal_path_glob(
                &crate::codebase::ts_source::relative_slash_path(root, &file.path),
            )
        })
        .collect::<Vec<_>>();
    if files.is_empty() {
        let project_dir = project_path.parent().unwrap_or(root);
        let rel = crate::codebase::ts_source::relative_slash_path(root, project_dir);
        files.push(format!("{rel}/**/*.cs"));
    }
    files
}
