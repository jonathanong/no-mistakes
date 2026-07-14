use crate::config::v2::schema::{NoMistakesConfig, TestProjectPolicy};
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use std::path::{Path, PathBuf};

pub(super) fn swift_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Vec<ConfigProject> {
    let mut projects = Vec::new();
    for package in &config.tests.swift.packages {
        let package_root = root.join(package);
        let package_slash = package.trim_end_matches('/');
        let package_projects =
            swift_test_targets_from_package(root, &package_root, package_slash, visible_paths);
        if package_projects.is_empty() {
            projects.push(ConfigProject {
                config: Some(package_slash.to_string()),
                policy_name: Some(package_slash.to_string()),
                runner_project_arg: None,
                scope: Some(package_slash.to_string()),
                include: vec![format!("{package_slash}/Tests/**/*.swift")],
                exclude: Vec::new(),
            });
        } else {
            projects.extend(package_projects);
        }
    }
    for (name, policy) in &config.tests.swift.projects {
        if let Some(project) =
            configured_swift_project(root, name, policy, &config.tests.swift.packages)
        {
            projects.retain(|candidate| candidate.policy_name.as_deref() != Some(name));
            projects.push(project);
        }
    }
    projects
}

fn configured_swift_project(
    root: &Path,
    project_name: &str,
    policy: &TestProjectPolicy,
    packages: &[String],
) -> Option<ConfigProject> {
    if policy.include.is_empty() {
        return None;
    }
    Some(ConfigProject {
        config: swift_package_for_policy(project_name, policy, packages),
        policy_name: Some(project_name.to_string()),
        runner_project_arg: None,
        scope: None,
        include: prefix_globs(root, root, &policy.include),
        exclude: prefix_globs(root, root, &policy.exclude),
    })
}

fn swift_package_for_policy(
    project_name: &str,
    policy: &TestProjectPolicy,
    packages: &[String],
) -> Option<String> {
    for package in packages {
        let package = package.trim_end_matches('/');
        if project_name == package || policy.include.iter().any(|glob| glob.starts_with(package)) {
            return Some(package.to_string());
        }
    }
    packages
        .first()
        .map(|package| package.trim_end_matches('/').to_string())
}

fn swift_test_targets_from_package(
    root: &Path,
    package_root: &Path,
    package: &str,
    visible_paths: &[PathBuf],
) -> Vec<ConfigProject> {
    let manifest =
        crate::codebase::ts_resolver::normalize_path(&package_root.join("Package.swift"));
    if !visible_paths
        .iter()
        .any(|path| crate::codebase::ts_resolver::normalize_path(path) == manifest)
    {
        return Vec::new();
    }
    let Ok(source) = std::fs::read_to_string(manifest) else {
        return Vec::new();
    };
    let mut projects = Vec::new();
    for target in crate::codebase::swift::extract_test_target_names(&source) {
        let include = vec![format!("{package}/Tests/{target}/**/*.swift")];
        projects.push(ConfigProject {
            config: Some(package.to_string()),
            policy_name: Some(target.clone()),
            runner_project_arg: Some(target),
            scope: Some(format!("{package}/Tests")),
            include: prefix_globs(root, root, &include),
            exclude: Vec::new(),
        });
    }
    projects
}
