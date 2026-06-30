use crate::config::v2::schema::NoMistakesConfig;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use std::path::{Path, PathBuf};

pub(super) fn dotnet_projects(root: &Path, config: &NoMistakesConfig) -> Vec<ConfigProject> {
    let all_files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    let configured = crate::codebase::dotnet::configured_projects(root, &config.tests.dotnet);
    let facts = crate::codebase::dotnet::collect_dotnet_facts(root, &all_files, &configured);
    let mut projects = Vec::new();
    for configured_project in configured {
        let project_path =
            crate::codebase::ts_resolver::normalize_path(&root.join(&configured_project.project));
        let Some(project_facts) = facts.projects.get(&project_path) else {
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
    projects
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
