use crate::config::v2::schema::{NoMistakesConfig, TestProjectPolicy};
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use std::path::{Path, PathBuf};

pub(super) fn swift_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
) -> Vec<ConfigProject> {
    if visible_paths.is_empty() || config.tests.swift.packages.is_empty() {
        return swift_projects_from_facts(root, config, &Default::default());
    }
    let all_files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        visible_paths,
    );
    let facts =
        crate::codebase::swift::collect_swift_facts(root, &all_files, &config.tests.swift.packages);
    swift_projects_from_facts(root, config, &facts)
}

pub(super) fn swift_projects_from_facts(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &crate::codebase::swift::SwiftFactMap,
) -> Vec<ConfigProject> {
    let mut projects = Vec::new();
    for package in &config.tests.swift.packages {
        let package_slash = package.trim_end_matches('/');
        let package_root = crate::codebase::ts_resolver::normalize_path(&root.join(package_slash));
        let package_projects = facts
            .packages
            .iter()
            .find(|candidate| {
                crate::codebase::ts_resolver::normalize_path(&candidate.package_root)
                    == package_root
            })
            .map(|facts| {
                facts
                    .targets
                    .values()
                    .filter(|target| target.is_test)
                    .map(|target| ConfigProject {
                        config: Some(package_slash.to_string()),
                        policy_name: Some(target.name.clone()),
                        runner_project_arg: Some(target.name.clone()),
                        scope: Some(format!("{package_slash}/Tests")),
                        include: prefix_globs(
                            root,
                            root,
                            &[format!("{package_slash}/Tests/{}/**/*.swift", target.name)],
                        ),
                        exclude: Vec::new(),
                        vitest_setup: Vec::new(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if package_projects.is_empty() {
            projects.push(ConfigProject {
                config: Some(package_slash.to_string()),
                policy_name: Some(package_slash.to_string()),
                runner_project_arg: None,
                scope: Some(package_slash.to_string()),
                include: vec![format!("{package_slash}/Tests/**/*.swift")],
                exclude: Vec::new(),
                vitest_setup: Vec::new(),
            });
        } else {
            projects.extend(package_projects);
        }
    }
    apply_configured_projects(root, config, &mut projects);
    projects
}

fn apply_configured_projects(
    root: &Path,
    config: &NoMistakesConfig,
    projects: &mut Vec<ConfigProject>,
) {
    for (name, policy) in &config.tests.swift.projects {
        if let Some(project) =
            configured_swift_project(root, name, policy, &config.tests.swift.packages)
        {
            projects.retain(|candidate| candidate.policy_name.as_deref() != Some(name));
            projects.push(project);
        }
    }
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
        vitest_setup: Vec::new(),
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
