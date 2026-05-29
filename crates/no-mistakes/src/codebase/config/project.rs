use super::RuleConfig;
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
    if rules.get(rule_id).is_some_and(|rule| !rule.enabled) {
        return Vec::new();
    }

    let mut project_roots = Vec::new();
    let mut inferred_roots = InferredRoots::default();
    for project in projects.values() {
        if !project.rules.iter().any(|rule| rule == rule_id) {
            continue;
        }
        if let Some(project_root) = project.effective_root_with_cache(root, &mut inferred_roots) {
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
            None if self.type_ == Some(ProjectType::Nextjs) => inferred_roots
                .nextjs
                .get_or_insert_with(|| infer_nextjs_root(workspace_root))
                .clone(),
            None if self.type_ == Some(ProjectType::Remix) => inferred_roots
                .remix
                .get_or_insert_with(|| infer_remix_root(workspace_root))
                .clone(),
            None if self.type_ == Some(ProjectType::Vitejs) => inferred_roots
                .vitejs
                .get_or_insert_with(|| infer_vitejs_root(workspace_root))
                .clone(),
            None => None,
        }
    }
}

fn infer_framework_root(
    root: &Path,
    config_names: &[&str],
    filter: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let mut roots = crate::codebase::ts_source::discover_with_basenames(root, &[], config_names)
        .into_iter()
        .filter(|path| filter(path))
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    match roots.as_slice() {
        [root] => Some(root.clone()),
        _ => None,
    }
}

pub fn infer_nextjs_root(root: &Path) -> Option<PathBuf> {
    infer_framework_root(
        root,
        &[
            "next.config.js",
            "next.config.mjs",
            "next.config.ts",
            "next.config.mts",
        ],
        |_| true,
    )
}

fn is_remix_vite_config(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|content| {
            content.contains("@remix-run/dev") || content.contains("vitePlugin as remix")
        })
        .unwrap_or(false)
}

pub fn infer_remix_root(root: &Path) -> Option<PathBuf> {
    // Try remix.config.* files first
    let remix_config_root = infer_framework_root(
        root,
        &[
            "remix.config.js",
            "remix.config.ts",
            "remix.config.mjs",
            "remix.config.mts",
            "remix.config.cjs",
            "remix.config.cts",
        ],
        |_| true,
    );
    if remix_config_root.is_some() {
        return remix_config_root;
    }

    // Otherwise, try vite.config.* files that import Remix plugin
    infer_framework_root(
        root,
        &[
            "vite.config.js",
            "vite.config.ts",
            "vite.config.mjs",
            "vite.config.mts",
            "vite.config.cjs",
            "vite.config.cts",
        ],
        is_remix_vite_config,
    )
}

pub fn infer_vitejs_root(root: &Path) -> Option<PathBuf> {
    // Try vite.config.* files that are NOT Remix configurations
    infer_framework_root(
        root,
        &[
            "vite.config.js",
            "vite.config.ts",
            "vite.config.mjs",
            "vite.config.mts",
            "vite.config.cjs",
            "vite.config.cts",
        ],
        |path| !is_remix_vite_config(path),
    )
}
