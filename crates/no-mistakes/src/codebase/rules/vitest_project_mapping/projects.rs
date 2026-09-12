use super::{normalize_scope, rel_in_scope};
use crate::integration_tests::{project_config, types::ConfigProject};
use anyhow::Result;
use globset::GlobSet;

#[derive(Debug)]
pub(super) struct ProjectGlob {
    pub(super) name: String,
    pub(super) explicit: bool,
    pub(super) scope: Option<String>,
    pub(super) include: GlobSet,
    pub(super) exclude: GlobSet,
}

pub(super) fn build_project_globs(projects: &[ConfigProject]) -> Result<Vec<ProjectGlob>> {
    projects
        .iter()
        .map(|project| {
            Ok(ProjectGlob {
                name: project
                    .policy_name
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                explicit: project.config.is_none(),
                scope: project.scope.clone(),
                include: project_config::build_globset(&project.include)?,
                exclude: project_config::build_globset(&project.exclude)?,
            })
        })
        .collect()
}

pub(super) fn matching_projects(rel: &str, projects: &[ProjectGlob]) -> Vec<String> {
    let matches = projects
        .iter()
        .filter(|project| project.matches(rel))
        .collect::<Vec<_>>();
    let deepest_config_scope = matches
        .iter()
        .filter(|project| !project.explicit)
        .filter_map(|project| project.scope.as_deref())
        .map(scope_depth)
        .max();
    matches
        .into_iter()
        .filter(|project| {
            project.explicit
                || deepest_config_scope.is_none_or(|depth| {
                    project.scope.as_deref().map(scope_depth).unwrap_or(0) == depth
                })
        })
        .map(|project| project.name.clone())
        .collect()
}

impl ProjectGlob {
    fn matches(&self, rel: &str) -> bool {
        self.scope
            .as_deref()
            .is_none_or(|scope| rel_in_scope(rel, scope))
            && self.include.is_match(rel)
            && !self.exclude.is_match(rel)
    }
}

fn scope_depth(scope: &str) -> usize {
    normalize_scope(scope)
        .split('/')
        .filter(|part| !part.is_empty())
        .count()
}
