use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};

pub(crate) fn filter_rule_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let filter = RulePathFilter::new(root, config, rule)?;
    Ok(files
        .iter()
        .filter(|path| filter.is_match(path))
        .cloned()
        .collect())
}

pub(crate) fn filter_findings(
    root: &Path,
    config: &NoMistakesConfig,
    rule_id: &str,
    findings: Vec<super::RuleFinding>,
) -> Result<Vec<super::RuleFinding>> {
    let mut filtered = Vec::new();
    for rule in config.rule_applications(rule_id) {
        let filter = RulePathFilter::new(root, config, rule)?;
        filtered.extend(
            findings
                .iter()
                .filter(|finding| filter.is_match(&root.join(&finding.file)))
                .cloned(),
        );
    }
    super::sort_findings(&mut filtered);
    Ok(filtered)
}

pub(crate) struct RulePathFilter {
    root: PathBuf,
    repository: bool,
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
    pub(crate) fn new(root: &Path, config: &NoMistakesConfig, rule: &RuleDef) -> Result<Self> {
        let mut inferred_roots = crate::codebase::config::InferredRoots::default();
        Self::new_with_inferred(root, config, rule, &mut inferred_roots)
    }

    pub(crate) fn new_with_inferred(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        inferred_roots: &mut crate::codebase::config::InferredRoots,
    ) -> Result<Self> {
        let root = crate::codebase::ts_resolver::normalize_path(root);
        let include = GlobMatcher::new(&rule.include, &format!("rule `{}` include", rule.rule))?;
        let exclude = GlobMatcher::new(&rule.exclude, &format!("rule `{}` exclude", rule.rule))?;
        let mut projects = Vec::new();
        for project_name in &rule.projects {
            let Some(project) = config.projects.get(project_name) else {
                continue;
            };
            let Some(project_root) = super::target_project_root(&root, project, inferred_roots)
            else {
                continue;
            };
            projects.push(ProjectPathFilter {
                root: crate::codebase::ts_resolver::normalize_path(&project_root),
                include: GlobMatcher::new(
                    &project.include,
                    &format!("project `{project_name}` include"),
                )?,
                exclude: GlobMatcher::new(
                    &project.exclude,
                    &format!("project `{project_name}` exclude"),
                )?,
            });
        }

        Ok(Self {
            root,
            repository: rule.applies_to_repository() || has_test_target(rule),
            projects,
            include,
            exclude,
        })
    }

    pub(crate) fn is_match(&self, path: &Path) -> bool {
        let path = if path.is_absolute() {
            crate::codebase::ts_resolver::normalize_path(path)
        } else {
            crate::codebase::ts_resolver::normalize_path(&self.root.join(path))
        };
        if !path.starts_with(&self.root) {
            return false;
        }
        let repo_rel = relative_slash_path(&self.root, &path);
        if self.repository && self.matches_rule(&repo_rel, None) {
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

fn has_test_target(rule: &RuleDef) -> bool {
    !rule.tests.vitest.is_empty() || !rule.tests.playwright.is_empty()
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

pub(crate) struct GlobMatcher {
    globset: Option<GlobSet>,
}

impl GlobMatcher {
    pub(crate) fn new(patterns: &[String], context: &str) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        let mut count = 0usize;
        for pattern in patterns {
            let normalized = pattern.trim_start_matches("./");
            let glob_result = GlobBuilder::new(normalized)
                .literal_separator(false)
                .build();
            let glob = match glob_result {
                Ok(glob) => glob,
                Err(error) => {
                    return Err(anyhow::Error::new(error)
                        .context(format!("{context} contains invalid glob `{pattern}`")));
                }
            };
            builder.add(glob);
            count += 1;
        }
        let globset = if count == 0 {
            None
        } else {
            let result = builder.build();
            let globset = match result {
                Ok(globset) => globset,
                Err(error) => {
                    return Err(anyhow::Error::new(error)
                        .context(format!("failed to build {context} glob set")));
                }
            };
            Some(globset)
        };
        Ok(Self { globset })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.globset.is_none()
    }

    pub(crate) fn is_match(&self, rel: &str) -> bool {
        self.globset
            .as_ref()
            .is_some_and(|globset| globset.is_match(rel))
    }
}

#[cfg(test)]
mod tests;
