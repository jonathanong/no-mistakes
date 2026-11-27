use crate::paths::relative_slash_path;
use anyhow::Result;
use no_mistakes_ts_facts::path_norm::normalize_path;
use std::path::{Path, PathBuf};

mod glob_matcher;
pub use glob_matcher::GlobMatcher;

pub struct RuleScope<'a> {
    pub id: &'a str,
    pub include: &'a [String],
    pub exclude: &'a [String],
    pub repository: bool,
    pub allow_external_projects: bool,
}

pub struct ProjectScope<'a> {
    pub name: &'a str,
    pub root: PathBuf,
    pub include: &'a [String],
    pub exclude: &'a [String],
}

pub struct RulePathFilter {
    root: PathBuf,
    repository: bool,
    allow_external_projects: bool,
    projects: Vec<ProjectPathFilter>,
    include: GlobMatcher,
    exclude: GlobMatcher,
}

struct ProjectPathFilter {
    root: PathBuf,
    include: GlobMatcher,
    exclude: GlobMatcher,
}

impl RulePathFilter {
    pub fn new<'a>(
        root: &Path,
        scope: RuleScope<'_>,
        projects: impl IntoIterator<Item = ProjectScope<'a>>,
    ) -> Result<Self> {
        let root = normalize_path(root);
        let rule = scope.id;
        let include = GlobMatcher::new(scope.include, &format!("rule `{rule}` include"))?;
        let exclude = GlobMatcher::new(scope.exclude, &format!("rule `{rule}` exclude"))?;
        let mut project_filters = Vec::new();
        for project in projects {
            let name = project.name;
            project_filters.push(ProjectPathFilter {
                root: normalize_path(&project.root),
                include: GlobMatcher::new(project.include, &format!("project `{name}` include"))?,
                exclude: GlobMatcher::new(project.exclude, &format!("project `{name}` exclude"))?,
            });
        }
        Ok(Self {
            root,
            repository: scope.repository,
            allow_external_projects: scope.allow_external_projects,
            projects: project_filters,
            include,
            exclude,
        })
    }

    pub fn is_match(&self, path: &Path) -> bool {
        let path = if path.is_absolute() {
            normalize_path(path)
        } else {
            normalize_path(&self.root.join(path))
        };
        if !path.starts_with(&self.root)
            && (!self.allow_external_projects
                || !self
                    .projects
                    .iter()
                    .any(|project| path.starts_with(&project.root)))
        {
            return false;
        }
        let repo_rel = relative_slash_path(&self.root, &path);
        if self.repository && path.starts_with(&self.root) && self.matches_rule(&repo_rel, None) {
            return true;
        }
        self.projects.iter().any(|project| {
            if !path.starts_with(&project.root) {
                return false;
            }
            let project_rel = relative_slash_path(&project.root, &path);
            project.matches_project(&repo_rel, &project_rel)
                && self.matches_rule(&repo_rel, Some(&project_rel))
        })
    }

    fn matches_rule(&self, repo_rel: &str, project_rel: Option<&str>) -> bool {
        let include_project_rel = match project_rel {
            Some(rel) => self.include.is_match(rel),
            None => false,
        };
        let exclude_project_rel = match project_rel {
            Some(rel) => self.exclude.is_match(rel),
            None => false,
        };
        (self.include.is_empty() || self.include.is_match(repo_rel) || include_project_rel)
            && !self.exclude.is_match(repo_rel)
            && !exclude_project_rel
    }
}

impl ProjectPathFilter {
    fn matches_project(&self, repo_rel: &str, project_rel: &str) -> bool {
        (self.include.is_empty()
            || self.include.is_match(repo_rel)
            || self.include.is_match(project_rel))
            && !self.exclude.is_match(repo_rel)
            && !self.exclude.is_match(project_rel)
    }
}
