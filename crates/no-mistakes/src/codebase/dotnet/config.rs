use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

use super::{msbuild_path, normalize_path, DotnetConfigProject};

pub(crate) fn config_projects(
    config: &crate::config::v2::schema::DotnetConfig,
) -> Vec<DotnetConfigProject> {
    config
        .projects
        .iter()
        .filter(|(_, project)| !project.project.trim().is_empty())
        .map(|(name, project)| DotnetConfigProject {
            name: name.clone(),
            project: project.project.clone(),
            include: project.include.clone(),
            exclude: project.exclude.clone(),
            test: project.test,
        })
        .collect()
}

pub(crate) fn configured_projects(
    root: &Path,
    config: &crate::config::v2::schema::DotnetConfig,
) -> Vec<DotnetConfigProject> {
    let root = normalize_path(root);
    let mut projects = config_projects(config);
    let mut seen = projects
        .iter()
        .map(|project| normalize_path(&root.join(&project.project)))
        .collect::<HashSet<_>>();
    for project in solution_projects(&root, config) {
        let project_path = normalize_path(&root.join(&project.project));
        if seen.insert(project_path) {
            projects.push(project);
        }
    }
    projects
}

fn solution_projects(
    root: &Path,
    config: &crate::config::v2::schema::DotnetConfig,
) -> Vec<DotnetConfigProject> {
    let mut projects = Vec::new();
    for solution in &config.solutions {
        let solution_path = normalize_path(&root.join(msbuild_path(solution)));
        let solution_dir = solution_path.parent().unwrap_or(root);
        let Ok(source) = std::fs::read_to_string(&solution_path) else {
            continue;
        };
        projects.extend(parse_solution_projects(root, solution_dir, &source));
    }
    projects
}

fn parse_solution_projects(
    root: &Path,
    solution_dir: &Path,
    source: &str,
) -> Vec<DotnetConfigProject> {
    let re = Regex::new(r#"(?m)^Project\("\{[^"]+\}"\)\s*=\s*"([^"]+)",\s*"([^"]+\.csproj)""#)
        .expect("valid regex");
    re.captures_iter(source)
        .filter_map(|cap| {
            let name = cap.get(1)?.as_str().to_string();
            let project_path =
                normalize_path(&solution_dir.join(msbuild_path(cap.get(2)?.as_str())));
            Some(DotnetConfigProject {
                name,
                project: crate::codebase::ts_source::relative_slash_path(root, &project_path),
                include: Vec::new(),
                exclude: Vec::new(),
                test: false,
            })
        })
        .collect()
}
