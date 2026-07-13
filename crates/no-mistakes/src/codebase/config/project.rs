use super::RuleConfig;
use super::{infer_nextjs_root, infer_remix_root, infer_vitejs_root};
use crate::config::v2::schema::ProjectType;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct InferredRoots {
    pub nextjs: Option<Option<PathBuf>>,
    pub remix: Option<Option<PathBuf>>,
    pub vitejs: Option<Option<PathBuf>>,
}

impl InferredRoots {
    pub fn from_visible(workspace_root: &Path, visible_paths: &[PathBuf]) -> Self {
        Self {
            nextjs: Some(super::infer_nextjs_root_from_visible(
                workspace_root,
                visible_paths,
            )),
            remix: Some(super::infer_remix_root_from_visible(
                workspace_root,
                visible_paths,
            )),
            vitejs: Some(super::infer_vitejs_root_from_visible(
                workspace_root,
                visible_paths,
            )),
        }
    }

    pub fn nextjs_root(&mut self, workspace_root: &Path) -> Option<PathBuf> {
        self.nextjs
            .get_or_insert_with(|| infer_nextjs_root(workspace_root))
            .clone()
    }

    pub fn remix_root(&mut self, workspace_root: &Path) -> Option<PathBuf> {
        self.remix
            .get_or_insert_with(|| infer_remix_root(workspace_root))
            .clone()
    }

    pub fn vitejs_root(&mut self, workspace_root: &Path) -> Option<PathBuf> {
        self.vitejs
            .get_or_insert_with(|| infer_vitejs_root(workspace_root))
            .clone()
    }
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct ProjectConfig {
    #[serde(rename = "type")]
    pub type_: Option<ProjectType>,
    pub root: Option<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub rules: Vec<String>,
}

pub(super) fn roots_for_rule(
    projects: &HashMap<String, ProjectConfig>,
    rules: &HashMap<String, RuleConfig>,
    repository_rules: &HashSet<String>,
    root: &Path,
    rule_id: &str,
) -> Vec<PathBuf> {
    let mut inferred_roots = InferredRoots::default();
    roots_for_rule_with_inferred(
        projects,
        rules,
        repository_rules,
        root,
        rule_id,
        &mut inferred_roots,
    )
}

pub(super) fn roots_for_rule_with_inferred(
    projects: &HashMap<String, ProjectConfig>,
    rules: &HashMap<String, RuleConfig>,
    repository_rules: &HashSet<String>,
    root: &Path,
    rule_id: &str,
    inferred_roots: &mut InferredRoots,
) -> Vec<PathBuf> {
    if rules.get(rule_id).is_some_and(|rule| !rule.enabled) {
        return Vec::new();
    }

    let mut project_roots = Vec::new();
    for project in projects.values() {
        if !project.rules.iter().any(|rule| rule == rule_id) {
            continue;
        }
        if let Some(project_root) = project.effective_root_with_cache(root, inferred_roots) {
            project_roots.push(project_root);
        } else {
            project_roots.push(root.to_path_buf());
        }
    }
    if repository_rules.contains(rule_id) {
        project_roots.push(root.to_path_buf());
    }
    if !project_roots.is_empty() {
        project_roots.sort();
        project_roots.dedup();
        return project_roots;
    }

    if projects.is_empty() || rules.contains_key(rule_id) {
        vec![root.to_path_buf()]
    } else {
        Vec::new()
    }
}

impl ProjectConfig {
    pub fn effective_root(&self, workspace_root: &Path) -> Option<PathBuf> {
        let mut inferred_roots = InferredRoots::default();
        self.effective_root_with_cache(workspace_root, &mut inferred_roots)
    }

    pub(crate) fn effective_root_with_cache(
        &self,
        workspace_root: &Path,
        inferred_roots: &mut InferredRoots,
    ) -> Option<PathBuf> {
        match self.root.as_deref() {
            Some(root) => Some(workspace_root.join(root)),
            None if self.type_ == Some(ProjectType::Nextjs) => {
                inferred_roots.nextjs_root(workspace_root)
            }
            None if self.type_ == Some(ProjectType::Remix) => {
                inferred_roots.remix_root(workspace_root)
            }
            None if self.type_ == Some(ProjectType::Vitejs) => {
                inferred_roots.vitejs_root(workspace_root)
            }
            None => None,
        }
    }
}
