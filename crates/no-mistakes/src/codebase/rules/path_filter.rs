use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use no_mistakes_rule_kernel::path_filter::{self as kernel, ProjectScope, RuleScope};

mod findings;
mod markdown;
pub(crate) use findings::filter_findings;
pub(crate) use kernel::GlobMatcher;
pub(crate) use markdown::filter_markdown_rule_files;

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

pub(crate) struct RulePathFilter(kernel::RulePathFilter);

impl RulePathFilter {
    pub(crate) fn new(root: &Path, config: &NoMistakesConfig, rule: &RuleDef) -> Result<Self> {
        let mut inferred_roots = crate::codebase::config::InferredRoots::default();
        Self::new_with_inferred(root, config, rule, &mut inferred_roots)
    }

    fn new_with_external_projects(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
    ) -> Result<Self> {
        let mut inferred_roots = crate::codebase::config::InferredRoots::default();
        Self::new_with_inferred_and_external(root, config, rule, &mut inferred_roots, true)
    }

    pub(crate) fn new_with_inferred(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        inferred_roots: &mut crate::codebase::config::InferredRoots,
    ) -> Result<Self> {
        Self::new_with_inferred_and_external(root, config, rule, inferred_roots, false)
    }

    fn new_with_inferred_and_external(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        inferred_roots: &mut crate::codebase::config::InferredRoots,
        allow_external_projects: bool,
    ) -> Result<Self> {
        let root = normalize_path(root);
        let projects = rule.projects.iter().filter_map(|name| {
            let project = config.projects.get(name)?;
            let project_root = super::target_project_root(&root, project, inferred_roots)?;
            Some(ProjectScope {
                name,
                root: project_root,
                include: &project.include,
                exclude: &project.exclude,
            })
        });
        let scope = RuleScope {
            id: &rule.rule,
            include: &rule.include,
            exclude: &rule.exclude,
            // Test targets without project selectors use the configured default target.
            // With explicit projects, the repository match would bypass their filters.
            repository: rule.applies_to_repository()
                || (has_test_target(rule) && rule.projects.is_empty()),
            allow_external_projects,
        };
        kernel::RulePathFilter::new(&root, scope, projects).map(Self)
    }

    pub(crate) fn is_match(&self, path: &Path) -> bool {
        self.0.is_match(path)
    }
}

fn has_test_target(rule: &RuleDef) -> bool {
    !rule.tests.vitest.is_empty() || !rule.tests.playwright.is_empty()
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
